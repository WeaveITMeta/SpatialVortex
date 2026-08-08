//! Dynamic context rings — a time-extended, mergeable context window.
//!
//! ## The problem this solves
//! A fixed context window forces a hard choice: keep everything and pay O(n²)
//! attention, or truncate and lose the past. Neither lets a system work a
//! complex problem *across time*, where early context must stay reachable long
//! after it scrolled out.
//!
//! ## The design
//! Context is stored in **rings** — fixed-capacity buffers of encoded entries.
//! Exactly one ring is *open* and receives pushes. When it fills, it is
//! **sealed**: a summary vector is computed and the ring becomes read-only, and
//! a fresh ring opens. Nothing is discarded on seal.
//!
//! What attention actually sees is a [`ContextView`]:
//!
//! ```text
//!   [ open ring entries (full resolution) ]   <- the recent past, verbatim
//!   [ sealed ring summaries (compressed)  ]   <- the deep past, 1 vector/ring
//!   [ headroom                            ]   <- reserved slots, see below
//! ```
//!
//! So the *effective* window is `ring_capacity + max_rings` vectors while the
//! per-step attention cost stays bounded by that same number — the deep past
//! costs one vector per ring, not one per entry. A 256-entry ring with 64 rings
//! addresses 16,384 entries of history through a 320-vector view.
//!
//! ## Headroom
//! `headroom` reserves slots in the view that no stored context may occupy.
//! Multi-step reasoning needs somewhere to put intermediate results that are
//! *not* retrieved history — scratch space that persists across steps within a
//! problem. Without it, working state competes with history for the same slots
//! and long problems evict their own intermediate conclusions.
//!
//! ## Merging
//! [`ContextRingBuffer::merge_sealed`] folds a span of sealed rings into one,
//! salience-weighting their summaries. This is how old context is compressed
//! further as it recedes, keeping the view bounded without dropping a region of
//! time entirely.
//!
//! ## Hot-path swapping
//! [`ContextRingBuffer::swap_out`] / [`ContextRingBuffer::swap_in`] move whole
//! rings out of and back into the live buffer, so a working set can be parked
//! and restored without rebuilding it.

use std::collections::VecDeque;

/// Identifier for a ring within one buffer. Monotonic; never reused.
pub type RingId = u64;

/// A single encoded piece of context.
#[derive(Debug, Clone)]
pub struct ContextEntry {
    /// Encoded representation.
    pub embedding: Vec<f32>,
    /// Retrieval weight. Higher survives compression and eviction.
    pub salience: f32,
    /// Logical step at which this entry was written.
    pub step: u64,
    /// Optional provenance (dataset, doc id, span) for debugging and audit.
    pub tag: Option<String>,
}

impl ContextEntry {
    /// Entry with neutral salience.
    pub fn new(embedding: Vec<f32>) -> Self {
        Self { embedding, salience: 1.0, step: 0, tag: None }
    }

    /// Builder: set salience.
    pub fn with_salience(mut self, s: f32) -> Self {
        self.salience = s;
        self
    }

    /// Builder: set provenance tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
}

/// A fixed-capacity buffer of context entries.
#[derive(Debug, Clone)]
pub struct ContextRing {
    /// Stable identifier.
    pub id: RingId,
    /// Entries, oldest first.
    pub entries: VecDeque<ContextEntry>,
    /// Salience-weighted mean of `entries`, computed at seal time.
    pub summary: Option<Vec<f32>>,
    /// Sealed rings are read-only.
    pub sealed: bool,
    /// Step at which the first entry landed.
    pub opened_step: u64,
    /// Step at which the ring was sealed.
    pub sealed_step: Option<u64>,
    /// How many rings were folded into this one (1 = never merged).
    pub merge_depth: u32,
    /// How many original entries this ring stands for.
    ///
    /// Distinct from `merge_depth`, which counts *rings*. After a merge the
    /// entry vector is emptied (the summary represents it), so `entries.len()`
    /// can no longer answer "how much history is in here" — this can.
    pub represented: usize,
}

