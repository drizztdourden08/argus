//! Locks that survive a panic in another thread: the data they guard is plain state, so a poisoned lock is still usable.

use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

/// Lock a mutex, ignoring poison.
pub fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Wait on a condition variable, ignoring poison.
pub fn waited<'a, T>(condvar: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    condvar.wait(guard).unwrap_or_else(PoisonError::into_inner)
}
