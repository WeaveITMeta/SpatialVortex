//! Memory-mapped file storage for confidence patterns
//!
//! Uses memory-mapped files for efficient, persistent storage of high-value patterns.
//! Supports optional AES-256-GCM-SIV encryption for secure storage.

use anyhow::{Context, Result};
use memmap2::{MmapMut, MmapOptions};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;

#[cfg(feature = "lake")]
use super::encryption::SecureStorage;

/// Entry metadata stored in the index
#[derive(Debug, Clone)]
struct Entry {
    offset: usize,    // Offset in memory-mapped file
    length: usize,    // Length of stored data
    #[allow(dead_code)]  // Reserved for temporal queries and expiration
    timestamp: u64,   // When it was stored
}

/// Memory-mapped confidence lake for pattern storage
///
/// Provides efficient persistent storage using memory-mapped files.
/// Patterns are stored with their timestamps as keys.
///
/// # Examples
///
/// ```no_run
/// use spatial_vortex::confidence_lake::ConfidenceLake;
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// // Create 100MB lake
/// let mut lake = ConfidenceLake::create(Path::new("patterns.lake"), 100)?;
///
/// // Store pattern
/// let data = vec![1, 2, 3, 4, 5];
/// lake.store(12345, &data)?;
///
/// // Retrieve pattern
/// let retrieved = lake.retrieve(12345)?;
/// assert_eq!(data, retrieved);
/// # Ok(())
/// # }
/// ```
pub struct ConfidenceLake {
    /// Memory-mapped file
    mmap: MmapMut,
    /// Index mapping timestamps to entries
    index: HashMap<u64, Entry>,
    /// Current write offset
    free_offset: usize,
    /// Total capacity in bytes
    capacity: usize,
    /// Optional encryption for secure storage
    #[cfg(feature = "lake")]
    encryption: Option<SecureStorage>,
}

impl ConfidenceLake {
    /// Creates a new confidence lake with specified size
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the lake file
    /// * `size_mb` - Size in megabytes
    ///
    /// # Errors
    ///
    /// Returns error if file creation or mapping fails
    pub fn create(path: &Path, size_mb: usize) -> Result<Self> {
        let capacity = size_mb * 1024 * 1024;
        
        // Create file with specified size
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Failed to create lake file")?;
        
        file.set_len(capacity as u64)
            .context("Failed to set file size")?;
        
        // Memory map the file
        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .context("Failed to create memory map")?
        };
        