impl ContextRing {
    fn new(id: RingId, step: u64) -> Self {
        Self {
            id,
            entries: VecDeque::new(),
            summary: None,
            sealed: false,
            opened_step: step,
            sealed_step: None,
            merge_depth: 1,
            represented: 0,
        }
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the ring holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total salience, used to weight this ring during a merge.
    pub fn total_salience(&self) -> f32 {
        self.entries.iter().map(|e| e.salience).sum()
    }

    /// Compute the salience-weighted mean of the entries.
    ///
    /// Returns `None` for an empty ring or when every salience is zero, rather
    /// than emitting a zero vector that would look like real context.
    fn compute_summary(&self, dim: usize) -> Option<Vec<f32>> {
        if self.entries.is_empty() {
            return None;
        }
        let total: f32 = self.entries.iter().map(|e| e.salience.max(0.0)).sum();
        if total <= f32::EPSILON {
            return None;
        }
        let mut acc = vec![0.0f32; dim];
        for e in &self.entries {
            let w = e.salience.max(0.0) / total;
            for (i, v) in e.embedding.iter().take(dim).enumerate() {
                acc[i] += v * w;
            }
        }
        Some(acc)
    }
}

/// What attention consumes for one step.
#[derive(Debug, Clone)]
pub struct ContextView {
    /// Full-resolution entries from the open ring.
    pub recent: Vec<Vec<f32>>,
    /// One summary vector per sealed ring, oldest first.
    pub compressed: Vec<Vec<f32>>,
    /// Free slots reserved for intermediate reasoning state.
    pub headroom: usize,
}

impl ContextView {
    /// Vectors attention should attend over: recent, then compressed.
    pub fn keys(&self) -> Vec<Vec<f32>> {
        let mut out = Vec::with_capacity(self.recent.len() + self.compressed.len());
        out.extend(self.recent.iter().cloned());
        out.extend(self.compressed.iter().cloned());
        out
    }

    /// Number of vectors in the view (excludes headroom, which holds nothing).
    pub fn len(&self) -> usize {
        self.recent.len() + self.compressed.len()
    }

    /// True when there is nothing to attend over.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Tuning for a [`ContextRingBuffer`].
#[derive(Debug, Clone)]
pub struct ContextRingConfig {
    /// Embedding width. Entries wider than this are truncated; narrower are padded.
    pub dim: usize,
    /// Entries per ring before it seals.
    pub ring_capacity: usize,
    /// Sealed rings retained before the oldest are merged away.
    pub max_rings: usize,
    /// View slots reserved for working state.
    pub headroom: usize,
    /// How many rings a single automatic merge folds together.
    pub merge_span: usize,
}

impl Default for ContextRingConfig {
    fn default() -> Self {
        Self { dim: 256, ring_capacity: 256, max_rings: 64, headroom: 32, merge_span: 4 }
    }
}

impl ContextRingConfig {
    /// Start from defaults.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set embedding width.
    pub fn with_dim(mut self, d: usize) -> Self {
        self.dim = d;
        self
    }
    /// Set entries per ring.
    pub fn with_ring_capacity(mut self, c: usize) -> Self {
        self.ring_capacity = c;
        self
    }
    /// Set retained sealed rings.
    pub fn with_max_rings(mut self, m: usize) -> Self {
        self.max_rings = m;
        self
    }
    /// Set reserved working slots.
    pub fn with_headroom(mut self, h: usize) -> Self {
        self.headroom = h;
        self
    }
    /// Set rings folded per automatic merge.
    pub fn with_merge_span(mut self, s: usize) -> Self {
        self.merge_span = s;
        self
    }

    /// Entries addressable before any merging: `ring_capacity * (max_rings + 1)`.
    pub fn addressable_entries(&self) -> usize {
        self.ring_capacity * (self.max_rings + 1)
    }

