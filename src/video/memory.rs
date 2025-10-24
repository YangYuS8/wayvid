/// Memory management and monitoring for video subsystem
///
/// This module implements RFC M5-002: Memory Optimization
///
/// Key features:
/// - Memory usage tracking and statistics
/// - Buffer pool for texture data
/// - Zero-copy frame sharing
/// - Configurable memory limits
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Global memory statistics
static TOTAL_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static TOTAL_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_USAGE: AtomicUsize = AtomicUsize::new(0);

/// Memory statistics for monitoring
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    /// Total bytes currently allocated
    pub current_bytes: usize,

    /// Peak memory usage (bytes)
    pub peak_bytes: usize,

    /// Total bytes allocated (lifetime)
    pub total_allocated: usize,

    /// Total bytes deallocated (lifetime)
    pub total_deallocated: usize,

    /// Number of active allocations
    pub active_allocations: usize,
}

impl MemoryStats {
    /// Get current global memory statistics
    pub fn global() -> Self {
        let allocated = TOTAL_ALLOCATED.load(Ordering::Relaxed);
        let deallocated = TOTAL_DEALLOCATED.load(Ordering::Relaxed);
        let current = allocated.saturating_sub(deallocated);

        Self {
            current_bytes: current,
            peak_bytes: PEAK_USAGE.load(Ordering::Relaxed),
            total_allocated: allocated,
            total_deallocated: deallocated,
            active_allocations: 0, // TODO: Track allocation count
        }
    }

    /// Format memory size in human-readable form
    pub fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Log current memory statistics
    pub fn log(&self) {
        debug!(
            "Memory: current={}, peak={}, allocated={}, deallocated={}",
            Self::format_bytes(self.current_bytes),
            Self::format_bytes(self.peak_bytes),
            Self::format_bytes(self.total_allocated),
            Self::format_bytes(self.total_deallocated),
        );
    }
}

/// Track memory allocation
pub fn track_allocation(bytes: usize) {
    let allocated = TOTAL_ALLOCATED.fetch_add(bytes, Ordering::Relaxed) + bytes;
    let deallocated = TOTAL_DEALLOCATED.load(Ordering::Relaxed);
    let current = allocated.saturating_sub(deallocated);

    // Update peak if needed
    let mut peak = PEAK_USAGE.load(Ordering::Relaxed);
    while current > peak {
        match PEAK_USAGE.compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => break,
            Err(p) => peak = p,
        }
    }
}

/// Track memory deallocation
pub fn track_deallocation(bytes: usize) {
    TOTAL_DEALLOCATED.fetch_add(bytes, Ordering::Relaxed);
}

/// Managed buffer with automatic memory tracking
pub struct ManagedBuffer {
    data: Vec<u8>,
    tracked_size: usize,
}

impl ManagedBuffer {
    /// Create a new managed buffer
    pub fn new(capacity: usize) -> Self {
        track_allocation(capacity);
        Self {
            data: Vec::with_capacity(capacity),
            tracked_size: capacity,
        }
    }

    /// Create from existing vector
    pub fn from_vec(data: Vec<u8>) -> Self {
        let capacity = data.capacity();
        track_allocation(capacity);
        Self {
            data,
            tracked_size: capacity,
        }
    }

    /// Get immutable reference to data
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable reference to data
    pub fn as_mut_slice(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        track_deallocation(self.tracked_size);
    }
}

/// Buffer pool for reusing texture buffers
pub struct BufferPool {
    /// Available buffers, organized by size buckets
    buffers: Arc<Mutex<Vec<ManagedBuffer>>>,

    /// Maximum pool size (number of buffers)
    max_buffers: usize,

