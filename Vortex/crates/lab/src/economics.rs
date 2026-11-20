//! # 015 | Vortex Economics
//!
//! What one request costs to serve, and what to charge so the system pays
//! for itself, its research, and its legal and compliance obligations, with a
//! margin left over.
//!
//! **Cost** comes from *measured* per-request compute (harness latency on a
//! known core count), priced at a cloud instance's on-demand rate and
//! divided by realistic utilisation, plus an infrastructure overhead.
//!
//! **Price** is set so that, out of every dollar of revenue, fixed shares go
//! to payment processing, R&D, legal/compliance and operating margin, and
//! the remainder covers cost:
//!
//! `price = cost / (1 - payment - rnd - legal - margin)`
//!
//! **Fixed costs** (an always-on serving instance, monitoring, storage) do
//! not vary per request; the report states the monthly volume at which the
//! variable margin covers them (break-even).
//!
//! Every number here is either measured (latency) or an explicit, sourced
//! assumption in [`Assumptions`]; change the assumptions, not the code.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub usd_per_hour: f64,
    pub vcpus: u32,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assumptions {
    pub serving: Instance,
    /// Average fraction of paid capacity doing useful work.
    pub utilization: f64,
    /// Storage, egress, monitoring, orchestration, as a fraction of compute.
    pub infra_overhead: f64,
    /// Card processing: 2.9% + $0.30 on an average invoice.
    pub payment_rate: f64,
    pub payment_fixed_usd: f64,
    pub average_invoice_usd: f64,
    /// Revenue shares reserved before margin.
    pub rnd_share: f64,
    pub legal_share: f64,
    /// Operating margin target.
    pub margin_share: f64,
    /// Always-on fixed cost per month (one serving instance + extras).
    pub fixed_monthly_usd: f64,
}

impl Default for Assumptions {
    fn default() -> Self {
        let serving = Instance {
            name: "AWS c7i.2xlarge (8 vCPU), on-demand, us-east-1".into(),
            usd_per_hour: 0.357,
            vcpus: 8,
            source: "calculator.holori.com/aws/ec2/c7i.2xlarge, updated 2026-08-30".into(),
        };
        // One instance always on (730 h) + 25% for monitoring, storage, backups.
        let fixed = serving.usd_per_hour * 730.0 * 1.25;
        Self {
            serving,
            utilization: 0.35,
            infra_overhead: 0.25,
            payment_rate: 0.029,
            payment_fixed_usd: 0.30,
            average_invoice_usd: 20.0,
            rnd_share: 0.20,
            legal_share: 0.08,
            margin_share: 0.30,
            fixed_monthly_usd: fixed,
        }
    }
}

impl Assumptions {
    /// Payment processing as a share of revenue.
    pub fn payment_share(&self) -> f64 {
        self.payment_rate + self.payment_fixed_usd / self.average_invoice_usd.max(0.01)
    }