    /// Vectors attention sees per step: `ring_capacity + max_rings`.
    pub fn view_width(&self) -> usize {
        self.ring_capacity + self.max_rings
    }
}

/// Ring-buffered context with merge-across-time.
#[derive(Debug, Clone)]
pub struct ContextRingBuffer {
    config: ContextRingConfig,
    /// Sealed rings, oldest first.
    sealed: VecDeque<ContextRing>,
    /// The single ring currently accepting pushes.
    open: ContextRing,
    /// Rings parked by `swap_out`.
    parked: Vec<ContextRing>,
    next_id: RingId,
    step: u64,
}

impl ContextRingBuffer {
    /// Create an empty buffer with one open ring.
    pub fn new(config: ContextRingConfig) -> Self {
        let open = ContextRing::new(0, 0);
        Self { config, sealed: VecDeque::new(), open, parked: Vec::new(), next_id: 1, step: 0 }
    }

    /// Configuration in use.
    pub fn config(&self) -> &ContextRingConfig {
        &self.config
    }

    /// Current logical step.
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Number of sealed rings retained.
    pub fn sealed_count(&self) -> usize {
        self.sealed.len()
    }

    /// Number of parked rings.
    pub fn parked_count(&self) -> usize {
        self.parked.len()
    }

    /// Entries in the open ring.
    pub fn open_len(&self) -> usize {
        self.open.len()
    }

    /// Entries still held at full resolution plus those folded into summaries.
    ///
    /// Counts *represented* entries, including those inside merged rings, so
    /// this keeps rising as history is compressed rather than appearing to drop.
    pub fn total_represented(&self) -> usize {
        self.open.len() + self.sealed.iter().map(|r| r.represented).sum::<usize>()
    }

    /// Normalize an embedding to `config.dim` (truncate or zero-pad).
    fn fit(&self, mut v: Vec<f32>) -> Vec<f32> {
        v.resize(self.config.dim, 0.0);
        v
    }

    /// Append context. Seals the open ring and opens a new one when full.
    pub fn push(&mut self, mut entry: ContextEntry) {
        entry.embedding = self.fit(entry.embedding);
        entry.step = self.step;
        if self.open.entries.is_empty() {
            self.open.opened_step = self.step;
        }
        self.open.entries.push_back(entry);
        self.step += 1;

        if self.open.len() >= self.config.ring_capacity {
            self.seal_open();
        }
    }

    /// Convenience: push a bare embedding with neutral salience.
    pub fn push_embedding(&mut self, embedding: Vec<f32>) {
        self.push(ContextEntry::new(embedding));
    }

    /// Seal the open ring and start a fresh one.
    ///
    /// A no-op on an empty ring, so callers may force a boundary safely.
    pub fn seal_open(&mut self) {
        if self.open.entries.is_empty() {
            return;
        }
        let mut ring = std::mem::replace(&mut self.open, ContextRing::new(self.next_id, self.step));
        self.next_id += 1;
        ring.represented = ring.entries.len();
        ring.summary = ring.compute_summary(self.config.dim);
        ring.sealed = true;
        ring.sealed_step = Some(self.step);
        self.sealed.push_back(ring);

        while self.sealed.len() > self.config.max_rings {
            self.merge_oldest();
        }
    }

    /// Fold the oldest `merge_span` sealed rings into one.
    fn merge_oldest(&mut self) {
        let span = self.config.merge_span.clamp(2, self.sealed.len().max(2));
        if self.sealed.len() < 2 {
            return;
        }
        let take = span.min(self.sealed.len());
        let group: Vec<ContextRing> = (0..take).filter_map(|_| self.sealed.pop_front()).collect();
        if let Some(merged) = self.merge_group(group) {
            self.sealed.push_front(merged);
        }
    }

