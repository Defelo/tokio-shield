#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug, clippy::todo)]
#![warn(missing_docs, missing_debug_implementations)]

use std::future::Future;

use futures::{future::Map, FutureExt};
use tokio::task::{JoinError, JoinHandle};

/// Adds methods to futures to prevent them from being aborted.
pub trait Shield
where
    Self: Future + Send + 'static,
    Self::Output: Send,
{
    /// The [`Future`] returned from [`shield()`](Self::shield).
    type ShieldFuture: Future<Output = Self::Output>;
    /// The [`Future`] returned from [`try_shield()`](Self::try_shield).
    type TryShieldFuture: Future<Output = Result<Self::Output, Self::TryShieldError>>;
    /// The error returned from [`try_shield()`](Self::try_shield).
    type TryShieldError;

    /// Prevent this future from being aborted by wrapping it in a task.
    ///
    /// `future.shield().await` is equivalent to `future.try_shield().await.unwrap()`.
    ///
    /// # Panics
    /// This function panics if awaiting the spawned task fails.
    fn shield(self) -> Self::ShieldFuture;

    /// Prevent this future from being aborted by wrapping it in a task.
    ///
    /// Since the task is created using [`tokio::spawn()`], execution of this future starts immediately.
    fn try_shield(self) -> Self::TryShieldFuture;
}

impl<T> Shield for T
where
    T: Future + Send + 'static,
    T::Output: Send,
{
    type ShieldFuture = Map<JoinHandle<T::Output>, fn(Result<T::Output, JoinError>) -> T::Output>;
    type TryShieldFuture = JoinHandle<T::Output>;
    type TryShieldError = JoinError;

    #[inline]
    fn shield(self) -> Self::ShieldFuture {
        self.try_shield().map(Result::unwrap)
    }

    #[inline]
    fn try_shield(self) -> Self::TryShieldFuture {
        tokio::spawn(self)
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::{sync::Mutex, time::sleep};

    use super::*;

    #[tokio::test]
    async fn returns_value() {
        let result = async { 42 }.shield().await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn survives_cancel() {
        let x = Arc::new(Mutex::new(false));
        let y = Arc::clone(&x);
        let task = tokio::spawn(
            async move {
                sleep(Duration::from_millis(100)).await;
                *y.lock().await = true;
            }
            .shield(),
        );
        sleep(Duration::from_millis(50)).await;
        task.abort();
        sleep(Duration::from_millis(100)).await;
        assert!(*x.lock().await);
    }

    #[tokio::test]
    async fn inner_panic() {
        async {
            panic!();
        }
        .try_shield()
        .await
        .unwrap_err();
    }
}
