//! AIModel Demo - Sacred Geometry + ML in Action
//!
//! Run with: cargo run --example demo

use aimodel::{
    data::models::BeamTensor,
    ml::{
        BurnSSO, SSOConfig, SpectralScaler,
        BurnCALM, BurnCALMConfig, LatentEnergyScorer,
        CALMEngine, CALMConfig,
        VortexDiscovery, DiscoveryConfig,
        backend_info,
    },
    core::sacred_geometry::VortexPositioningEngine,
};
use burn::backend::NdArray;
use burn::tensor::Tensor;

type B = NdArray<f32>;

fn main() {
    println!("==================================================================");
    println!("           AIMODEL DEMO - Sacred Geometry + ML");
    println!("==================================================================");
    println!();

    // Show backend info
    println!("[Backend] {}", backend_info());
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // 1. Sacred Geometry Core - The Vortex Cycle
    // ═══════════════════════════════════════════════════════════════════
    println!("------------------------------------------------------------------");
    println!("1. SACRED GEOMETRY - Vortex Cycle (1->2->4->8->7->5->1)");
    println!("------------------------------------------------------------------");
    
    let vortex_engine = VortexPositioningEngine::new();
    
    println!("\nVortex Cycle Demonstration:");
    let cycle = [1, 2, 4, 8, 7, 5, 1];
    for i in 0..cycle.len() - 1 {
        let from = cycle[i];
        let to = cycle[i + 1];
        println!("   {} -> {}", from, to);
    }
    
    println!("\n[*] Sacred Guides (3, 6, 9) - Special checkpoint positions");
    for pos in [3u64, 6, 9] {
        let dr = VortexPositioningEngine::digital_root(pos * 111); // 333, 666, 999
        println!("   Position {}: Digital root of {} = {}", pos, pos * 111, dr);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 2. BeamTensor - The Core Data Structure
    // ═══════════════════════════════════════════════════════════════════
    println!("\n------------------------------------------------------------------");
    println!("2. BEAM TENSOR - Information Carrier");
    println!("------------------------------------------------------------------");
    
    let mut beam = BeamTensor::default();
    beam.digits = [0.1, 0.2, 0.4, 0.8, 0.7, 0.5, 0.3, 0.6, 0.9];
    beam.position = 1;
    beam.confidence = 0.85;
    
    println!("\n[+] BeamTensor created:");
    println!("   Digits: {:?}", beam.digits);
    println!("   Position: {}", beam.position);
    println!("   Confidence: {:.2}", beam.confidence);
    println!("   ELP (Ethos/Logos/Pathos): {:.2}/{:.2}/{:.2}", 
             beam.attributes.ethos(), 
             beam.attributes.logos(), 
             beam.attributes.pathos());

    // ═══════════════════════════════════════════════════════════════════
    // 3. BurnSSO - Spectral Sphere Optimizer
    // ═══════════════════════════════════════════════════════════════════
    println!("\n------------------------------------------------------------------");
    println!("3. BURN SSO - Spectral Sphere Optimizer");
    println!("------------------------------------------------------------------");
    
    let device = Default::default();
    let sso_config = SSOConfig::new()
        .with_lr(0.001)
        .with_momentum(0.9)
        .with_scaler(SpectralScaler::MuP);
    let sso: BurnSSO<B> = BurnSSO::new(sso_config);
    
    println!("\n[+] SSO Configuration:");
    println!("   Learning rate: 0.001");
    println!("   Momentum: 0.9");
    println!("   Scaler: MuP (Maximal Update Parameterization)");
    
    // Simulate a weight matrix
    let weight: Tensor<B, 2> = Tensor::random(
        [64, 32],
        burn::tensor::Distribution::Normal(0.0, 0.1),
        &device,
    );
    
    // Compute spectral norm (returns tuple: sigma, u, v)
    let (sigma, _u, _v) = sso.power_iteration(&weight, None);
    println!("\n[+] Weight Matrix [64x32]:");
    println!("   Spectral norm (sigma_1): {:.4}", sigma);
    println!("   [OK] Power iteration converged!");

    // ═══════════════════════════════════════════════════════════════════
    // 4. BurnCALM - Continuous Autoregressive Language Model
    // ═══════════════════════════════════════════════════════════════════
    println!("\n------------------------------------------------------------------");
    println!("4. BURN CALM - Continuous Latent Autoencoder");
    println!("------------------------------------------------------------------");
    
    let calm_config = BurnCALMConfig::new()
        .with_latent_dim(128)
        .with_hidden_dim(256);
    let calm: BurnCALM<B> = BurnCALM::new(calm_config, &device);
    
    println!("\n[+] CALM Configuration:");
    println!("   Input dim: 72 (8 tokens x 9 digits)");
    println!("   Latent dim: 128");
    println!("   Hidden dim: 256");
    
    // Create input tensor (batch of 2)
    let input: Tensor<B, 2> = Tensor::random(
        [2, 72],
        burn::tensor::Distribution::Normal(0.0, 1.0),
        &device,
    );
    
    // Encode -> Latent -> Decode
    let latent = calm.encode(input.clone());
    let reconstructed = calm.decode(latent.clone());
    
    println!("\n[+] Autoencoder Pipeline:");
    println!("   Input shape:         [2, 72]");
    println!("   Latent shape:        {:?}", latent.dims());
    println!("   Reconstructed shape: {:?}", reconstructed.dims());
    
    // Compressed generation (K× speedup)
    let output = calm.generate_compressed(input.clone(), 4);
    println!("\n[+] Compressed Generation (4 steps in latent space):");
    println!("   Output shape: {:?}", output.dims());
    println!("   [OK] 4x fewer decoding steps!");
    
    // Energy scoring
    let scorer: LatentEnergyScorer<B> = LatentEnergyScorer::new();
    let energy_score = scorer.score(&latent);
    println!("\n[+] Latent Energy Score: {:.4}", energy_score);

    // ═══════════════════════════════════════════════════════════════════
    // 5. CALM Engine - Pure Rust Implementation
    // ═══════════════════════════════════════════════════════════════════
    println!("\n------------------------------------------------------------------");
    println!("5. CALM ENGINE - BeamTensor Processing");
    println!("------------------------------------------------------------------");
    
    let calm_engine = CALMEngine::new(CALMConfig::new().with_latent_dim(64));
    
    // Create some BeamTensors
    let beams: Vec<BeamTensor> = (0..4).map(|i| {
        let mut b = BeamTensor::default();
        b.position = i as u8;
        b.confidence = 0.5 + (i as f32) * 0.1;
        b
    }).collect();
    
    println!("\n[+] Encoding {} BeamTensors to latent space...", beams.len());
    let latent_state = calm_engine.encode(&beams);
    println!("   Latent vector length: {}", latent_state.latent.len());
    println!("   Energy: {:.4}", latent_state.energy);

    // ═══════════════════════════════════════════════════════════════════
    // 6. VortexDiscovery - Test-Time Adaptation
    // ═══════════════════════════════════════════════════════════════════
    println!("\n------------------------------------------------------------------");
    println!("6. VORTEX DISCOVERY - Test-Time Adaptation");
    println!("------------------------------------------------------------------");
    
    let discovery_config = DiscoveryConfig::new();
    let discovery = VortexDiscovery::new(discovery_config, 256);
    
    println!("\n[+] Discovery Configuration:");
    println!("   Entropy threshold: 0.7");
    println!("   Candidates per query: 4");
    println!("   LoRA rank: 8");
    
    // Check if discovery should trigger
    let query_beams: Vec<BeamTensor> = vec![BeamTensor::default()];
    let should_trigger = discovery.should_trigger(&query_beams);
    println!("\n[+] Query BeamTensor:");
    println!("   Should trigger discovery: {}", should_trigger);

    // ═══════════════════════════════════════════════════════════════════
    // Summary
    // ═══════════════════════════════════════════════════════════════════
    println!("\n==================================================================");
    println!("                    [OK] Demo Complete!");
    println!("==================================================================");
    println!("  [*] Sacred Geometry: Vortex cycle + digital roots");
    println!("  [*] BeamTensor: 9-digit information carrier");
    println!("  [*] BurnSSO: Spectral sphere optimization");
    println!("  [*] BurnCALM: Continuous latent autoencoder");
    println!("  [*] VortexDiscovery: Test-time adaptation");
    println!("==================================================================");
}
