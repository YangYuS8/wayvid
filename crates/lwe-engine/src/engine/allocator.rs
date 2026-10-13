//! Return large freed native heaps at resource-release boundaries.
//!
//! Repeated libmpv/GL initialization can leave glibc-owned free pages resident
//! after the native objects and workers were destroyed. Releasing those pages
//! reduces allocator retention without changing arena policy or live resources.
//! This is reclamation of already-free pages, not evidence of no native leaks.

#[cfg(target_env = "gnu")]
pub(super) fn reclaim_released_pages(boundary: &'static str) {
    const MIN_FREE_BYTES: usize = 64 * 1024 * 1024;
    // SAFETY: GNU allocator statistics are read-only and thread-safe.
    let free_bytes = unsafe { libc::mallinfo2() }.fordblks;
    if free_bytes < MIN_FREE_BYTES {
        return;
    }
    let started = std::time::Instant::now();
    // SAFETY: malloc_trim is thread-safe and only returns allocator-owned free
    // pages to the OS. Callers have released their obsolete resources first;
    // pointers belonging to current/candidate/rollback renderers are untouched.
    let released_pages = unsafe { libc::malloc_trim(0) } != 0;
    tracing::debug!(
        boundary,
        free_bytes,
        released_pages,
        elapsed_micros = started.elapsed().as_micros(),
        "Reclaimed released native allocator pages"
    );
}

#[cfg(not(target_env = "gnu"))]
pub(super) fn reclaim_released_pages(_boundary: &'static str) {}
