//! Useful synchronization primitives.

use std::{cell::UnsafeCell, future::Future, mem::MaybeUninit, ptr::addr_of_mut, sync::Arc};
use tokio::sync::RwLock;

/// A synchronization primitive which can initialize its value in a separated task.
#[derive(Debug)]
pub struct ConcurrentInit<T>(Arc<ConcurrentInitInner<T>>);
impl<T: Send + Sync + 'static> ConcurrentInit<T> {
    /// Creates a new `ConcurrentInit<T>` instance.
    pub fn new<F: Future<Output = T> + Send + 'static>(op: F) -> Self {
        let inner = ConcurrentInitInner {
            lock: RwLock::new(()),
            data: MaybeUninit::uninit().into(),
        };
        let object = Self(Arc::new(inner));
        {
            let object = object.clone();
            tokio::spawn(async move {
                let _lock = object.0.lock.write().await;
                let value = op.await;

                // SAFETY: Before we released `_lock`, there will be no access to `object.0.data`.
                unsafe {
                    let ptr = (*object.0.data.get()).as_mut_ptr();
                    addr_of_mut!(*ptr).write(value);
                }
            });
        }
        object
    }

    /// Gets the value.
    pub async fn get(&self) -> &T {
        self.0.get().await
    }
}
impl<T> Clone for ConcurrentInit<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug)]
struct ConcurrentInitInner<T> {
    lock: RwLock<()>,
    data: UnsafeCell<MaybeUninit<T>>,
}
impl<T> ConcurrentInitInner<T> {
    /// Gets the value.
    async fn get(&self) -> &T {
        // SAFETY: The data is always initialized when the `RwLock<MaybeUninit<T>>` is read since the initialization task
        //         holds an exclusive `RwLockWriteGuard<_>`.
        let _lock = self.lock.read().await;
        unsafe { &*((*self.data.get()).assume_init_ref() as *const T) }
    }
}
impl<T> Drop for ConcurrentInitInner<T> {
    fn drop(&mut self) {
        // SAFETY: The data is always initialized when `Self` gets dropped since this type is only used in `Arc`s and the
        //         initialization task owns an `Arc<ConcurrentInitInner<_>>`.
        unsafe {
            (*self.data.get()).assume_init_drop();
        }
    }
}
unsafe impl<T> Send for ConcurrentInitInner<T> where T: Send {}
unsafe impl<T> Sync for ConcurrentInitInner<T> where T: Sync {}