    /// Share of revenue left to cover per-request cost.
    pub fn cost_share(&self) -> f64 {
        1.0 - self.payment_share() - self.rnd_share - self.legal_share - self.margin_share
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workload {
    pub class: String,
    /// Measured single-core milliseconds per request.
    pub cpu_ms: f64,
    /// Where the measurement came from.
    pub measured_by: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PriceLine {
    pub class: String,
    pub cpu_ms: f64,
    pub cost_per_1k_usd: f64,
    pub price_per_1k_usd: f64,
    /// Price floor: never charge less than this per 1k, so tiny compute costs
    /// still fund fixed costs at realistic volumes.
    pub list_price_per_1k_usd: f64,
    pub gross_margin_at_list: f64,
    /// Requests per month at list price for variable margin to cover fixed cost.
    pub breakeven_requests_per_month: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CostReport {
    pub assumptions: Assumptions,
    pub cost_share: f64,
    pub markup: f64,
    pub lines: Vec<PriceLine>,
}

/// Minimum list price per 1k requests.
pub const LIST_FLOOR_PER_1K: f64 = 0.10;

pub fn cost_per_request(a: &Assumptions, cpu_ms: f64) -> f64 {
    let usd_per_core_ms = a.serving.usd_per_hour / a.serving.vcpus as f64 / 3_600_000.0;
    cpu_ms * usd_per_core_ms / a.utilization.max(0.01) * (1.0 + a.infra_overhead)
}

pub fn price(a: &Assumptions, workloads: &[Workload]) -> CostReport {
    let share = a.cost_share();
    assert!(share > 0.0, "revenue shares exceed 100%");
    let markup = 1.0 / share;
    let lines = workloads
        .iter()
        .map(|w| {
            let cost = cost_per_request(a, w.cpu_ms) * 1000.0;
            let price = cost * markup;
            // Round list price up to the next cent, with a floor.
            let list = ((price * 100.0).ceil() / 100.0).max(LIST_FLOOR_PER_1K);
            let variable_margin_per_1k = list * share - cost;
            let breakeven = if variable_margin_per_1k > 0.0 { a.fixed_monthly_usd / variable_margin_per_1k * 1000.0 } else { f64::INFINITY };
            PriceLine {
                class: w.class.clone(),
                cpu_ms: w.cpu_ms,
                cost_per_1k_usd: cost,
                price_per_1k_usd: price,
                list_price_per_1k_usd: list,
                gross_margin_at_list: 1.0 - cost / list,
                breakeven_requests_per_month: breakeven,
            }
        })
        .collect();
    CostReport { assumptions: a.clone(), cost_share: share, markup, lines }
}

impl CostReport {
    pub fn table(&self) -> String {
        let mut s = format!(
            "Serving: {} at ${:.3}/h; utilisation {:.0}%, infra overhead {:.0}%\n\
             Revenue split: payment {:.1}%, R&D {:.0}%, legal/compliance {:.0}%, margin {:.0}%, cost {:.1}%  (markup x{:.2})\n\
             Fixed: ${:.0}/month always-on\n\n",
            self.assumptions.serving.name,
            self.assumptions.serving.usd_per_hour,
            self.assumptions.utilization * 100.0,
            self.assumptions.infra_overhead * 100.0,
            self.assumptions.payment_share() * 100.0,
            self.assumptions.rnd_share * 100.0,
            self.assumptions.legal_share * 100.0,
            self.assumptions.margin_share * 100.0,
            self.cost_share * 100.0,
            self.markup,
            self.assumptions.fixed_monthly_usd,
        );
        s.push_str(&format!("{:<22} {:>10} {:>14} {:>14} {:>12} {:>10} {:>16}\n", "workload", "cpu ms", "cost/1k", "cost-based/1k", "list/1k", "gm@list", "break-even/mo"));
        for l in &self.lines {
            s.push_str(&format!(
                "{:<22} {:>10.3} {:>14.6} {:>14.6} {:>12.2} {:>9.1}% {:>16.0}\n",
                l.class, l.cpu_ms, l.cost_per_1k_usd, l.price_per_1k_usd, l.list_price_per_1k_usd, l.gross_margin_at_list * 100.0, l.breakeven_requests_per_month
            ));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_covers_every_share() {
        let a = Assumptions::default();
        let r = price(&a, &[Workload { class: "x".into(), cpu_ms: 100.0, measured_by: "test".into() }]);
        let l = &r.lines[0];
        // Revenue at the cost-based price splits exactly into the shares.
        let revenue = l.price_per_1k_usd;
        let parts = revenue * (a.payment_share() + a.rnd_share + a.legal_share + a.margin_share) + l.cost_per_1k_usd;
        assert!((parts - revenue).abs() < 1e-9);
        assert!(l.list_price_per_1k_usd >= l.price_per_1k_usd);
        // 100 ms on c7i.2xlarge at 35% utilisation: ~0.00000443 USD each.
        assert!((cost_per_request(&a, 100.0) - 0.357 / 8.0 / 3_600_000.0 * 100.0 / 0.35 * 1.25).abs() < 1e-15);
    }
}
