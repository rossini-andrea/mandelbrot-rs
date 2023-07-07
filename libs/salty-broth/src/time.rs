use tokio::{
    select,
    time,
    time::Duration,
};
use tokio_util::sync::CancellationToken;

/// Repeatedly calls a function based on a given `tokio::time::Duration`
/// Stops when dropped.
pub struct Ticker {
    cancellation: CancellationToken,
}

impl Ticker {
    pub fn looping<F: Fn() + Send + 'static>(duration: Duration, f: F) -> Self {
        let cancellation = CancellationToken::new();
        tokio::spawn({
            let cancellation = cancellation.clone();
            async move {
                Self::timer_thread(duration, cancellation, f)
            }
        });
        Self { cancellation }
    }

    async fn timer_thread<F: Fn()>(duration: Duration, cancellation: CancellationToken, fun: F) {
        'looping: loop {
            let timer = time::sleep(duration);
            select!(
                _ = timer => fun(),
                _ = cancellation.cancelled() => break 'looping,
            );
        }
    }
    
    pub fn once<F: FnOnce() + Send + 'static>(duration: Duration, f: F) -> Self {
        let cancellation = CancellationToken::new();
        tokio::spawn({
            let cancellation = cancellation.clone();
            async move {
                let timer = time::sleep(duration);
                select!(
                    _ = timer => f(),
                    _ = cancellation.cancelled() => {},
                );
            }
        });
        Self { cancellation }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}
