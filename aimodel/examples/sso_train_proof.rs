//! End-to-end proof that the training loop closes.
//!
//! Before this example, `.backward()` appeared **zero times** in the entire
//! crate. Every "training" entry point either computed a loss and discarded it
//! (`JEPATrainer::train_step`), applied a hand-derived update missing its input
//! term (`CALMEngine::train_contrastive`), or was never called at all
//! (`QuantumJEPAOptimizer::train`, `BurnSSO::update_2d`). The SSO/Muon
//! optimizer was correct code with no model attached.
//!
//! This wires the real path:
//!
//!   BurnCALM (Linear layers)
//!     -> forward
//!     -> MSE reconstruction loss
//!     -> loss.backward()                  <- real autodiff gradients
//!     -> GradientsParams::from_grads
//!     -> SSO step (spectral retraction + Newton-Schulz msign + momentum)
//!     -> updated weights
//!
//! ## Why this task
//! The target is a rank-3 structured signal in a 32-dim space: every sample is
//! a random combination of three fixed basis vectors plus noise. A model that
//! is genuinely learning discovers the 3-dim subspace and drives reconstruction
//! error well below the noise floor of the untrained baseline. A model that is
//! *not* learning stays flat. The gap is unambiguous, which is the point — the
//! failure mode being ruled out here is "looks like training, changes nothing".
//!
//! Run with:
//! ```bash
//! cargo run --release --example sso_train_proof
//! ```

use burn::backend::{Autodiff, NdArray};
use burn::module::AutodiffModule;
use burn::optim::{GradientsParams, Optimizer};
use burn::tensor::{Tensor, TensorData};

use vortex::ml::burn_calm::{BurnCALM, BurnCALMConfig};
use vortex::ml::training::SsoConfig;

type B = Autodiff<NdArray>;

const INPUT_DIM: usize = 32;
const RANK: usize = 3;
const BATCH: usize = 64;
const STEPS: usize = 1200;
const LR: f64 = 5e-3;
/// Each step draws a fresh random batch, so single-step losses are noisy
/// (step 25 can read higher than step 1). Compare windowed means instead.
const WINDOW: usize = 50;

/// Deterministic LCG so runs are reproducible without an rng dependency.
struct Lcg(u64);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((self.0 >> 33) as f32) / (u32::MAX as f32)) * 2.0 - 1.0
    }
}

/// Fixed rank-3 basis: the structure the model must discover.
fn basis() -> Vec<Vec<f32>> {
    let mut rng = Lcg(0xC0FFEE);
    (0..RANK)
        .map(|_| {
            let v: Vec<f32> = (0..INPUT_DIM).map(|_| rng.next_f32()).collect();
            let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
            v.into_iter().map(|x| x / norm).collect()
        })
        .collect()
}

/// One batch of rank-3 samples + small noise.
fn batch(rng: &mut Lcg, basis: &[Vec<f32>], device: &<B as burn::tensor::backend::Backend>::Device) -> Tensor<B, 2> {
    let mut flat = Vec::with_capacity(BATCH * INPUT_DIM);
    for _ in 0..BATCH {
        let coeffs: Vec<f32> = (0..RANK).map(|_| rng.next_f32()).collect();
        for d in 0..INPUT_DIM {
            let mut v = 0.0;
            for r in 0..RANK {
                v += coeffs[r] * basis[r][d];
            }
            flat.push(v + rng.next_f32() * 0.02);
        }
    }
    Tensor::<B, 2>::from_data(TensorData::new(flat, [BATCH, INPUT_DIM]), device)
}

fn main() {
    let device = Default::default();
    let basis = basis();
    let mut rng = Lcg(0x5EED);

    // `input_dim` defaults to 72 (8 tokens x 9 digits, the CALM beam layout)
    // and has no builder, so set it directly to match our sample width.
    let mut config = BurnCALMConfig::new()
        .with_latent_dim(RANK * 2)
        .with_hidden_dim(64);
    config.input_dim = INPUT_DIM;
    let mut model: BurnCALM<B> = BurnCALM::new(config, &device);

    let mut optim = SsoConfig::new()
        .with_momentum(0.9)
        .with_newton_schulz_iters(5)
        .init::<B, BurnCALM<B>>();

    // Baseline: reconstruction error before any weight has moved.
    let probe = batch(&mut rng, &basis, &device);
    let baseline: f32 = model.reconstruction_loss(probe).into_scalar();

    println!("SSO / Muon end-to-end training proof");
    println!("  task      : reconstruct a rank-{RANK} subspace of R^{INPUT_DIM}");
    println!("  optimizer : SSO (spectral retraction + Newton-Schulz msign + momentum)");
    println!("  backend   : Autodiff<NdArray>");
    println!("  baseline (untrained) MSE = {baseline:.6}\n");

    let mut history: Vec<f32> = Vec::with_capacity(STEPS);

    for step in 1..=STEPS {
        let x = batch(&mut rng, &basis, &device);

        let loss = model.reconstruction_loss(x);
        let loss_val: f32 = loss.clone().into_scalar();

        // The line that did not previously exist anywhere in this crate.
        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &model);
        model = optim.step(LR, model, grads);

        history.push(loss_val);

        if step % 100 == 0 || step == 1 {
            println!("  step {step:>4}   MSE = {loss_val:.6}");
        }
    }

    let mean = |xs: &[f32]| xs.iter().sum::<f32>() / xs.len() as f32;
    let first = mean(&history[..WINDOW]);
    let last = mean(&history[history.len() - WINDOW..]);
    let improvement = 100.0 * (1.0 - last / first);

    println!("\n  mean MSE, first {WINDOW} steps = {first:.6}");
    println!("  mean MSE, last  {WINDOW} steps = {last:.6}");
    println!("  reduction                   = {improvement:.1}%");

    // A loop that does not learn produces a flat curve within batch noise.
    // Require the windowed means to separate by a clear margin.
    let learned = last < first * 0.6;
    println!(
        "\n  RESULT: {}",
        if learned {
            "PASS — gradients reached the weights and the model learned"
        } else {
            "FAIL — loss did not fall; the loop is not closed"
        }
    );
    if !learned {
        std::process::exit(1);
    }
}
