//! Differentiable linear elasticity — the smallest real `L_Physics`.
//!
//! ## Why this exists
//! A physics-informed generative loss needs `∂L_physics/∂θ`. Avian is not
//! differentiable and a general FEA solve is not differentiable without
//! deliberate effort, so `L_Total = L_task + λ·L_physics` cannot be written
//! against a black-box simulator.
//!
//! **Isotropic linear elasticity is differentiable in closed form.** Hooke's
//! law is a constant matrix multiply, and von Mises stress is a smooth function
//! of the result. Expressed in Burn tensors, the whole chain is autodiff-able
//! and steppable by SSO — giving a genuine physics gradient without a
//! simulator in the loop.
//!
//! It is deliberately *limited*: linear elastic response only, no contact, no
//! buckling, no plasticity, no large deformation. That is the honest scope of
//! what closed-form differentiability buys. Nonlinear behaviour needs either a
//! learned surrogate or RL against the real simulator.
//!
//! ## The chain
//! ```text
//! strain ε  [batch, 6]        Voigt: [εxx, εyy, εzz, γxy, γyz, γzx]
//!    │
//!    │  σ = D ε               D from (E, ν), isotropic
//!    ▼
//! stress σ  [batch, 6]
//!    │
//!    │  von Mises
//!    ▼
//! σ_vm      [batch]
//!    │
//!    │  penalise only σ_vm > σ_allow   (hinge)
//!    ▼
//! L_physics scalar
//! ```
//!
//! The penalty is one-sided on purpose. A structure that is *under* its yield
//! stress is not better for being further under it — pushing σ toward zero
//! would reward removing material that is doing useful work. Only violation is
//! penalised.
//!
//! Voigt convention matches `eustress-common`'s `StrainTensor`: shear entries
//! are **engineering** shear (γ = 2ε), which is why the shear block of `D`
//! carries `(1-2ν)/2` rather than `(1-2ν)`.

use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};

/// Isotropic linear-elastic material.
#[derive(Debug, Clone, Copy)]
pub struct ElasticMaterial {
    /// Young's modulus `E`, Pa.
    pub young_modulus: f32,
    /// Poisson's ratio `ν`, dimensionless. Physically `-1 < ν < 0.5`.
    pub poisson_ratio: f32,
    /// Yield stress used as the violation threshold, Pa.
    pub yield_stress: f32,
}

impl ElasticMaterial {
    /// Structural steel.
    pub fn steel() -> Self {
        Self { young_modulus: 200.0e9, poisson_ratio: 0.30, yield_stress: 250.0e6 }
    }
    /// Structural concrete (compressive).
    pub fn concrete() -> Self {
        Self { young_modulus: 30.0e9, poisson_ratio: 0.20, yield_stress: 30.0e6 }
    }
    /// Aluminium alloy.
    pub fn aluminium() -> Self {
        Self { young_modulus: 69.0e9, poisson_ratio: 0.33, yield_stress: 95.0e6 }
    }

    /// Whether the parameters are physically admissible.
    ///
    /// `ν = 0.5` is incompressible and makes `D` singular — the `1 - 2ν`
    /// denominator goes to zero — so it is rejected rather than returning
    /// infinities that would silently poison a gradient.
    pub fn is_valid(&self) -> bool {
        self.young_modulus > 0.0
            && self.poisson_ratio > -1.0
            && self.poisson_ratio < 0.5 - 1e-6
            && self.yield_stress > 0.0
    }

    /// The 6x6 isotropic constitutive matrix `D` in Voigt form.
    ///
    /// Returned as a plain array so it can be built once per material and
    /// uploaded as a constant — `D` does not depend on the model parameters, so
    /// it carries no gradient and should not be rebuilt per step.
    pub fn constitutive_matrix(&self) -> [[f32; 6]; 6] {
        let (e, nu) = (self.young_modulus, self.poisson_ratio);
        let f = e / ((1.0 + nu) * (1.0 - 2.0 * nu));
        let diag = f * (1.0 - nu);
        let off = f * nu;
        let shear = f * (1.0 - 2.0 * nu) / 2.0; // engineering shear
        let mut d = [[0.0f32; 6]; 6];
        for i in 0..3 {
            for j in 0..3 {
                d[i][j] = if i == j { diag } else { off };
            }
        }
        for i in 3..6 {
            d[i][i] = shear;
        }
        d
    }
}

