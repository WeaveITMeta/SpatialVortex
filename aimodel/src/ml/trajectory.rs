//! Object journeys — spatial-temporal memory you can question.
//!
//! An object moves through space over time. This module encodes that journey
//! into a form attention can retrieve from, so the system can answer *where was
//! it*, *when was it there*, and *what happened near this place* long after the
//! moment scrolled out of the recent window.
//!
//! ## Why a positional encoding and not raw coordinates
//! Raw `(x, y, z, t)` are useless as attention keys. Dot-product similarity
//! between two raw coordinate vectors is dominated by magnitude — a point at
//! `(1000, 0, 0)` looks "similar" to everything far from the origin, and two
//! adjacent points near the origin look almost identical to two adjacent points
//! a kilometre away. Neither is the relation we want.
//!
//! [`JourneyEncoder`] applies a multi-frequency sinusoidal encoding (the same
//! trick NeRF uses for coordinates). Each axis contributes
//! `sin(2^k · π · v)` and `cos(2^k · π · v)` across `bands` octaves. That gives:
//!
//! - **locality** — nearby positions produce nearby codes at high frequencies
//! - **separation** — distant positions decorrelate rather than collapsing
//! - **scale coverage** — coarse bands capture the region, fine bands the spot
//!
//! Time is encoded on its own bands, so a query can be *temporal* ("where at
//! t=0.4"), *spatial* ("what happened near here"), or both, simply by which
//! part of the code the query populates.
//!
//! ## How it pairs with context rings
//! Each waypoint becomes one [`crate::ml::context_ring::ContextEntry`]. The
//! open ring holds recent motion verbatim; sealed rings compress older legs of
//! the journey to one vector each. With
//! [`crate::ml::context_ring::ContextRingConfig::wide_16m`] that is 16.7M
//! waypoints addressable — a very long journey — through an 8,191-vector view.

use crate::ml::context_ring::{ContextEntry, ContextRingBuffer, ContextRingConfig};

/// A single observation of an object along its path.
#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    /// Normalized time in `[0, 1]` across the journey.
    pub t: f32,
    /// World position.
    pub pos: [f32; 3],
    /// What happened here, if anything ("picked up key", "entered room 3").
    pub event: Option<String>,
    /// Retrieval weight. Events should outrank ordinary motion.
    pub salience: f32,
}

impl Waypoint {
    /// Ordinary motion, neutral salience.
    pub fn new(t: f32, pos: [f32; 3]) -> Self {
        Self { t, pos, event: None, salience: 1.0 }
    }

    /// Mark something that happened here. Raises salience so it survives
    /// compression when this leg of the journey is sealed and merged.
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self.salience = 4.0;
        self
    }

    /// Override salience explicitly.
    pub fn with_salience(mut self, s: f32) -> Self {
        self.salience = s;
        self
    }
}

/// Turns waypoints and queries into comparable vectors.
#[derive(Debug, Clone)]
pub struct JourneyEncoder {
    /// Sinusoidal octaves per axis.
    pub bands: usize,
    /// World extent used to normalize positions into `[-1, 1]`.
    pub world_scale: f32,
}

impl Default for JourneyEncoder {
    fn default() -> Self {
        Self { bands: 8, world_scale: 512.0 }
    }
}

impl JourneyEncoder {
    /// New encoder.
    pub fn new(bands: usize, world_scale: f32) -> Self {
        Self { bands, world_scale: world_scale.max(1e-3) }
    }

    /// Width of the produced code: 3 spatial axes + time, 2 phases, `bands` octaves.
    pub fn dim(&self) -> usize {
        (3 + 1) * 2 * self.bands
    }

    /// Offset at which the time bands begin. Useful for masking a query to be
    /// purely spatial or purely temporal.
    pub fn time_offset(&self) -> usize {
        3 * 2 * self.bands
    }

    fn encode_scalar(&self, v: f32, out: &mut Vec<f32>) {
        for k in 0..self.bands {
            let f = (1u32 << k) as f32 * std::f32::consts::PI;
            out.push((v * f).sin());
            out.push((v * f).cos());
        }
    }

    /// Encode a waypoint into a retrievable code.
    pub fn encode(&self, w: &Waypoint) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.dim());
        for axis in 0..3 {
            self.encode_scalar(w.pos[axis] / self.world_scale, &mut out);
        }
        self.encode_scalar(w.t, &mut out);
        out
    }

    /// Query by time alone — "where was it at `t`?".
    ///
    /// Spatial bands are zeroed, so the match is driven entirely by time.
    pub fn query_at_time(&self, t: f32) -> Vec<f32> {
        let mut out = vec![0.0; self.time_offset()];
        self.encode_scalar(t, &mut out);
        out
    }

    /// Query by place alone — "what happened near `pos`?".
    ///
    /// Time bands are zeroed, so the match is driven entirely by position.
    pub fn query_at_place(&self, pos: [f32; 3]) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.dim());
        for axis in 0..3 {
            self.encode_scalar(pos[axis] / self.world_scale, &mut out);
        }
        out.resize(self.dim(), 0.0);
        out
    }
}

