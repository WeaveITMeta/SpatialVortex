//! Generators Module
//! 
//! Stub module for generators functionality.

use crate::error::Result;

pub mod subjects;

/// Generator trait stub
pub trait Generator {
    fn generate(&self) -> Result<String>;
}

/// Text generator stub
pub struct TextGenerator;

impl TextGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextGenerator {
    fn default() -> Self {
        Self::new()
    }
}
