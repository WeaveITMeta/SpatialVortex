//! Batched version of `mc_train`: ONE autodiff graph per batch, not per item.
//!
//! `mc_train.rs` builds a fresh graph for every single item — `logits()` ->
//! `cross_entropy` -> `backward()` -> `optim.step()` in a per-item loop. That
//! costs one full optimizer step (power iteration + Newton-Schulz per rank-2
//! parameter) per item, and it leaves the autodiff server churning ~60 nodes
//! per item.
//!
//! Measured on `data/mc_corpus.jsonl` (3,600 items, DIM=128, MAX_TOK=48):
//!
//! ```text
//!                         peak RSS   throughput      6-epoch held-out
//!   mc_train (per item)    239.5 MB    ~15 pass/s    (2 epochs in 540 s)
//!   this, batch =   64      34.9 MB   ~487 pass/s    25.8%  (lr 2e-3)
//!   this, batch =   64      34.6 MB   ~460 pass/s    27.2%  (lr 1.28e-2)
//! ```
//!
//! Peak memory now fits `31.4 MB + 0.159 MB * batch` (measured at batch
//! 8/16/64/512/1024), i.e. it is a function of the BATCH, not of the corpus.
//! That is what makes 130,960 Mountain Ascension instances reachable: the
//! corpus size drops out of the memory term entirely and only costs time.
//!
//! Note the tradeoff this exposes: batching by N cuts optimizer steps per
//! epoch by N. SSO's `update_2d` orthogonalises the update via Newton-Schulz,
//! so per-step travel is set by `lr` rather than by gradient magnitude, and
//! total travel goes as `lr * n_steps`. Cutting steps therefore needs `lr`
//! raised to compensate — which is why the `lr 1.28e-2` row above both trains
//! fastest AND reaches the best held-out number of any configuration measured.
//!
//! What changes here:
//!   * token sequences are padded to the batch max and the padding is MASKED
//!     out of the mean-pool, so pad slots do not drag the mean toward `<PAD>`;
//!   * the choice axis is padded to the batch max and masked in BOTH the
//!     attention softmax and the final logits, so 3-choice and 4-choice items
//!     share a batch;
//!   * attention runs through `BurnMultiHeadAttention::forward_batched`, whose
//!     context carries a leading batch axis (the rank-2 `forward` shares one
//!     context across the batch, which is the wrong contract here);
//!   * `backward()` / `GradientsParams::from_grads` / `optim.step` fire ONCE
//!     per batch;
//!   * eval runs on `model.valid()` — the plain `NdArray` inner module — so
//!     the held-out pass registers no autodiff nodes at all.
//!
//! ```bash
//! cargo run --release --example mc_train_batched
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use burn::backend::{Autodiff, NdArray};
use burn::module::{AutodiffModule, Module};
use burn::nn::loss::CrossEntropyLossConfig;
use burn::nn::{Embedding, EmbeddingConfig, Linear, LinearConfig};
use burn::optim::{GradientsParams, Optimizer};
use burn::tensor::activation::log_softmax;
use burn::tensor::backend::Backend;
use burn::tensor::{Int, Tensor, TensorData};

use vortex::ml::burn_attention::{BurnAttentionConfig, BurnMultiHeadAttention};
use vortex::ml::generative_arch::SubwordTokenizer;
use vortex::ml::training::SsoConfig;

type B = Autodiff<NdArray>;
type Inner = NdArray;
type Dev = <B as Backend>::Device;

const DIM: usize = 128;
const SEED: u64 = 0x5EED_2026;
const TOKENIZER_DIR: &str = "models/mc_tokenizer";
const BPE_MERGES: usize = 1000;
const MAX_TOK: usize = 48;
const HELD_OUT: f32 = 0.2;

/// Read a `usize`/`f64` knob from the environment so batch size, learning rate
/// and epoch count can be swept without a 4-minute rebuild.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// `SubwordTokenizer::pad_id`. Row 0 of the embedding table is `<PAD>`; it is a
/// real trainable row, which is exactly why it has to be masked out rather than
/// left to accumulate gradient on every item.
const PAD: i32 = 0;

