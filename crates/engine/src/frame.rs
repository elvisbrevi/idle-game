//! Frame pacing (ADR-0003): `next_frame()` is the await point that hands
//! control back to the Winit event loop once per game tick.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

use crate::context;

/// Yields control back to the event loop and resolves when the frame being
/// drawn has been presented:
///
/// ```no_run
/// # use engine::*;
/// # let config = WindowConfig::default();
/// run(config, || async {
///     loop {
///         // update state and queue draws here
///         next_frame().await;
///     }
/// });
/// ```
///
/// Panics if polled outside of [`crate::run`] (ADR-0001).
pub fn next_frame() -> NextFrame {
    NextFrame { target_frame: None }
}

/// Future returned by [`next_frame`]: resolves on the first presented frame
/// after it was created, however many times the executor polls it.
///
/// It never arms a waker: per ADR-0003 the engine re-polls the game future
/// unconditionally on every rendered frame, so readiness is driven purely by
/// the frame counter comparison.
pub struct NextFrame {
    /// Presented-frame count this future resolves on, captured on first poll.
    target_frame: Option<u64>,
}

impl Future for NextFrame {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<()> {
        let this = self.get_mut();
        context::with(|ctx| {
            let target = *this.target_frame.get_or_insert_with(|| ctx.frame + 1);
            if ctx.frame >= target {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn next_frame_panics_outside_of_run_context() {
        use std::pin::pin;
        use std::task::{Context, Waker};

        let mut future = pin!(super::next_frame());
        let mut cx = Context::from_waker(Waker::noop());
        let _ = future.as_mut().poll(&mut cx);
    }
}
