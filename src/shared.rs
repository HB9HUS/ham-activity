use log::trace;
use std::sync::{Arc, RwLock};

/// A thin wrapper around `Arc<RwLock<T>>`.
pub struct Shared<T>(Arc<RwLock<T>>);

impl<T> Shared<T> {
    /// Create a new shared value
    pub fn new(inner: T) -> Self {
        Shared(Arc::new(RwLock::new(inner)))
    }

    /// Acquire a write lock, returning the guard.
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        trace!("write lock");
        self.0.write().expect("RwLock poisoned")
    }

    /// Acquire a read lock (optional convenience).
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
        trace!("read lock");
        self.0.read().expect("RwLock poisoned")
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}