/// Additive mask sentinel. Finite on purpose: `softmax` subtracts a detached
/// per-row max before `exp`, and `-inf - -inf` is `NaN`.
const NEG: f32 = -1.0e30;

#[derive(Debug, Clone)]
struct RawItem {
    dataset: String,
    question: String,
    choices: Vec<String>,
    answer: usize,
}

#[derive(Debug, Clone)]
struct Item {
    dataset: String,
    question: Vec<usize>,
    choices: Vec<Vec<usize>>,
    answer: usize,
}

fn encode(tok: &SubwordTokenizer, s: &str) -> Vec<usize> {
    tok.tokenize(s).into_iter().take(MAX_TOK).map(|t| t as usize).collect()
}

fn load_raw(path: &str) -> Vec<RawItem> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}\nRun the ingest tool first.");
        std::process::exit(1);
    });
    let mut out = Vec::new();
    for line in text.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let q = v["question"].as_str().unwrap_or("");
        let ch = match v["choices"].as_array() {
            Some(c) => c,
            None => continue,
        };
        let ans = v["answer_idx"].as_u64().unwrap_or(0) as usize;
        if q.is_empty() || ch.len() < 2 || ans >= ch.len() {
            continue;
        }
        let choices: Vec<String> = ch.iter().map(|c| c.as_str().unwrap_or("").to_string()).collect();
        if choices.iter().any(|c| c.trim().is_empty()) {
            continue;
        }
        out.push(RawItem {
            dataset: v["dataset"].as_str().unwrap_or("?").to_string(),
            question: q.to_string(),
            choices,
            answer: ans,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Padded batch
// ---------------------------------------------------------------------------

/// One microbatch, padded on the CPU once and uploaded as five tensors.
///
/// Two independent ragged axes have to be rectangularised:
///   * **token length** — per sequence, padded to `s_q` / `s_c`;
///   * **choice count** — 3 or 4 in this corpus, padded to `c_max`.
/// Each gets its own mask, and the two are applied at different points.
struct Batch {
    /// Items in the batch.
    n: usize,
    /// Choices per item after padding.
    c_max: usize,
    /// Question token ids, `[n, s_q]`.
    q_ids: Tensor<B, 2, Int>,
    /// `1.0` on real question tokens, `0.0` on pad, `[n, s_q]`.
    q_tok_mask: Tensor<B, 2>,
    /// Choice token ids flattened over the choice axis, `[n * c_max, s_c]`.
    c_ids: Tensor<B, 2, Int>,
    /// `1.0` on real choice tokens, `0.0` on pad, `[n * c_max, s_c]`.
    c_tok_mask: Tensor<B, 2>,
    /// Additive `0 / NEG` over the choice axis, `[n, c_max]`.
    choice_bias: Tensor<B, 2>,
    /// Correct choice index per item, `[n]`.
    targets: Tensor<B, 1, Int>,
}

impl Batch {
    fn build(items: &[Item], device: &Dev) -> Self {
        let n = items.len();
        let c_max = items.iter().map(|i| i.choices.len()).max().unwrap_or(1);
        let s_q = items.iter().map(|i| i.question.len()).max().unwrap_or(1);
        let s_c = items
            .iter()
            .flat_map(|i| i.choices.iter().map(|c| c.len()))
            .max()
            .unwrap_or(1);

        let mut q_ids = vec![PAD; n * s_q];
        let mut q_mask = vec![0.0f32; n * s_q];
        let mut c_ids = vec![PAD; n * c_max * s_c];
        let mut c_mask = vec![0.0f32; n * c_max * s_c];
        let mut bias = vec![NEG; n * c_max];
        let mut targets = vec![0i32; n];

        for (bi, it) in items.iter().enumerate() {
            for (ti, &t) in it.question.iter().enumerate() {
                q_ids[bi * s_q + ti] = t as i32;
                q_mask[bi * s_q + ti] = 1.0;
            }
            for (ci, ch) in it.choices.iter().enumerate() {
                bias[bi * c_max + ci] = 0.0;
                let base = (bi * c_max + ci) * s_c;
                for (ti, &t) in ch.iter().enumerate() {
                    c_ids[base + ti] = t as i32;
                    c_mask[base + ti] = 1.0;
                }
            }
            targets[bi] = it.answer as i32;
        }

        Self {
            n,
            c_max,
            q_ids: Tensor::from_data(TensorData::new(q_ids, [n, s_q]), device),
            q_tok_mask: Tensor::from_data(TensorData::new(q_mask, [n, s_q]), device),
            c_ids: Tensor::from_data(TensorData::new(c_ids, [n * c_max, s_c]), device),
            c_tok_mask: Tensor::from_data(TensorData::new(c_mask, [n * c_max, s_c]), device),
            choice_bias: Tensor::from_data(TensorData::new(bias, [n, c_max]), device),
            targets: Tensor::from_data(TensorData::new(targets, [n]), device),
        }
    }

    /// Same padding, on the inner (non-autodiff) backend, for eval.
    fn build_inner(items: &[Item], device: &Dev) -> BatchInner {
        let b = Self::build(items, device);
        BatchInner {
            n: b.n,
            c_max: b.c_max,
            q_ids: b.q_ids.inner(),
            q_tok_mask: b.q_tok_mask.inner(),
            c_ids: b.c_ids.inner(),
            c_tok_mask: b.c_tok_mask.inner(),
            choice_bias: b.choice_bias.inner(),
        }
    }
}

struct BatchInner {
    n: usize,
    c_max: usize,
    q_ids: Tensor<Inner, 2, Int>,
    q_tok_mask: Tensor<Inner, 2>,
    c_ids: Tensor<Inner, 2, Int>,
    c_tok_mask: Tensor<Inner, 2>,
    choice_bias: Tensor<Inner, 2>,
}

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

#[derive(Module, Debug)]
struct McModel<B: Backend> {
    embed: Embedding<B>,
    attn: BurnMultiHeadAttention<B>,
    proj: Linear<B>,
}

impl<B: Backend> McModel<B> {
    fn new(vocab: usize, device: &B::Device) -> Self {
        Self {
            embed: EmbeddingConfig::new(vocab, DIM).init(device),
            attn: BurnMultiHeadAttention::new(
                &BurnAttentionConfig::new().with_model_dim(DIM).with_num_heads(4).with_head_dim(32),
                device,
            ),
            proj: LinearConfig::new(DIM * 3, 1).init(device),
        }
    }

    /// Masked mean-pool over a padded batch of token sequences.
    ///
    /// `ids`  `[rows, seq]`, `mask` `[rows, seq]` -> `[rows, DIM]`.
    ///
    /// The mask is what makes padding free. Without it the mean is
    /// `(Σ_real + (seq - n_i)·e_pad) / seq`: every real token is scaled by
    /// `n_i / seq`, which varies per item, and a constant `e_pad` direction is
    /// injected into every vector. That is a length-dependent confound — short
    /// names get attenuated relative to long ones — and `e_pad` collects
    /// gradient on every item, training row 0 into a general-purpose bias.
    /// Multiplying by the mask zeroes the pad contributions, and dividing by
    /// the true token count (not `seq`) restores the correct denominator.
    fn pool(&self, ids: Tensor<B, 2, Int>, mask: Tensor<B, 2>) -> Tensor<B, 2> {
        let [rows, seq] = ids.dims();
        // Embedding::forward: [batch, seq] -> [batch, seq, d_model].
        let e = self.embed.forward(ids);
        let m = mask.clone().reshape([rows, seq, 1]); // broadcasts over DIM
        let summed = (e * m).sum_dim(1); // [rows, 1, DIM]
        // clamp_min(1.0) guards a fully-padded row (a pad CHOICE slot): the
        // numerator is already 0 there, so the result is the zero vector
        // instead of 0/0 = NaN.
        let counts = mask.sum_dim(1).clamp_min(1.0).reshape([rows, 1, 1]);
        (summed / counts).reshape([rows, DIM])
    }

    /// Logits over the padded choice axis, `[n, c_max]`.
    ///
    /// Padded choice slots come back as `NEG` and can never be selected by
    /// `argmax` nor receive probability mass in the cross-entropy.
    fn logits(
        &self,
        q_ids: Tensor<B, 2, Int>,
        q_tok_mask: Tensor<B, 2>,
        c_ids: Tensor<B, 2, Int>,
        c_tok_mask: Tensor<B, 2>,
        choice_bias: Tensor<B, 2>,
        n: usize,
        c_max: usize,
    ) -> Tensor<B, 2> {
        let q = self.pool(q_ids, q_tok_mask).reshape([n, 1, DIM]); // [n, 1, DIM]
        let c = self.pool(c_ids, c_tok_mask).reshape([n, c_max, DIM]); // [n, C, DIM]

        // Attention bias: [n, 1, 1, C], broadcast to [n, heads, 1, C].
        let attn_bias = choice_bias.clone().reshape([n, 1, 1, c_max]);
        let attended = self.attn.forward_batched(q, c.clone(), Some(attn_bias)); // [n, 1, DIM]

        // Broadcast the single attended vector across the choice axis, then the
        // same [att ; c ; att*c] contrast as the unbatched model.
        let att = attended.expand([n, c_max, DIM]);
        let feat = Tensor::cat(vec![att.clone(), c.clone(), att * c], 2); // [n, C, 3*DIM]

        self.proj.forward(feat).reshape([n, c_max]) + choice_bias
    }
}

fn main() {
    let device: Dev = Default::default();
    <NdArray as Backend>::seed(SEED);

    // Items per autodiff graph. Peak memory is linear in this; everything else
    // in the loop is constant.
    let batch = env_usize("MC_BATCH", 64);
    let epochs = env_usize("MC_EPOCHS", 6);
    let lr = env_f64("MC_LR", 2e-3);

    let path = "data/mc_corpus.jsonl";
    let mut raw = load_raw(path);
    if raw.is_empty() {
        eprintln!("no usable items in {path}");
        std::process::exit(1);
    }

    let mut idx: Vec<usize> = (0..raw.len()).collect();
    idx.sort_by_key(|&i| {
        let mut h = DefaultHasher::new();
        (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).hash(&mut h);
        h.finish()
    });
    raw = idx.into_iter().map(|i| raw[i].clone()).collect();
    let split = ((raw.len() as f32) * (1.0 - HELD_OUT)) as usize;

    let train_text: Vec<String> = raw[..split]
        .iter()
        .flat_map(|r| std::iter::once(r.question.clone()).chain(r.choices.iter().cloned()))
        .collect();
    let tok = SubwordTokenizer::load_or_train(
        std::path::Path::new(TOKENIZER_DIR),
        &train_text,
        BPE_MERGES,
        DIM,
    );

    let items: Vec<Item> = raw
        .iter()
        .map(|r| Item {
            dataset: r.dataset.clone(),
            question: encode(&tok, &r.question),
            choices: r.choices.iter().map(|c| encode(&tok, c)).collect(),
            answer: r.answer,
        })
        .filter(|i| !i.question.is_empty() && !i.choices.iter().any(|c| c.is_empty()))
        .collect();
    let split = split.min(items.len().saturating_sub(1));
    let (train, test) = items.split_at(split);

    println!("Vortex multiple-choice — BATCHED autodiff\n");
    println!("  corpus      : {} items ({} train / {} held-out)", items.len(), train.len(), test.len());
    println!("  batch       : {batch} items per autodiff graph");
    let chance: f32 = test.iter().map(|i| 1.0 / i.choices.len() as f32).sum::<f32>() / test.len() as f32;
    println!("  chance      : {:.1}%", chance * 100.0);
    println!("  vocab       : {}", tok.vocab_size());

    let mut model = McModel::<B>::new(tok.vocab_size(), &device);
    println!("  parameters  : {}\n", model.num_params());

    // Bucket by choice count so a 3-choice item never pads out to 4. Within a
    // bucket, sort by question length so token padding is near-zero too. This
    // is pure waste reduction — the masks above are what make it *correct*.
    let mut order: Vec<usize> = (0..train.len()).collect();
    order.sort_by_key(|&i| (train[i].choices.len(), train[i].question.len()));

    // Eval runs on the INNER backend. `model.valid()` strips the autodiff
    // wrapper, so a 720-item held-out pass registers zero nodes in the autodiff
    // server. Running eval on the autodiff model — as the unbatched example
    // does — leaves every one of those graphs resident until the next
    // `backward()` sweeps them.
    let eval = |m: &McModel<B>, set: &[Item]| -> (f32, std::collections::BTreeMap<String, (usize, usize)>) {
        let inner: McModel<Inner> = m.valid();
        let mut hits = 0usize;
        let mut per: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
        for chunk in set.chunks(batch) {
            let b = Batch::build_inner(chunk, &device);
            let l = inner
                .logits(b.q_ids, b.q_tok_mask, b.c_ids, b.c_tok_mask, b.choice_bias, b.n, b.c_max)
                .into_data()
                .convert::<f32>();
            let v = l.as_slice::<f32>().unwrap();
            for (row, it) in chunk.iter().enumerate() {
                let s = &v[row * b.c_max..(row + 1) * b.c_max];
                let pred = s
                    .iter()
                    .enumerate()
                    .max_by(|a, c| a.1.partial_cmp(c.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let e = per.entry(it.dataset.clone()).or_insert((0, 0));
                e.1 += 1;
                if pred == it.answer {
                    hits += 1;
                    e.0 += 1;
                }
            }
        }
        (hits as f32 / set.len().max(1) as f32, per)
    };

    let (untrained, _) = eval(&model, test);
    println!("  untrained held-out accuracy: {:.1}%\n", untrained * 100.0);

    let mut optim = SsoConfig::new().with_momentum(0.9).init::<B, McModel<B>>();
    let loss_fn = CrossEntropyLossConfig::new().init::<B>(&device);

    for epoch in 1..=epochs {
        let mut loss_sum = 0.0f32;
        let mut batches = 0usize;

        for chunk in order.chunks(batch) {
            let batch_items: Vec<Item> = chunk.iter().map(|&i| train[i].clone()).collect();
            let b = Batch::build(&batch_items, &device);

            let logits = model.logits(
                b.q_ids,
                b.q_tok_mask,
                b.c_ids,
                b.c_tok_mask,
                b.choice_bias,
                b.n,
                b.c_max,
            );

            // Mean over the batch. CrossEntropyLoss does log_softmax(dim 1) ->
            // gather(target) -> mean, so the reduction is already per-item mean.
            let loss = loss_fn.forward(logits, b.targets);
            loss_sum += loss.clone().into_scalar();
            batches += 1;

            // ONE graph, ONE backward, ONE optimizer step for the whole batch.
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            model = optim.step(lr, model, grads);
        }

        let (acc, _) = eval(&model, test);
        println!(
            "  epoch {epoch}  mean loss {:.4}   held-out accuracy {:.1}%",
            loss_sum / batches.max(1) as f32,
            acc * 100.0
        );
    }

    let (final_acc, per) = eval(&model, test);
    println!("\n  --- held-out accuracy by dataset ---");
    for (name, (hit, tot)) in &per {
        println!("    {name:<12} {:>5.1}%   ({hit}/{tot})", 100.0 * *hit as f32 / *tot as f32);
    }
    println!("\n  chance          : {:.1}%", chance * 100.0);
    println!("  untrained       : {:.1}%", untrained * 100.0);
    println!("  trained         : {:.1}%", final_acc * 100.0);
    println!("  lift over chance: {:+.1} points", (final_acc - chance) * 100.0);
}

/// Kept only so the manual cross-entropy stays available for comparison against
/// `CrossEntropyLoss`; both reduce to a per-item mean over the batch.
#[allow(dead_code)]
fn cross_entropy_batched(logits: Tensor<B, 2>, targets: Tensor<B, 1, Int>) -> Tensor<B, 1> {
    let [n, _] = logits.dims();
    let logp = log_softmax(logits, 1);
    logp.gather(1, targets.reshape([n, 1])).reshape([n]).mean().neg()
}