        Ok(Self {
            mmap,
            index: HashMap::new(),
            free_offset: 0,
            capacity,
            #[cfg(feature = "lake")]
            encryption: None,
        })
    }
    
    /// Opens an existing confidence lake
    ///
    /// # Errors
    ///
    /// Returns error if file doesn't exist or mapping fails
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .context("Failed to open lake file")?;
        
        let capacity = file.metadata()?.len() as usize;
        
        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .context("Failed to create memory map")?
        };
        
        // TODO: Rebuild index from stored metadata
        let mut lake = Self {
            mmap,
            index: HashMap::new(),
            free_offset: 0,
            capacity,
            #[cfg(feature = "lake")]
            encryption: None,
        };
        
        lake.rebuild_index()?;
        
        Ok(lake)
    }
    
    /// Enables encryption with a provided key
    ///
    /// # Arguments
    ///
    /// * `key` - 256-bit (32 byte) encryption key
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use spatial_vortex::confidence_lake::{ConfidenceLake, SecureStorage};
    /// # use std::path::Path;
    /// # fn main() -> anyhow::Result<()> {
    /// let key = SecureStorage::generate_key();
    /// let mut lake = ConfidenceLake::create(Path::new("secure.lake"), 100)?;
    /// lake.enable_encryption(&key);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "lake")]
    pub fn enable_encryption(&mut self, key: &[u8; 32]) {
        self.encryption = Some(SecureStorage::new(key));
    }
    
    /// Disables encryption (stores data in plaintext)
    #[cfg(feature = "lake")]
    pub fn disable_encryption(&mut self) {
        self.encryption = None;
    }
    
    /// Returns whether encryption is enabled
    #[cfg(feature = "lake")]
    pub fn is_encrypted(&self) -> bool {
        self.encryption.is_some()
    }
    
    /// Stores data with the given timestamp
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in milliseconds
    /// * `data` - Data to store
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Not enough space
    /// - Write fails
    /// - Timestamp already exists
    ///
    /// # Format
    ///
    /// ```text
    /// [8 bytes: length][data][8 bytes: timestamp][1 byte: 0xFF marker]
    /// ```
    pub fn store(&mut self, timestamp: u64, data: &[u8]) -> Result<()> {
        if self.index.contains_key(&timestamp) {
            anyhow::bail!("Timestamp {} already exists", timestamp);
        }
        
        // Encrypt data if encryption is enabled
        #[cfg(feature = "lake")]
        let data_to_store = if let Some(ref encryption) = self.encryption {
            encryption.encrypt(data)?
        } else {
            data.to_vec()
        };
        
        #[cfg(not(feature = "lake"))]
        let data_to_store = data.to_vec();
        
        // Calculate total size needed
        let entry_size = 8 + data_to_store.len() + 8 + 1; // length + data + timestamp + marker
        
        if self.free_offset + entry_size > self.capacity {
            anyhow::bail!("Not enough space in lake");
        }
        
        let start_offset = self.free_offset;
        
        // Write length
        self.mmap[self.free_offset..self.free_offset + 8]
            .copy_from_slice(&(data_to_store.len() as u64).to_le_bytes());
        self.free_offset += 8;
        
        // Write data (encrypted or plaintext)
        self.mmap[self.free_offset..self.free_offset + data_to_store.len()]
            .copy_from_slice(&data_to_store);
        self.free_offset += data_to_store.len();
        
        // Write timestamp
        self.mmap[self.free_offset..self.free_offset + 8]
            .copy_from_slice(&timestamp.to_le_bytes());
        self.free_offset += 8;
        
        // Write marker
        self.mmap[self.free_offset] = 0xFF;
        self.free_offset += 1;
        
        // Flush to disk
        self.mmap.flush()?;
        
        // Update index
        self.index.insert(
            timestamp,
            Entry {
                offset: start_offset,
                length: data.len(),
                timestamp,
            },
        );
        
        Ok(())
    }
    
    /// Retrieves data for the given timestamp
    ///
    /// # Errors
    ///
    /// Returns error if timestamp not found or read fails
    pub fn retrieve(&self, timestamp: u64) -> Result<Vec<u8>> {
        let entry = self
            .index
            .get(&timestamp)
            .context("Timestamp not found")?;
        
        // Skip length field (8 bytes), read data
        let data_start = entry.offset + 8;
        let data_end = data_start + entry.length;
        
        let stored_data = self.mmap[data_start..data_end].to_vec();
        
        // Decrypt data if encryption is enabled
        #[cfg(feature = "lake")]
        if let Some(ref encryption) = self.encryption {
            return encryption.decrypt(&stored_data);
        }
        
        Ok(stored_data)
    }
    
    /// Returns all stored timestamps
    pub fn list_timestamps(&self) -> Vec<u64> {
        let mut timestamps: Vec<u64> = self.index.keys().copied().collect();
        timestamps.sort_unstable();
        timestamps
    }
    
    /// Returns the number of entries
    pub fn len(&self) -> usize {
        self.index.len()
    }
    
    /// Returns true if the lake is empty
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    
    /// Returns the used space in bytes
    pub fn used_space(&self) -> usize {
        self.free_offset
    }
    
    /// Returns the available space in bytes
    pub fn available_space(&self) -> usize {
        self.capacity - self.free_offset
    }
    
    /// Rebuilds the index by scanning the mmap file
    fn rebuild_index(&mut self) -> Result<()> {
        self.index.clear();
        self.free_offset = 0;
        
        while self.free_offset + 17 < self.capacity {
            // Check for marker
            if self.mmap[self.free_offset] == 0 {
                break; // End of data
            }
            
            let start_offset = self.free_offset;
            
            // Read length
            let mut length_bytes = [0u8; 8];
            length_bytes.copy_from_slice(&self.mmap[self.free_offset..self.free_offset + 8]);
            let length = u64::from_le_bytes(length_bytes) as usize;
            self.free_offset += 8;
            
            // Skip data
            self.free_offset += length;
            
            // Read timestamp
            let mut timestamp_bytes = [0u8; 8];
            timestamp_bytes.copy_from_slice(&self.mmap[self.free_offset..self.free_offset + 8]);
            let timestamp = u64::from_le_bytes(timestamp_bytes);
            self.free_offset += 8;
            
            // Verify marker
            if self.mmap[self.free_offset] != 0xFF {
                break; // Invalid entry
            }
            self.free_offset += 1;
            
            // Add to index
            self.index.insert(
                timestamp,
                Entry {
                    offset: start_offset,
                    length,
                    timestamp,
                },
            );
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_lake() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let lake = ConfidenceLake::create(&path, 10).unwrap();
        
        assert_eq!(lake.capacity, 10 * 1024 * 1024);
        assert_eq!(lake.free_offset, 0);
        assert!(lake.is_empty());
    }
    
    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let mut lake = ConfidenceLake::create(&path, 10).unwrap();
        
        let data = b"Test pattern data";
        lake.store(12345, data).unwrap();
        
        let retrieved = lake.retrieve(12345).unwrap();
        assert_eq!(data, &retrieved[..]);
    }
    
    #[test]
    fn test_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let mut lake = ConfidenceLake::create(&path, 10).unwrap();
        
        for i in 0..10 {
            let data = format!("Pattern {}", i);
            lake.store(i, data.as_bytes()).unwrap();
        }
        
        assert_eq!(lake.len(), 10);
        
        for i in 0..10 {
            let retrieved = lake.retrieve(i).unwrap();
            let expected = format!("Pattern {}", i);
            assert_eq!(expected.as_bytes(), &retrieved[..]);
        }
    }
    
    #[test]
    fn test_duplicate_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let mut lake = ConfidenceLake::create(&path, 10).unwrap();
        
        lake.store(12345, b"First").unwrap();
        let result = lake.store(12345, b"Second");
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let lake = ConfidenceLake::create(&path, 10).unwrap();
        
        let result = lake.retrieve(99999);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_list_timestamps() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let mut lake = ConfidenceLake::create(&path, 10).unwrap();
        
        lake.store(300, b"c").unwrap();
        lake.store(100, b"a").unwrap();
        lake.store(200, b"b").unwrap();
        
        let timestamps = lake.list_timestamps();
        assert_eq!(timestamps, vec![100, 200, 300]); // Should be sorted
    }
    
    #[test]
    fn test_space_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        let mut lake = ConfidenceLake::create(&path, 10).unwrap();
        
        let initial_available = lake.available_space();
        
        lake.store(12345, b"Test data").unwrap();
        
        assert!(lake.used_space() > 0);
        assert!(lake.available_space() < initial_available);
    }
    
    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.lake");
        
        // Create and store
        {
            let mut lake = ConfidenceLake::create(&path, 10).unwrap();
            lake.store(12345, b"Persistent data").unwrap();
        }
        
        // Reopen and verify
        {
            let lake = ConfidenceLake::open(&path).unwrap();
            let retrieved = lake.retrieve(12345).unwrap();
            assert_eq!(b"Persistent data", &retrieved[..]);
        }
    }
}
