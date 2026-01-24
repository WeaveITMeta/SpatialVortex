//! Vortex Mathematics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct VortexPositioningEngine {
    current_position: u8,
}

impl VortexPositioningEngine {
    pub fn new() -> Self { Self { current_position: 1 } }

    pub fn digital_root(n: u64) -> u8 {
        if n == 0 { return 0; }
        let r = (n % 9) as u8;
        if r == 0 { 9 } else { r }
    }

    pub fn advance(&mut self) -> u8 {
        self.current_position = match self.current_position {
            1 => 2, 2 => 4, 4 => 8, 8 => 7, 7 => 5, 5 => 1,
            _ => 1,
        };
        self.current_position
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxPosition {
    pub value: u8,
    pub is_sacred: bool,
}