/// A journey held in ring-buffered memory, queryable across time.
pub struct JourneyMemory {
    encoder: JourneyEncoder,
    ring: ContextRingBuffer,
    /// Waypoints kept alongside the codes so a retrieved match can be resolved
    /// back to a concrete position and event.
    waypoints: Vec<Waypoint>,
}

impl JourneyMemory {
    /// Build memory sized for `config`, encoding with `encoder`.
    ///
    /// The ring's `dim` is forced to the encoder width so codes are never
    /// silently truncated.
    pub fn new(encoder: JourneyEncoder, config: ContextRingConfig) -> Self {
        let cfg = ContextRingConfig { dim: encoder.dim(), ..config };
        Self { encoder, ring: ContextRingBuffer::new(cfg), waypoints: Vec::new() }
    }

    /// Sized for a long journey: 16.7M addressable waypoints, ~8 MiB resident.
    pub fn wide(encoder: JourneyEncoder) -> Self {
        Self::new(encoder, ContextRingConfig::wide_16m())
    }

    /// Record one observation.
    pub fn observe(&mut self, w: Waypoint) {
        let code = self.encoder.encode(&w);
        let mut entry = ContextEntry::new(code).with_salience(w.salience);
        if let Some(ev) = &w.event {
            entry = entry.with_tag(ev.clone());
        }
        self.ring.push(entry);
        self.waypoints.push(w);
    }

    /// Every observation recorded so far.
    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    /// Number of observations.
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// True when nothing has been observed.
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// The encoder in use.
    pub fn encoder(&self) -> &JourneyEncoder {
        &self.encoder
    }

    /// The underlying ring buffer.
    pub fn ring(&self) -> &ContextRingBuffer {
        &self.ring
    }

    /// Context vectors attention should attend over this step.
    pub fn context(&self) -> Vec<Vec<f32>> {
        self.ring.view().keys()
    }

    /// Cosine-nearest recorded waypoint to `query`.
    ///
    /// A deterministic retrieval baseline: no learning involved, so it measures
    /// what the *encoding* alone supports. A trained attention head should meet
    /// or beat this; if it cannot, the problem is the model, not the geometry.
    pub fn nearest(&self, query: &[f32]) -> Option<&Waypoint> {
        let mut best: Option<(f32, usize)> = None;
        for (i, w) in self.waypoints.iter().enumerate() {
            let code = self.encoder.encode(w);
            let sim = cosine(query, &code);
            if best.map_or(true, |(b, _)| sim > b) {
                best = Some((sim, i));
            }
        }
        best.map(|(_, i)| &self.waypoints[i])
    }

    /// Where was the object at time `t`?
    pub fn recall_at_time(&self, t: f32) -> Option<&Waypoint> {
        self.nearest(&self.encoder.query_at_time(t))
    }

    /// The nearest recorded waypoint to `pos`, event or not.
    pub fn recall_at_place(&self, pos: [f32; 3]) -> Option<&Waypoint> {
        self.nearest(&self.encoder.query_at_place(pos))
    }

    /// The nearest waypoint to `pos` that actually recorded an event.
    ///
    /// [`Self::recall_at_place`] answers "what is the closest sample", which on
    /// a densely sampled path is almost always ordinary motion — the event
    /// waypoint is one among thousands of neighbours. When the question is
    /// *what happened here*, search only the tagged waypoints.
    pub fn recall_event_at_place(&self, pos: [f32; 3]) -> Option<&Waypoint> {
        let query = self.encoder.query_at_place(pos);
        let mut best: Option<(f32, usize)> = None;
        for (i, w) in self.waypoints.iter().enumerate() {
            if w.event.is_none() {
                continue;
            }
            let sim = cosine(&query, &self.encoder.encode(w));
            if best.map_or(true, |(b, _)| sim > b) {
                best = Some((sim, i));
            }
        }
        best.map(|(_, i)| &self.waypoints[i])
    }

    /// Events recorded between two times, in order.
    pub fn events_between(&self, t0: f32, t1: f32) -> Vec<&Waypoint> {
        let (lo, hi) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
        self.waypoints
            .iter()
            .filter(|w| w.t >= lo && w.t <= hi && w.event.is_some())
            .collect()
    }

