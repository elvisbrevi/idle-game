use std::time::Instant;

use crate::context;

/// Per-frame timing state, updated once per rendered frame.
/// The game reads it through the free functions ([`get_frame_time`],
/// [`get_time`], [`get_fps`]); the game-relevant fields are public for
/// direct inspection.
pub struct TimeState {
    /// Seconds elapsed since the previous frame.
    pub delta: f32,
    /// Total seconds elapsed since the engine started.
    pub elapsed: f64,
    /// Estimated frames per second (exponentially smoothed).
    pub fps: f32,
    // engine bookkeeping: mutating it from the game would corrupt delta
    pub(crate) last_frame: Instant,
}

/// Weight of each new sample in the FPS estimate; ~10-frame effective window.
const FPS_SMOOTHING: f32 = 0.1;

impl Default for TimeState {
    fn default() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            fps: 0.0,
            last_frame: Instant::now(),
        }
    }
}

impl TimeState {
    /// Advances timing for this frame; the engine calls it once per rendered
    /// frame before the game reads it.
    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.elapsed += self.delta as f64;
        if self.delta > 0.0 {
            let instant_fps = 1.0 / self.delta;
            // exponential smoothing keeps get_fps() stable even when frame
            // times vary; the first frame seeds it so it never under-reports
            if self.fps == 0.0 {
                self.fps = instant_fps;
            } else {
                self.fps += (instant_fps - self.fps) * FPS_SMOOTHING;
            }
        }
    }
}

/// Returns the delta time of the current frame in seconds; use it to make
/// movement framerate-independent (`x += speed * get_frame_time()`).
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn get_frame_time() -> f32 {
    context::with(|ctx| ctx.time.delta)
}

/// Returns total elapsed seconds since the engine started.
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn get_time() -> f64 {
    context::with(|ctx| ctx.time.elapsed)
}

/// Returns the estimated frames per second (exponentially smoothed).
///
/// Panics if called outside of [`crate::run`] (ADR-0001).
pub fn get_fps() -> f32 {
    context::with(|ctx| ctx.time.fps)
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn update_reports_positive_delta() {
        let mut time = TimeState::default();
        assert_eq!(time.delta, 0.0);

        sleep(std::time::Duration::from_millis(10));
        time.update();
        assert!(time.delta > 0.0);
    }

    #[test]
    fn update_accumulates_elapsed_time() {
        let mut time = TimeState::default();
        sleep(std::time::Duration::from_millis(10));
        time.update();
        let first = time.elapsed;
        assert!(first > 0.0);

        sleep(std::time::Duration::from_millis(10));
        time.update();
        // each frame adds its own delta, so total grows monotonically
        assert!(time.elapsed > first);
        assert!(time.elapsed - first <= 1.0);
    }

    #[test]
    fn update_estimates_finite_positive_fps() {
        let mut time = TimeState::default();
        sleep(std::time::Duration::from_millis(10));
        time.update();
        assert!(time.fps.is_finite());
        assert!(time.fps > 0.0);
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn get_frame_time_panics_outside_of_run_context() {
        super::get_frame_time();
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn get_time_panics_outside_of_run_context() {
        super::get_time();
    }

    #[test]
    #[should_panic(expected = "pet2d: fuera de contexto")]
    fn get_fps_panics_outside_of_run_context() {
        super::get_fps();
    }
}
