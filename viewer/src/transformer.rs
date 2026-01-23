use murmur3::murmur3_32;
use std::io::Cursor;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device, Kind, Tensor};

fn reduce_digits(mut n: u64) -> u8 {
    if n == 0 { return 9; }
    while n > 9 {
        let mut sum = 0u64;
        while n > 0 { sum += n % 10; n /= 10; }
        n = sum;
    }
    if n == 0 { 9 } else { n as u8 }
}

fn string_to_class(s: &str) -> i64 {
    let mut cursor = Cursor::new(s.as_bytes());
    let h = murmur3_32(&mut cursor, 0).unwrap_or(0) as u64;
    let d = reduce_digits(h);
    (d.saturating_sub(1)) as i64
}

fn ids_from_text(s: &str, max_len: usize) -> Vec<i64> {
    let mut ids = Vec::with_capacity(max_len);
    for b in s.as_bytes().iter().take(max_len) { ids.push((*b as i64) & 0xFFFF); }
    ids
}

pub struct TransformerNet {
    pub vs: nn::VarStore,
    token_emb: nn::Embedding,
    pos_emb: nn::Embedding,
    w_q: nn::Linear,
    w_k: nn::Linear,
    w_v: nn::Linear,
    w_o: nn::Linear,
    ln1: nn::LayerNorm,
    ff1: nn::Linear,
    ff2: nn::Linear,
    ln2: nn::LayerNorm,
    head: nn::Linear,
    d_model: i64,
    n_heads: i64,
    d_head: i64,
    max_len: i64,
    sacred_idx: [i64; 3],
    sacred_vals: [f64; 3],
}

impl TransformerNet {
    pub fn new(vocab_size: i64, d_model: i64, n_heads: i64, max_len: i64, ff_dim: i64) -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let root = &vs.root();
        let token_emb = nn::embedding(root / "tok", vocab_size, d_model, Default::default());
        let pos_emb = nn::embedding(root / "pos", max_len, d_model, Default::default());
        let w_q = nn::linear(root / "wq", d_model, d_model, Default::default());
        let w_k = nn::linear(root / "wk", d_model, d_model, Default::default());
        let w_v = nn::linear(root / "wv", d_model, d_model, Default::default());
        let w_o = nn::linear(root / "wo", d_model, d_model, Default::default());
        let ln1 = nn::layer_norm(root / "ln1", vec![d_model], Default::default());
        let ff1 = nn::linear(root / "ff1", d_model, ff_dim, Default::default());
        let ff2 = nn::linear(root / "ff2", ff_dim, d_model, Default::default());
        let ln2 = nn::layer_norm(root / "ln2", vec![d_model], Default::default());
        let head = nn::linear(root / "head", d_model, 9, Default::default());
        let d_head = d_model / n_heads;
        Self {
            vs,
            token_emb,
            pos_emb,
            w_q,
            w_k,
            w_v,
            w_o,
            ln1,
            ff1,
            ff2,
            ln2,
            head,
            d_model,
            n_heads,
            d_head,
            max_len,
            sacred_idx: [2, 5, 8],
            sacred_vals: [3.0, 6.0, 9.0],
        }
    }

    fn mha(&self, x: &Tensor) -> Tensor {
        let bsz = x.size()[0];
        let seq = x.size()[1];
        let q = self.w_q.forward(x);
        let k = self.w_k.forward(x);
        let v = self.w_v.forward(x);
        let h = self.n_heads as i64;
        let dh = self.d_head as i64;
        let reshape = |t: Tensor| t.view([bsz, seq, h, dh]).transpose(1, 2);
        let q = reshape(q);
        let k = reshape(k);
        let v = reshape(v);
        let attn_scores = (&q.matmul(&k.transpose(-2, -1))) / (dh as f64).sqrt();
        let attn = attn_scores.softmax(-1, Kind::Float);
        let ctx = attn.matmul(&v).transpose(1, 2).contiguous().view([bsz, seq, self.d_model]);
        self.w_o.forward(&ctx)
    }

    fn clamp_sacred(&self, logits: &Tensor) -> Tensor {
        let mut y = logits.shallow_clone();
        for (i, &idx) in self.sacred_idx.iter().enumerate() {
            y.narrow(1, idx, 1).fill_(self.sacred_vals[i]);
        }
        y
    }

    pub fn forward_ids(&self, ids: &[u32]) -> Tensor {
        let seq_len = ids.len().min(self.max_len as usize) as i64;
        let id_t = Tensor::of_slice(&ids.iter().take(seq_len as usize).map(|x| *x as i64).collect::<Vec<_>>()).view([1, seq_len]);
        let pos_t = Tensor::arange(seq_len, (Kind::Int64, Device::Cpu)).view([1, seq_len]);
        let x = self.token_emb.forward(&id_t) + self.pos_emb.forward(&pos_t);
        let y = self.ln1.forward(&(x.copy() + self.mha(&x)));
        let y2 = self.ff2.forward(&self.ff1.forward(&y).relu());
        let z = self.ln2.forward(&(y + y2));
        let pooled = z.mean_dim([1].as_ref(), false, Kind::Float);
        let logits = self.head.forward(&pooled);
        self.clamp_sacred(&logits)
    }

    pub fn predict_digit_from_ids(&self, ids: &[u32]) -> i32 {
        let logits = self.forward_ids(ids);
        (logits.argmax(-1, false).int64_value(&[0]) as i32) + 1
    }

    pub fn train_on(&mut self, texts: &[String], epochs: i64, lr: f64, max_len: usize) {
        if texts.is_empty() { return; }
        let mut opt = nn::Adam::default().build(&self.vs, lr).unwrap();
        let xs: Vec<Tensor> = texts.iter().map(|t| {
            let ids = ids_from_text(t, max_len);
            Tensor::of_slice(&ids).view([1, ids.len() as i64])
        }).collect();
        let ys: Vec<i64> = texts.iter().map(|t| string_to_class(t)).collect();
        let y = Tensor::of_slice(&ys).to_kind(Kind::Int64);
        for _ in 0..epochs {
            let mut losses = Vec::new();
            for (i, x_ids) in xs.iter().enumerate() {
                let seq_len = x_ids.size()[1];
                let pos_t = Tensor::arange(seq_len, (Kind::Int64, Device::Cpu)).view([1, seq_len]);
                let x = self.token_emb.forward(x_ids) + self.pos_emb.forward(&pos_t);
                let y1 = self.ln1.forward(&(x.copy() + self.mha(&x)));
                let y2 = self.ff2.forward(&self.ff1.forward(&y1).relu());
                let z = self.ln2.forward(&(y1 + y2));
                let pooled = z.mean_dim([1].as_ref(), false, Kind::Float);
                let logits = self.clamp_sacred(&self.head.forward(&pooled));
                let loss = logits.cross_entropy_for_logits(&y.i(i as i64));
                opt.zero_grad();
                loss.backward();
                opt.step();
                losses.push(loss);
            }
        }
    }
}