    /// Straight-line distance travelled across the recorded path.
    pub fn path_length(&self) -> f32 {
        self.waypoints
            .windows(2)
            .map(|p| {
                let (a, b) = (p[0].pos, p[1].pos);
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
            })
            .sum()
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let d = (na.sqrt() * nb.sqrt()).max(1e-8);
    dot / d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc() -> JourneyEncoder {
        JourneyEncoder::new(6, 100.0)
    }

    /// A straight walk along +x over t in [0,1], with one event midway.
    fn walk(n: usize) -> JourneyMemory {
        let e = enc();
        let mut m = JourneyMemory::new(e, ContextRingConfig::new().with_ring_capacity(16));
        for i in 0..n {
            let t = i as f32 / (n - 1).max(1) as f32;
            let w = Waypoint::new(t, [t * 100.0, 0.0, 0.0]);
            if i == n / 2 {
                m.observe(w.with_event("crossed the bridge"));
            } else {
                m.observe(w);
            }
        }
        m
    }

    #[test]
    fn encoder_width_is_consistent() {
        let e = enc();
        assert_eq!(e.dim(), (3 + 1) * 2 * 6);
        let w = Waypoint::new(0.5, [1.0, 2.0, 3.0]);
        assert_eq!(e.encode(&w).len(), e.dim());
        assert_eq!(e.query_at_time(0.5).len(), e.dim());
        assert_eq!(e.query_at_place([1.0, 2.0, 3.0]).len(), e.dim());
    }

    #[test]
    fn nearby_positions_encode_more_similarly_than_distant_ones() {
        let e = enc();
        let a = e.encode(&Waypoint::new(0.0, [10.0, 0.0, 0.0]));
        let near = e.encode(&Waypoint::new(0.0, [10.5, 0.0, 0.0]));
        let far = e.encode(&Waypoint::new(0.0, [90.0, 0.0, 0.0]));
        assert!(
            cosine(&a, &near) > cosine(&a, &far),
            "locality violated: near {} vs far {}",
            cosine(&a, &near),
            cosine(&a, &far)
        );
    }

    #[test]
    fn recalls_position_at_a_queried_time() {
        let m = walk(41);
        // The object is at x = 70 when t = 0.7.
        let w = m.recall_at_time(0.7).expect("a waypoint");
        assert!((w.t - 0.7).abs() < 0.05, "recalled t={} for query 0.7", w.t);
        assert!((w.pos[0] - 70.0).abs() < 5.0, "recalled x={} for query 0.7", w.pos[0]);
    }

    #[test]
    fn recalls_what_happened_near_a_place() {
        let m = walk(41);
        // The event sits at the midpoint, x = 50.
        let w = m.recall_at_place([50.0, 0.0, 0.0]).expect("a waypoint");
        assert!((w.pos[0] - 50.0).abs() < 6.0, "recalled x={} near query 50", w.pos[0]);
    }

    #[test]
    fn event_query_returns_an_event_not_the_nearest_sample() {
        // On a dense path the nearest sample to any point is almost always
        // ordinary motion, so a "what happened here" query must search only
        // tagged waypoints or it reports nothing useful.
        let m = walk(201);
        let w = m.recall_event_at_place([52.0, 0.0, 0.0]).expect("an event");
        assert_eq!(w.event.as_deref(), Some("crossed the bridge"));

        // The unfiltered query, by contrast, lands on untagged motion here.
        let plain = m.recall_at_place([52.0, 0.0, 0.0]).expect("a waypoint");
        assert!(plain.event.is_none(), "dense sampling should surface plain motion");
    }

    #[test]
    fn event_query_is_none_when_nothing_was_tagged() {
        let e = enc();
        let mut m = JourneyMemory::new(e, ContextRingConfig::new().with_ring_capacity(8));
        for i in 0..10 {
            m.observe(Waypoint::new(i as f32 / 9.0, [i as f32, 0.0, 0.0]));
        }
        assert!(m.recall_event_at_place([5.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn events_between_returns_only_tagged_waypoints_in_range() {
        let m = walk(41);
        assert_eq!(m.events_between(0.0, 1.0).len(), 1);
        assert_eq!(m.events_between(0.0, 0.2).len(), 0, "event is at t=0.5");
        let e = m.events_between(0.4, 0.6);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].event.as_deref(), Some("crossed the bridge"));
    }

    #[test]
    fn events_outrank_ordinary_motion_in_salience() {
        let plain = Waypoint::new(0.5, [0.0; 3]);
        let evented = Waypoint::new(0.5, [0.0; 3]).with_event("something");
        assert!(evented.salience > plain.salience, "events must survive compression");
    }

    #[test]
    fn path_length_matches_the_walk() {
        let m = walk(41);
        assert!((m.path_length() - 100.0).abs() < 1e-2, "got {}", m.path_length());
    }

    #[test]
    fn ring_dim_follows_the_encoder() {
        let e = enc();
        let m = JourneyMemory::new(e.clone(), ContextRingConfig::new().with_dim(9999));
        assert_eq!(m.ring().config().dim, e.dim(), "codes must not be truncated");
    }

    #[test]
    fn long_journey_stays_bounded_and_fully_represented() {
        let e = enc();
        let mut m = JourneyMemory::new(
            e,
            ContextRingConfig::new().with_ring_capacity(64).with_max_rings(8).with_retain_sealed_entries(false),
        );
        for i in 0..5_000 {
            let t = i as f32 / 5_000.0;
            m.observe(Waypoint::new(t, [t * 100.0, (i % 7) as f32, 0.0]));
        }
        assert_eq!(m.len(), 5_000);
        assert!(m.ring().total_represented() >= 5_000, "history must survive sealing");
        assert!(m.context().len() <= m.ring().config().view_width());
    }
}
