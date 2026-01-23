pub mod elp_channels;
pub mod asi_16byte;

pub use elp_channels::{compress_text, CompressionHash, ELPChannels};
pub use asi_16byte::{
    ASI12ByteCompression, 
    ASI16ByteCompression,
    ASI16ByteBuilder,
    ASICompressionEngine, 
    CompressionStats,
    find_nearest_sacred_anchor,
    sixw,  // 6W encoding helpers
};
