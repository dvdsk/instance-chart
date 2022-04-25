use tokio::task;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Spawn a new tokio Task and cancel it on drop.
pub fn spawn<T>(future: T) -> Wrapper<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    Wrapper(task::spawn(future))
}

/// Cancels the wrapped tokio Task on Drop.
pub struct Wrapper<T>(task::JoinHandle<T>);

impl<T> Future for Wrapper<T>{
    type Output = Result<T, task::JoinError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.0) }.poll(cx)
    }
}

impl<T> Drop for Wrapper<T> {
    fn drop(&mut self) {
        // do `let _ = self.0.cancel()` for `async_std::task::Task`
        self.0.abort();
    }
}