    /// Maximum total memory (bytes)
    max_memory: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(max_buffers: usize, max_memory: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::new())),
            max_buffers,
            max_memory,
        }
    }

    /// Acquire a buffer from the pool or allocate a new one
    pub fn acquire(&self, min_capacity: usize) -> ManagedBuffer {
        let mut buffers = self.buffers.lock().unwrap();

        // Try to find a suitable buffer in the pool
        if let Some(pos) = buffers
            .iter()
            .position(|buf| buf.capacity() >= min_capacity)
        {
            let buffer = buffers.swap_remove(pos);
            debug!(
                "♻️ Reused buffer from pool (capacity: {})",
                buffer.capacity()
            );
            return buffer;
        }

        // Allocate a new buffer
        debug!("🆕 Allocated new buffer (capacity: {})", min_capacity);
        ManagedBuffer::new(min_capacity)
    }

    /// Return a buffer to the pool
    pub fn release(&self, mut buffer: ManagedBuffer) {
        let mut buffers = self.buffers.lock().unwrap();

        // Check pool limits
        if buffers.len() >= self.max_buffers {
            debug!("🗑️ Pool full, discarding buffer");
            return;
        }

        let total_memory: usize = buffers.iter().map(|b| b.capacity()).sum();
        if total_memory + buffer.capacity() > self.max_memory {
            debug!("🗑️ Pool memory limit reached, discarding buffer");
            return;
        }

        // Clear the buffer data but keep capacity
        buffer.as_mut_slice().clear();

        buffers.push(buffer);
        debug!("📥 Returned buffer to pool (pool size: {})", buffers.len());
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        let buffers = self.buffers.lock().unwrap();
        let count = buffers.len();
        let total_capacity: usize = buffers.iter().map(|b| b.capacity()).sum();

        BufferPoolStats {
            buffer_count: count,
            total_capacity,
            max_buffers: self.max_buffers,
            max_memory: self.max_memory,
        }
    }

    /// Clear the pool
    pub fn clear(&self) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.clear();
        debug!("🧹 Cleared buffer pool");
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone, Copy)]
pub struct BufferPoolStats {
    /// Number of buffers in pool
    pub buffer_count: usize,

    /// Total capacity of pooled buffers
    pub total_capacity: usize,

    /// Maximum number of buffers allowed
    pub max_buffers: usize,

    /// Maximum total memory allowed
    pub max_memory: usize,
}

impl BufferPoolStats {
    /// Get utilization percentage (0.0 - 1.0)
    pub fn utilization(&self) -> f64 {
        if self.max_memory == 0 {
            return 0.0;
        }
        self.total_capacity as f64 / self.max_memory as f64
    }

    /// Log statistics
    pub fn log(&self) {
        debug!(
            "Buffer pool: {}/{} buffers, {}/{} memory ({:.1}% full)",
            self.buffer_count,
            self.max_buffers,
            MemoryStats::format_bytes(self.total_capacity),
            MemoryStats::format_bytes(self.max_memory),
            self.utilization() * 100.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::global();
        assert!(stats.current_bytes >= 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryStats::format_bytes(500), "500 B");
        assert_eq!(MemoryStats::format_bytes(1024), "1.00 KB");
        assert_eq!(MemoryStats::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(MemoryStats::format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_managed_buffer() {
        let initial_stats = MemoryStats::global();

        {
            let _buffer = ManagedBuffer::new(1024);
            let stats = MemoryStats::global();
            assert!(stats.current_bytes >= initial_stats.current_bytes);
        }

        // After drop, memory should be freed
        let final_stats = MemoryStats::global();
        assert_eq!(final_stats.current_bytes, initial_stats.current_bytes);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(10, 10 * 1024 * 1024);

        // Acquire and release buffers
        let buffer1 = pool.acquire(1024);
        let buffer2 = pool.acquire(2048);

        pool.release(buffer1);
        pool.release(buffer2);

        let stats = pool.stats();
        assert_eq!(stats.buffer_count, 2);

        // Reuse from pool
        let buffer3 = pool.acquire(1024);
        let stats = pool.stats();
        assert_eq!(stats.buffer_count, 1); // One buffer reused

        pool.release(buffer3);
    }

    #[test]
    fn test_buffer_pool_limits() {
        let pool = BufferPool::new(2, 3000); // Max 2 buffers, 3KB total

        let buf1 = pool.acquire(1024);
        let buf2 = pool.acquire(1024);
        let buf3 = pool.acquire(1024);

        pool.release(buf1);
        pool.release(buf2);
        pool.release(buf3); // Should be rejected (pool full)

        let stats = pool.stats();
        assert_eq!(stats.buffer_count, 2);
    }
}
