use murmur3::murmur3_32;
use std::io::Cursor;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device, Kind, Tensor};

const FEAT_DIM: i64 = 64;

fn reduce_digits(mut n: u64) -> u8 {
    if n == 0 {
        return 9;
    }

pub fn featurize_ids(ids: &[u32]) -> Tensor {
    let mut v = vec![0f32; FEAT_DIM as usize];
    if ids.is_empty() {
        return Tensor::of_slice(&v).view([1, FEAT_DIM]);
    }
    for id in ids {
        let idx = (*id as usize) % FEAT_DIM as usize;
        v[idx] += 1.0;
    }
    let len = ids.len() as f32;
    for x in v.iter_mut() { *x /= len; }
    Tensor::of_slice(&v).view([1, FEAT_DIM])
}
    while n > 9 {
        let mut sum = 0u64;
        while n > 0 {
            sum += n % 10;
            n /= 10;
        }
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

pub fn featurize_row(s: &str) -> Tensor {
    let mut v = vec![0f32; FEAT_DIM as usize];
    let bytes = s.as_bytes();
    for b in bytes {
        let idx = (*b as usize) % FEAT_DIM as usize;
        v[idx] += 1.0;
    }
    let len = (bytes.len().max(1)) as f32;
    for x in v.iter_mut() {
        *x /= len;
    }
    Tensor::of_slice(&v).view([1, FEAT_DIM])
}

pub struct VortexNet {
    vs: nn::VarStore,
    seq: nn::Sequential,
}

impl VortexNet {
    pub fn new(in_dim: i64, hidden: i64, out_dim: i64) -> Self {
        let vs = nn::VarStore::new(Device::Cpu);
        let root = &vs.root();
        let seq = nn::seq()
            .add(nn::linear(root / "lin1", in_dim, hidden, Default::default()))
            .add_fn(|x| x.relu())
            .add(nn::linear(root / "lin2", hidden, out_dim, Default::default()));
        Self { vs, seq }
    }

    pub fn forward(&self, x: &Tensor) -> Tensor {
        self.seq.forward(x)
    }

    pub fn predict_logits(&self, s: &str) -> Tensor {
        let x = featurize_row(s);
        self.forward(&x)
    }

    pub fn predict_distribution(&self, s: &str) -> Tensor {
        let logits = self.predict_logits(s);
        logits.softmax(-1, Kind::Float)
    }

    pub fn predict_digit(&self, s: &str) -> i32 {
        let logits = self.predict_logits(s);
        let idx = logits.argmax(-1, false).int64_value(&[0]) as i32;
        idx + 1
    }

    pub fn predict_logits_from_ids(&self, ids: &[u32]) -> Tensor {
        let x = featurize_ids(ids);
        self.forward(&x)
    }

    pub fn predict_digit_from_ids(&self, ids: &[u32]) -> i32 {
        let logits = self.predict_logits_from_ids(ids);
        let idx = logits.argmax(-1, false).int64_value(&[0]) as i32;
        idx + 1
    }

    pub fn train_on(&mut self, texts: &[String], epochs: i64, lr: f64) {
        let mut opt = nn::Sgd::default().build(&self.vs, lr).unwrap();
        if texts.is_empty() {
            return;
        }
        let xs: Vec<Tensor> = texts.iter().map(|t| featurize_row(t)).collect();
        let ys: Vec<i64> = texts.iter().map(|t| string_to_class(t)).collect();
        let x = Tensor::cat(&xs, 0);
        let y = Tensor::of_slice(&ys).to_kind(Kind::Int64);
        for _ in 0..epochs {
            let logits = self.forward(&x);
            let loss = logits.cross_entropy_for_logits(&y);
            opt.backward_step(&loss);
        }
    }
}
