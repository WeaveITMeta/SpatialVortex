//! Train a multiple-choice model on REAL benchmark data and measure it.
//!
//! This is the first Vortex model trained end-to-end on real data via
//! autodiff. Everything in the path is trainable: the token embedding table,
//! all four attention projections (w_q/w_k/w_v/w_o), and the scorer.
//!
//! ## The task
//! Given a question and N choices, pick the correct one. Scored against the
//! 25.0% chance rate of the ingested mix, on a **held-out split the model never
//! trains on**. This is the same task where the honest baseline recorded MMLU
//! and HellaSwag at near chance — the failure mode being attacked directly.
//!
//! ## Architecture
//! ```text
//!   question -> hashed tokens -> Embedding -> mean-pool -> q  [1, D]
//!   choice_i -> hashed tokens -> Embedding -> mean-pool -> c_i [N, D]
//!   attended = MHA(q, context = c)            question attends over choices
//!   logit_i  = <attended, c_i> / sqrt(D)
//!   loss     = cross_entropy(logits, answer_idx)
//! ```
//!
//! Corpus: `data/mc_corpus.jsonl`, produced by the ingest tool that normalises
//! six incompatible answer paradigms (integer index, letter key, separate
//! distractor fields, string-index, ...) into one shape.
//!
//! ```bash
//! cargo run --release --example mc_train
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use burn::backend::{Autodiff, NdArray};
use burn::module::Module;
use burn::nn::{Embedding, EmbeddingConfig, Linear, LinearConfig};
use burn::optim::{GradientsParams, Optimizer};
use burn::tensor::backend::Backend;
use burn::tensor::{Int, Tensor, TensorData};

use vortex::ml::burn_attention::{BurnAttentionConfig, BurnMultiHeadAttention};
use vortex::ml::generative_arch::SubwordTokenizer;
use vortex::ml::training::SsoConfig;

type B = Autodiff<NdArray>;
type Dev = <B as Backend>::Device;

const DIM: usize = 128;
/// Fixed so the untrained baseline is reproducible. It drifted 24.9%-28.5%
/// across runs while Burn's init was unseeded, which made any comparison
/// against "untrained" meaningless.
const SEED: u64 = 0x5EED_2026;
const TOKENIZER_DIR: &str = "models/mc_tokenizer";
const BPE_MERGES: usize = 1000;
const MAX_TOK: usize = 48;
const EPOCHS: usize = 6;
const LR: f64 = 2e-3;
const HELD_OUT: f32 = 0.2;

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

/// Encode text with the shared BPE tokenizer.
///
/// Ids are dense `0..vocab_size` by construction -- every `add_token` consumes a
/// fresh `next_id` and the vocabulary is keyed by id -- so `vocab_size()` is a
/// valid `num_embeddings` for the Burn table.
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
            // Scores [attended ; choice ; attended*choice] -> 1.
            // A plain dot product between the attention output and the choice
            // set is near-circular: `attended` IS a weighted mix of those same
            // vectors, so the logits collapse toward a Gram matrix over choices
            // and the model sits at ln(N) forever. The elementwise product term
            // is what gives it an actual contrast signal per choice.
            proj: LinearConfig::new(DIM * 3, 1).init(device),
        }
    }

    /// Mean-pooled embedding of one token sequence -> `[1, DIM]`.
    fn pool(&self, toks: &[usize], device: &B::Device) -> Tensor<B, 2> {
        let ids: Vec<i32> = toks.iter().map(|&t| t as i32).collect();
        let n = ids.len();
        let t = Tensor::<B, 2, Int>::from_data(TensorData::new(ids, [1, n]), device);
        // [1, n, DIM] -> mean over tokens -> [1, DIM]
        self.embed.forward(t).mean_dim(1).reshape([1, DIM])
    }

    /// Logits over the choices of one item.
    fn logits(&self, item: &Item, device: &B::Device) -> Tensor<B, 2> {
        let q = self.pool(&item.question, device);
        let cs: Vec<Tensor<B, 2>> = item.choices.iter().map(|c| self.pool(c, device)).collect();
        let n = cs.len();
        let ctx = Tensor::cat(cs.clone(), 0); // [n, DIM]

        // Context-aware question representation, then an explicit contrast
        // against each candidate.
        let attended = self.attn.forward(q, ctx.clone()); // [1, DIM]
        let att_n = Tensor::cat(vec![attended; n], 0); // [n, DIM]
        let feat = Tensor::cat(
            vec![att_n.clone(), ctx.clone(), att_n * ctx],
            1,
        ); // [n, 3*DIM]
        self.proj.forward(feat).reshape([1, n])
    }
}

