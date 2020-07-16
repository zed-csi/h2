use std::sync::{
    self,
    atomic::{AtomicUsize, Ordering::Relaxed},
};
use std::thread;

#[derive(Debug)]
pub(crate) struct Mutex<T> {
    inner: sync::Mutex<T>,
    id: usize,
}

#[derive(Debug)]
pub(crate) struct MutexGuard<'a, T> {
    inner: sync::MutexGuard<'a, T>,
    span: tracing::Span,
}

#[derive(Debug)]
pub(crate) enum TryLockError<'a, T> {
    WouldBlock,
    Poisoned(MutexGuard<'a, T>),
}

impl<T> Mutex<T> {
    pub(crate) fn new(data: T) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Relaxed);
        Self {
            inner: sync::Mutex::new(data),
            id,
        }
    }

    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, T>, MutexGuard<'_, T>> {
        tracing::trace_span!("lock", id = self.id, ty = %std::any::type_name::<T>())
            .in_scope(|| self.inner.lock())
            .map_err(|e| {
                let span = tracing::error_span!("poisoned", id = self.id, thread = ?thread::current().id(),  ty = %std::any::type_name::<T>());
                tracing::error!(parent: &span, "poisoned: {}", e);
                MutexGuard {
                    inner: e.into_inner(),
                    span,
                }
            }).map(|guard| {
                let span = tracing::trace_span!("locked", id = self.id, thread = ?thread::current().id(), ty = %std::any::type_name::<T>());
                tracing::trace!(parent: &span, "locked");
                MutexGuard {
                    inner: guard,
                    span
                }
            })
    }

    pub(crate) fn try_lock(&self) -> Result<MutexGuard<'_, T>, TryLockError<'_, T>> {
        let inner = self.inner.try_lock().map_err(|e| match e {
            sync::TryLockError::WouldBlock => {
                tracing::trace!(id = self.id, thread = ?thread::current().id(),"WouldBlock");
                TryLockError::WouldBlock
            },
            sync::TryLockError::Poisoned(e) => {
                let span = tracing::error_span!("poisoned", id = self.id, thread = ?thread::current().id(),  ty = %std::any::type_name::<T>());
                tracing::error!(parent: &span, "poisoned: {}", e);
                TryLockError::Poisoned(MutexGuard {
                    inner: e.into_inner(),
                    span,
                })
            }
        })?;
        let span = tracing::trace_span!("try_locked", id = self.id, thread = ?thread::current().id(), ty = %std::any::type_name::<T>());
        tracing::trace!(parent: &span, "locked");
        Ok(MutexGuard { inner, span })
    }
}

impl<T> std::ops::Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<T> std::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}