    /// Merge a set of sealed rings into one, salience-weighting their summaries.
    fn merge_group(&mut self, group: Vec<ContextRing>) -> Option<ContextRing> {
        if group.is_empty() {
            return None;
        }
        let dim = self.config.dim;
        let weights: Vec<f32> = group.iter().map(|r| r.total_salience().max(0.0)).collect();
        let total: f32 = weights.iter().sum();

        let mut acc = vec![0.0f32; dim];
        let mut any = false;
        for (r, w) in group.iter().zip(weights.iter()) {
            let Some(s) = r.summary.as_ref() else { continue };
            // Equal weighting when all salience is zero, so a merge never
            // silently produces an all-zero summary.
            let wn = if total > f32::EPSILON { w / total } else { 1.0 / group.len() as f32 };
            for (i, v) in s.iter().take(dim).enumerate() {
                acc[i] += v * wn;
            }
            any = true;
        }
        if !any {
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;
        let opened = group.first().map(|r| r.opened_step).unwrap_or(0);
        let sealed_step = group.last().and_then(|r| r.sealed_step);
        let depth: u32 = group.iter().map(|r| r.merge_depth).sum();
        let represented: usize = group.iter().map(|r| r.represented).sum();

        Some(ContextRing {
            id,
            entries: VecDeque::new(), // entries are represented by `summary`
            summary: Some(acc),
            sealed: true,
            opened_step: opened,
            sealed_step,
            merge_depth: depth,
            represented,
        })
    }

    /// Explicitly merge the oldest `count` sealed rings.
    ///
    /// Returns the id of the merged ring, or `None` if there was nothing to do.
    pub fn merge_sealed(&mut self, count: usize) -> Option<RingId> {
        if count < 2 || self.sealed.len() < 2 {
            return None;
        }
        let take = count.min(self.sealed.len());
        let group: Vec<ContextRing> = (0..take).filter_map(|_| self.sealed.pop_front()).collect();
        let merged = self.merge_group(group)?;
        let id = merged.id;
        self.sealed.push_front(merged);
        Some(id)
    }

    /// Build the view attention consumes this step.
    pub fn view(&self) -> ContextView {
        ContextView {
            recent: self.open.entries.iter().map(|e| e.embedding.clone()).collect(),
            compressed: self.sealed.iter().filter_map(|r| r.summary.clone()).collect(),
            headroom: self.config.headroom,
        }
    }

    /// Park a sealed ring, removing it from the live view.
    ///
    /// Returns `false` if no sealed ring has that id.
    pub fn swap_out(&mut self, id: RingId) -> bool {
        if let Some(pos) = self.sealed.iter().position(|r| r.id == id) {
            if let Some(r) = self.sealed.remove(pos) {
                self.parked.push(r);
                return true;
            }
        }
        false
    }

    /// Restore a parked ring, re-inserting it in `opened_step` order.
    pub fn swap_in(&mut self, id: RingId) -> bool {
        let Some(pos) = self.parked.iter().position(|r| r.id == id) else {
            return false;
        };
        let ring = self.parked.remove(pos);
        let at = self
            .sealed
            .iter()
            .position(|r| r.opened_step > ring.opened_step)
            .unwrap_or(self.sealed.len());
        self.sealed.insert(at, ring);
        true
    }

    /// Ids of sealed rings, oldest first.
    pub fn sealed_ids(&self) -> Vec<RingId> {
        self.sealed.iter().map(|r| r.id).collect()
    }

    /// Retrieve the summary of a sealed ring.
    pub fn summary_of(&self, id: RingId) -> Option<&Vec<f32>> {
        self.sealed.iter().find(|r| r.id == id).and_then(|r| r.summary.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ContextRingConfig {
        ContextRingConfig::new()
            .with_dim(4)
            .with_ring_capacity(3)
            .with_max_rings(3)
            .with_merge_span(2)
            .with_headroom(2)
    }

    #[test]
    fn seals_when_capacity_reached() {
        let mut b = ContextRingBuffer::new(cfg());
        for i in 0..3 {
            b.push_embedding(vec![i as f32; 4]);
        }
        assert_eq!(b.sealed_count(), 1, "ring should seal at capacity");
        assert_eq!(b.open_len(), 0, "a fresh ring should be open");
    }

    #[test]
    fn summary_is_salience_weighted() {
        let mut b = ContextRingBuffer::new(cfg());
        b.push(ContextEntry::new(vec![0.0; 4]).with_salience(0.0));
        b.push(ContextEntry::new(vec![10.0; 4]).with_salience(1.0));
        b.push(ContextEntry::new(vec![0.0; 4]).with_salience(0.0));
        let id = b.sealed_ids()[0];
        let s = b.summary_of(id).expect("summary");
        // Only the salient entry should contribute.
        assert!((s[0] - 10.0).abs() < 1e-4, "expected 10.0, got {}", s[0]);
    }

    #[test]
    fn zero_salience_ring_yields_no_summary() {
        let mut b = ContextRingBuffer::new(cfg());
        for _ in 0..3 {
            b.push(ContextEntry::new(vec![5.0; 4]).with_salience(0.0));
        }
        let id = b.sealed_ids()[0];
        // Must be absent, not a zero vector masquerading as context.
        assert!(b.summary_of(id).is_none());
    }

    #[test]
    fn view_width_is_bounded_while_history_grows() {
        let mut b = ContextRingBuffer::new(cfg());
        for i in 0..60 {
            b.push_embedding(vec![i as f32; 4]);
        }
        let v = b.view();
        assert!(
            v.len() <= cfg().view_width(),
            "view {} exceeded bound {}",
            v.len(),
            cfg().view_width()
        );
        assert!(b.total_represented() >= 60, "history must remain represented");
    }

    #[test]
    fn merging_preserves_representation_count() {
        let mut b = ContextRingBuffer::new(cfg());
        for i in 0..9 {
            b.push_embedding(vec![i as f32; 4]);
        }
        let before = b.total_represented();
        let sealed_before = b.sealed_count();
        b.merge_sealed(2);
        assert!(b.sealed_count() < sealed_before, "merge should reduce ring count");
        assert_eq!(before, b.total_represented(), "merge must not lose represented entries");
    }

    #[test]
    fn swap_out_and_in_round_trips() {
        let mut b = ContextRingBuffer::new(cfg());
        for i in 0..6 {
            b.push_embedding(vec![i as f32; 4]);
        }
        let ids = b.sealed_ids();
        let target = ids[0];
        let n = b.sealed_count();

        assert!(b.swap_out(target));
        assert_eq!(b.sealed_count(), n - 1);
        assert_eq!(b.parked_count(), 1);
        assert!(b.summary_of(target).is_none(), "parked ring must leave the view");

        assert!(b.swap_in(target));
        assert_eq!(b.sealed_count(), n);
        assert_eq!(b.parked_count(), 0);
        assert!(b.summary_of(target).is_some(), "restored ring must rejoin the view");
        assert_eq!(b.sealed_ids()[0], target, "restored ring keeps chronological order");
    }

    #[test]
    fn headroom_is_reserved_and_not_filled() {
        let mut b = ContextRingBuffer::new(cfg());
        for i in 0..30 {
            b.push_embedding(vec![i as f32; 4]);
        }
        assert_eq!(b.view().headroom, 2, "headroom must survive heavy churn");
    }

    #[test]
    fn embeddings_are_normalized_to_dim() {
        let mut b = ContextRingBuffer::new(cfg());
        b.push_embedding(vec![1.0; 9]); // too wide
        b.push_embedding(vec![1.0; 2]); // too narrow
        for e in b.view().recent {
            assert_eq!(e.len(), 4);
        }
    }

    #[test]
    fn addressable_capacity_far_exceeds_view_width() {
        let c = ContextRingConfig::new(); // 256-entry rings, 64 rings
        assert_eq!(c.view_width(), 320);
        assert_eq!(c.addressable_entries(), 16_640);
    }
}