fn cross_entropy(logits: Tensor<B, 2>, target: usize) -> Tensor<B, 1> {
    let logp = burn::tensor::activation::log_softmax(logits, 1);
    let [_, n] = logp.dims();
    let mask: Vec<f32> = (0..n).map(|i| if i == target { 1.0 } else { 0.0 }).collect();
    let m = Tensor::<B, 2>::from_data(
        TensorData::new(mask, [1, n]),
        &logp.device(),
    );
    (logp * m).sum().neg()
}

fn main() {
    let device: Dev = Default::default();
    // Reproducible init: without this the untrained baseline drifted 24.9%-28.5%
    // between runs, so "beats untrained" was not a stable claim.
    <NdArray as Backend>::seed(SEED);

    let path = "data/mc_corpus.jsonl";
    let mut raw = load_raw(path);
    if raw.is_empty() {
        eprintln!("no usable items in {path}");
        std::process::exit(1);
    }

    // Deterministic split, hash-shuffled so every dataset lands on both sides
    // rather than splitting by file order.
    let mut idx: Vec<usize> = (0..raw.len()).collect();
    idx.sort_by_key(|&i| {
        let mut h = DefaultHasher::new();
        (i as u64).wrapping_mul(0x9E3779B97F4A7C15).hash(&mut h);
        h.finish()
    });
    raw = idx.into_iter().map(|i| raw[i].clone()).collect();
    let split = ((raw.len() as f32) * (1.0 - HELD_OUT)) as usize;

    // BPE is trained on the TRAIN SPLIT ONLY. Fitting the vocabulary on the
    // full corpus would leak held-out text into the tokenisation and inflate
    // the held-out score.
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

    println!("Vortex multiple-choice — trained on REAL benchmark data
");
    println!("  corpus      : {} items ({} train / {} held-out)", items.len(), train.len(), test.len());
    let chance: f32 = test.iter().map(|i| 1.0 / i.choices.len() as f32).sum::<f32>() / test.len() as f32;
    println!("  chance      : {:.1}%", chance * 100.0);
    println!("  tokenizer   : BPE, {} merges, vocab {}", BPE_MERGES, tok.vocab_size());
    println!("  persisted   : {TOKENIZER_DIR}");
    println!("  seed        : {SEED:#x}");
    println!("  dim         : {DIM}");

    let mut model = McModel::<B>::new(tok.vocab_size(), &device);
    println!("  parameters  : {}
", model.num_params());

    let eval = |m: &McModel<B>, set: &[Item]| -> (f32, std::collections::BTreeMap<String, (usize, usize)>) {
        let mut hits = 0usize;
        let mut per: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
        for it in set {
            let l = m.logits(it, &device).into_data().convert::<f32>();
            let v = l.as_slice::<f32>().unwrap();
            let pred = v
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);
            let e = per.entry(it.dataset.clone()).or_insert((0, 0));
            e.1 += 1;
            if pred == it.answer {
                hits += 1;
                e.0 += 1;
            }
        }
        (hits as f32 / set.len().max(1) as f32, per)
    };

    let (untrained, _) = eval(&model, test);
    println!("  untrained held-out accuracy: {:.1}%\n", untrained * 100.0);

    let mut optim = SsoConfig::new().with_momentum(0.9).init::<B, McModel<B>>();

    for epoch in 1..=EPOCHS {
        let mut loss_sum = 0.0f32;
        for it in train {
            let logits = model.logits(it, &device);
            let loss = cross_entropy(logits, it.answer);
            loss_sum += loss.clone().into_scalar();
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            model = optim.step(LR, model, grads);
        }
        let (acc, _) = eval(&model, test);
        println!(
            "  epoch {epoch}  mean loss {:.4}   held-out accuracy {:.1}%",
            loss_sum / train.len() as f32,
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

    let beats_chance = final_acc > chance + 0.05;
    println!(
        "\n  RESULT: {}",
        if beats_chance {
            "PASS — learned real signal on held-out benchmark data"
        } else {
            "FAIL — did not separate from chance"
        }
    );
    if !beats_chance {
        std::process::exit(1);
    }
}
