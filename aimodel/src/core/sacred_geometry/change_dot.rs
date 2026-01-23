use crate::flux_matrix::FluxMatrixEngine;

/// Vortex Math Change Dot Iterator
/// 
/// Implements the sacred doubling sequence: 1 → 2 → 4 → 8 → 7 → 5 → 1 (forward)
/// Positions 3, 6, 9 do NOT appear in this sequence - they are sacred anchors
#[derive(Clone)]
pub struct ChangeDotIter {
    engine: FluxMatrixEngine,
    current: u8,
    steps: u64,
    in_cycle: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeDotEvent {
    Step { from: u8, to: u8, to_position: u8, cycle_index: u64 },
    SacredHit { position: u8 },
    Loop { length: u8 },
}

impl ChangeDotIter {
    pub fn from_seed(seed: u64, engine: &FluxMatrixEngine) -> Self {
        let reduced = engine.reduce_digits(seed) as u8;
        Self { engine: engine.clone(), current: reduced, steps: 0, in_cycle: false }
    }

    pub fn from_value(value: u8, engine: &FluxMatrixEngine) -> Self {
        let v = if value <= 9 { value } else { (engine.reduce_digits(value as u64) as u8) % 10 };
        Self { engine: engine.clone(), current: v, steps: 0, in_cycle: false }
    }

    /// Forward propagation step following doubling sequence: 1→2→4→8→7→5→1
    fn next_step(&mut self) -> ChangeDotEvent {
        let from = self.current;
        let next = self.engine.reduce_digits((self.current as u64) * 2) as u8;
        self.current = next;
        self.steps += 1;
        
        // Detect cycle completion (1→2→4→8→7→5→1)
        if next == 1 && self.in_cycle {
            self.in_cycle = false;
            ChangeDotEvent::Loop { length: 6 }
        } else {
            if from == 1 { self.in_cycle = true; }
            let pos = self.engine.flux_value_to_position(next).unwrap_or(0);
            
            // Detect if we're near sacred positions (but never land on them)
            // 3, 6, 9 don't appear in the doubling sequence - they are anchor points
            
            ChangeDotEvent::Step { from, to: next, to_position: pos, cycle_index: self.steps }
        }
    }

    pub fn current(&self) -> u8 { self.current }
}

impl Iterator for ChangeDotIter {
    type Item = ChangeDotEvent;
    fn next(&mut self) -> Option<Self::Item> {
        // Follow the natural doubling sequence
        // Sacred positions (3,6,9) influence but don't participate in the flow
        Some(self.next_step())
    }
}

pub fn parse_change_dot(input: &str, engine: &FluxMatrixEngine) -> Vec<ChangeDotIter> {
    input
        .split('.')
        .filter_map(|part| {
            let t = part.trim();
            if t.is_empty() { return None; }
            match t.parse::<u64>() {
                Ok(n) => Some(ChangeDotIter::from_seed(n, engine)),
                Err(_) => None,
            }
        })
        .collect()
}

/// Backward propagation chain for training: 1 → 5 → 7 → 8 → 4 → 2 → 1 (cycles)
/// 
/// This is the reverse of the doubling sequence, used for backpropagation
/// and gradient descent in the Vortex Math training engine.
/// 
/// # Examples
/// 
/// ```
/// use spatial_vortex::change_dot::BackwardChain;
/// 
/// let chain = BackwardChain::new();
/// let path: Vec<u8> = chain.take(6).collect();
/// assert_eq!(path, vec![1, 5, 7, 8, 4, 2]);
/// ```
#[derive(Clone, Debug)]
pub struct BackwardChain {
    sequence: [u8; 6],
    index: usize,
    cycle_count: u64,
}

impl BackwardChain {
    /// Creates a new backward chain iterator starting from position 1.
    pub fn new() -> Self {
        Self {
            sequence: [1, 5, 7, 8, 4, 2],
            index: 0,
            cycle_count: 0,
        }
    }
    
    /// Returns the current position in the chain.
    pub fn current(&self) -> u8 {
        self.sequence[self.index]
    }
    
    /// Returns the number of complete cycles traversed.
    pub fn cycles(&self) -> u64 {
        self.cycle_count
    }
}

impl Iterator for BackwardChain {
    type Item = u8;
    
    fn next(&mut self) -> Option<u8> {
        let val = self.sequence[self.index];
        self.index += 1;
        
        if self.index >= self.sequence.len() {
            self.index = 0;
            self.cycle_count += 1;
        }
        
        Some(val)
    }
}

#[cfg(test)]
mod backward_chain_tests {
    use super::*;
    
    #[test]
    fn test_backward_sequence() {
        let chain = BackwardChain::new();
        let path: Vec<u8> = chain.take(6).collect();
        assert_eq!(path, vec![1, 5, 7, 8, 4, 2]);
    }
    
    #[test]
    fn test_backward_cycle() {
        let mut chain = BackwardChain::new();
        for _ in 0..6 { chain.next(); }
        assert_eq!(chain.cycles(), 1);
        assert_eq!(chain.current(), 1);
    }
    
    #[test]
    fn test_multiple_cycles() {
        let chain = BackwardChain::new();
        let path: Vec<u8> = chain.take(18).collect();
        // Should cycle 3 times
        assert_eq!(path[0..6], [1, 5, 7, 8, 4, 2]);
        assert_eq!(path[6..12], [1, 5, 7, 8, 4, 2]);
        assert_eq!(path[12..18], [1, 5, 7, 8, 4, 2]);
    }
}