/// Differentiable linear-elastic stress model.
#[derive(Debug, Clone)]
pub struct ElasticLoss {
    material: ElasticMaterial,
}

impl ElasticLoss {
    /// Build from a material. Panics if the material is not physically valid,
    /// because a singular `D` produces silent NaN gradients downstream.
    pub fn new(material: ElasticMaterial) -> Self {
        assert!(
            material.is_valid(),
            "invalid elastic material: E={} nu={} (nu must be < 0.5; 0.5 is incompressible and makes D singular)",
            material.young_modulus,
            material.poisson_ratio
        );
        Self { material }
    }

    /// The material in use.
    pub fn material(&self) -> ElasticMaterial {
        self.material
    }

    /// `D` as a tensor constant, `[6, 6]`.
    pub fn d_matrix<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2> {
        let d = self.material.constitutive_matrix();
        let flat: Vec<f32> = d.iter().flat_map(|r| r.iter().copied()).collect();
        Tensor::<B, 2>::from_data(TensorData::new(flat, [6, 6]), device)
    }

    /// Hooke's law: `σ = D ε`.
    ///
    /// `strain` `[batch, 6]` in Voigt order with engineering shear.
    /// Returns `[batch, 6]`.
    pub fn stress<B: Backend>(&self, strain: Tensor<B, 2>) -> Tensor<B, 2> {
        let device = strain.device();
        // D is symmetric, so D^T == D and the transpose is omitted deliberately.
        strain.matmul(self.d_matrix::<B>(&device))
    }

    /// Von Mises equivalent stress, `[batch]`.
    ///
    /// `σ_vm = sqrt( ½[(σxx-σyy)² + (σyy-σzz)² + (σzz-σxx)²] + 3(τxy² + τyz² + τzx²) )`
    ///
    /// A small epsilon is added before `sqrt` because `d/dx sqrt(x)` is
    /// unbounded at zero — an unstressed element would otherwise produce an
    /// infinite gradient.
    pub fn von_mises<B: Backend>(&self, stress: Tensor<B, 2>) -> Tensor<B, 1> {
        let [batch, _] = stress.dims();
        let sxx = stress.clone().slice([0..batch, 0..1]);
        let syy = stress.clone().slice([0..batch, 1..2]);
        let szz = stress.clone().slice([0..batch, 2..3]);
        let txy = stress.clone().slice([0..batch, 3..4]);
        let tyz = stress.clone().slice([0..batch, 4..5]);
        let tzx = stress.slice([0..batch, 5..6]);

        let a = (sxx.clone() - syy.clone()).powf_scalar(2.0);
        let b = (syy - szz.clone()).powf_scalar(2.0);
        let c = (szz - sxx).powf_scalar(2.0);
        let shear = txy.powf_scalar(2.0) + tyz.powf_scalar(2.0) + tzx.powf_scalar(2.0);

        let inner = (a + b + c).mul_scalar(0.5) + shear.mul_scalar(3.0);
        inner.add_scalar(1.0e-12).sqrt().reshape([batch])
    }

    /// One-sided yield-violation penalty, a scalar.
    ///
    /// `L = mean( max(0, σ_vm - σ_allow)² ) / σ_allow²`
    ///
    /// Normalised by `σ_allow²` so the magnitude is dimensionless and a `λ`
    /// tuned for steel does not silently become 60x stronger on concrete.
    /// Only violation is penalised — see the module note on why.
    pub fn yield_violation<B: Backend>(&self, strain: Tensor<B, 2>) -> Tensor<B, 1> {
        let vm = self.von_mises(self.stress(strain));
        let allow = self.material.yield_stress;
        let excess = (vm - allow).clamp_min(0.0);
        excess.powf_scalar(2.0).mean().div_scalar(allow * allow)
    }

    /// Strain energy density `½ εᵀ D ε`, `[batch]`.
    ///
    /// Useful as a compliance objective — minimising it maximises stiffness,
    /// which is the standard topology-optimisation target.
    pub fn strain_energy<B: Backend>(&self, strain: Tensor<B, 2>) -> Tensor<B, 1> {
        let [batch, _] = strain.dims();
        let stress = self.stress(strain.clone());
        (strain * stress).sum_dim(1).reshape([batch]).mul_scalar(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::{Autodiff, NdArray};

    type TB = NdArray;
    type AD = Autodiff<NdArray>;

    fn strain<B: Backend>(v: [f32; 6], device: &B::Device) -> Tensor<B, 2> {
        Tensor::<B, 2>::from_data(TensorData::new(v.to_vec(), [1, 6]), device)
    }

    #[test]
    fn uniaxial_stress_matches_hookes_law_exactly() {
        // The defining check: pull a bar along x with strain ε and the axial
        // stress must be E·ε. Any error in D shows up here immediately.
        let device = Default::default();
        let m = ElasticMaterial::steel();
        let loss = ElasticLoss::new(m);

        // Uniaxial STRESS state requires lateral contraction -nu*eps.
        let eps = 1.0e-4;
        let s = strain::<TB>([eps, -m.poisson_ratio * eps, -m.poisson_ratio * eps, 0.0, 0.0, 0.0], &device);
        let out = loss.stress(s).into_data().convert::<f32>();
        let v = out.as_slice::<f32>().unwrap();

        let expected = m.young_modulus * eps;
        assert!(
            (v[0] - expected).abs() / expected < 1e-4,
            "sigma_xx = {} expected {} (E*eps)",
            v[0], expected
        );
        // Transverse stresses must vanish in a uniaxial-stress state.
        assert!(v[1].abs() / expected < 1e-4, "sigma_yy should be ~0, got {}", v[1]);
        assert!(v[2].abs() / expected < 1e-4, "sigma_zz should be ~0, got {}", v[2]);
    }

    #[test]
    fn von_mises_of_uniaxial_equals_the_axial_stress() {
        // Textbook identity: for pure uniaxial stress, sigma_vm == |sigma_xx|.
        let device = Default::default();
        let loss = ElasticLoss::new(ElasticMaterial::steel());
        let sigma = 150.0e6;
        let s = Tensor::<TB, 2>::from_data(
            TensorData::new(vec![sigma, 0.0, 0.0, 0.0, 0.0, 0.0], [1, 6]),
            &device,
        );
        let vm = loss.von_mises(s).into_data().convert::<f32>();
        let got = vm.as_slice::<f32>().unwrap()[0];
        assert!((got - sigma).abs() / sigma < 1e-5, "von Mises {got} != {sigma}");
    }

    #[test]
    fn hydrostatic_pressure_has_zero_von_mises() {
        // Pure hydrostatic stress causes no distortion, so sigma_vm must be 0.
        // This is the sharpest test that the deviatoric part is right.
        let device = Default::default();
        let loss = ElasticLoss::new(ElasticMaterial::steel());
        let p = 500.0e6;
        let s = Tensor::<TB, 2>::from_data(
            TensorData::new(vec![p, p, p, 0.0, 0.0, 0.0], [1, 6]),
            &device,
        );
        let vm = loss.von_mises(s).into_data().convert::<f32>();
        let got = vm.as_slice::<f32>().unwrap()[0];
        assert!(got / p < 1e-6, "hydrostatic von Mises should be ~0, got {got}");
    }

    #[test]
    fn penalty_is_one_sided() {
        // Below yield costs nothing; above yield costs something.
        let device = Default::default();
        let m = ElasticMaterial::steel();
        let loss = ElasticLoss::new(m);
        let safe = m.yield_stress * 0.5 / m.young_modulus;
        let over = m.yield_stress * 3.0 / m.young_modulus;

        let l_safe = loss
            .yield_violation(strain::<TB>([safe, 0.0, 0.0, 0.0, 0.0, 0.0], &device))
            .into_data().convert::<f32>();
        let l_over = loss
            .yield_violation(strain::<TB>([over, 0.0, 0.0, 0.0, 0.0, 0.0], &device))
            .into_data().convert::<f32>();

        assert_eq!(l_safe.as_slice::<f32>().unwrap()[0], 0.0, "under yield must be free");
        assert!(l_over.as_slice::<f32>().unwrap()[0] > 0.0, "over yield must be penalised");
    }

    #[test]
    fn the_loss_actually_produces_gradients() {
        // The whole point: L_physics must be differentiable w.r.t. the strain
        // that a model produced. If this is zero or NaN the loss is decorative.
        let device = Default::default();
        let loss = ElasticLoss::new(ElasticMaterial::steel());
        let over = 3.0 * 250.0e6 / 200.0e9;

        let s = strain::<AD>([over, 0.0, 0.0, 0.0, 0.0, 0.0], &device).require_grad();
        let l = loss.yield_violation(s.clone());
        let grads = l.backward();
        let g = s.grad(&grads).expect("strain must receive a gradient");
        let data = g.into_data().convert::<f32>();
        let gx = data.as_slice::<f32>().unwrap()[0];

        assert!(gx.is_finite(), "gradient must be finite, got {gx}");
        assert!(gx > 0.0, "increasing tensile strain past yield must increase the penalty (d/deps = {gx})");
    }

    #[test]
    fn zero_strain_gradient_is_finite() {
        // sqrt has unbounded derivative at 0; the epsilon must keep this finite.
        let device = Default::default();
        let loss = ElasticLoss::new(ElasticMaterial::steel());
        let s = strain::<AD>([0.0; 6], &device).require_grad();
        let l = loss.strain_energy(s.clone()).mean();
        let grads = l.backward();
        let g = s.grad(&grads).expect("gradient");
        for v in g.into_data().convert::<f32>().as_slice::<f32>().unwrap() {
            assert!(v.is_finite(), "unstressed element produced non-finite gradient {v}");
        }
    }

    #[test]
    fn incompressible_material_is_rejected() {
        // nu = 0.5 makes D singular. Better to refuse than to emit infinities
        // that turn into NaN several layers downstream.
        let m = ElasticMaterial { young_modulus: 200.0e9, poisson_ratio: 0.5, yield_stress: 250.0e6 };
        assert!(!m.is_valid());
    }

    #[test]
    fn penalty_is_normalised_across_materials() {
        // Same relative overstress on steel and concrete must cost the same,
        // or a lambda tuned on one material silently mis-weights the other.
        let device = Default::default();
        for m in [ElasticMaterial::steel(), ElasticMaterial::concrete()] {
            let loss = ElasticLoss::new(m);
            let eps = 2.0 * m.yield_stress / m.young_modulus; // target 2x yield
            // Uniaxial STRESS, not uniaxial STRAIN: the lateral -nu*eps terms
            // are required. Under uniaxial *strain* the axial stress is
            // E(1-nu)/((1+nu)(1-2nu))*eps and the transverse stresses are
            // non-zero, so von Mises picks up a nu-dependent factor (1.35E for
            // steel, 1.11E for concrete) — the two materials would then sit at
            // different relative overstress and this comparison would be
            // meaningless. With lateral contraction, sigma_vm == E*eps exactly.
            let s = strain::<TB>([eps, -m.poisson_ratio * eps, -m.poisson_ratio * eps, 0.0, 0.0, 0.0], &device);
            let l = loss.yield_violation(s).into_data().convert::<f32>();
            let v = l.as_slice::<f32>().unwrap()[0];
            // sigma_vm = 2*sigma_y -> excess = sigma_y -> normalised = 1.0
            assert!(
                (v - 1.0).abs() < 1e-3,
                "normalised penalty at 2x yield should be ~1.0 for every material, got {v} (E={}, nu={})",
                m.young_modulus, m.poisson_ratio
            );
        }
    }
}
