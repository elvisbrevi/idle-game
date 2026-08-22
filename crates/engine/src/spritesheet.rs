//! Grid spritesheets and time-based animation (PLAN.md Fase 6).
//!
//! A [`SpriteSheet`] slices a texture into equal cells laid out row-major;
//! an [`Animation`] walks those frames driven by delta time, never by frame
//! count, so playback speed is independent of FPS.

use glam::Vec2;
use graphics::Rect;

use crate::Texture2D;

/// A texture sliced into equal cells laid out row-major from the top-left
/// corner. Frame rects and UVs are computed from the grid, so games never
/// hardcode pixel coordinates.
#[derive(Clone)]
pub struct SpriteSheet {
    /// Texture holding every frame of the grid.
    pub texture: Texture2D,
    /// Grid columns.
    pub columns: u32,
    /// Grid rows.
    pub rows: u32,
}

impl SpriteSheet {
    /// Slices `texture` into a `columns` x `rows` grid.
    pub fn new(texture: Texture2D, columns: u32, rows: u32) -> Self {
        Self {
            texture,
            columns,
            rows,
        }
    }

    /// Source rectangle in pixels of `frame`, ready for
    /// [`crate::draw_texture_ex`] (`DrawTextureParams::source`). Frames beyond
    /// the grid wrap around.
    pub fn frame_rect(&self, frame: usize) -> Rect {
        frame_rect(self.texture.size(), (self.columns, self.rows), frame)
    }

    /// Normalized UV corners (top-left, bottom-right) of `frame`. Frames
    /// beyond the grid wrap around.
    pub fn frame_uv(&self, frame: usize) -> (Vec2, Vec2) {
        frame_uv(self.texture.size(), (self.columns, self.rows), frame)
    }
}

/// Pixel rect of `frame` inside a sheet of `size` pixels laid out as `grid`
/// (columns, rows), row-major from the top-left corner. Frames beyond the
/// grid wrap around so sampling always stays inside the texture.
pub(crate) fn frame_rect(size: (u32, u32), grid: (u32, u32), frame: usize) -> Rect {
    let (cols, rows) = (grid.0.max(1) as usize, grid.1.max(1) as usize);
    let frame = frame % (cols * rows);
    let (frame_w, frame_h) = (size.0 as f32 / cols as f32, size.1 as f32 / rows as f32);
    Rect::new(
        (frame % cols) as f32 * frame_w,
        (frame / cols) as f32 * frame_h,
        frame_w,
        frame_h,
    )
}

/// Normalized UV corners (top-left, bottom-right) of `frame` inside a sheet
/// of `size` pixels laid out as `grid` (columns, rows), row-major. Frames
/// beyond the grid wrap around.
pub(crate) fn frame_uv(size: (u32, u32), grid: (u32, u32), frame: usize) -> (Vec2, Vec2) {
    let rect = frame_rect(size, grid, frame);
    let (tex_w, tex_h) = (size.0.max(1) as f32, size.1.max(1) as f32);
    (
        Vec2::new(rect.x / tex_w, rect.y / tex_h),
        Vec2::new((rect.x + rect.w) / tex_w, (rect.y + rect.h) / tex_h),
    )
}

/// Time-driven playback over a fixed list of frames.
///
/// `frame`, `fps`, `looping` and `elapsed` are public so games can read and
/// reset them directly; [`Animation::update`] does the advancing.
#[derive(Debug, Clone)]
pub struct Animation {
    /// Number of frames in the playback sequence.
    pub frames: usize,
    /// Active frame index; always inside `0..frames`.
    pub frame: usize,
    /// Frames per second.
    pub fps: f32,
    /// Whether the animation wraps to frame 0 after the last frame.
    pub looping: bool,
    /// Seconds accumulated toward the next frame advance.
    pub elapsed: f32,
}

impl Animation {
    /// Creates a playback of `frames` frames at `fps` frames per second,
    /// starting at frame 0.
    pub fn new(frames: usize, fps: f32, looping: bool) -> Self {
        Self {
            frames,
            frame: 0,
            fps,
            looping,
            elapsed: 0.0,
        }
    }

    /// Advances playback by `delta` seconds: one frame per `1.0 / fps`
    /// elapsed, wrapping if looping or holding the last frame otherwise.
    /// Non-positive or non-finite `fps` freezes playback.
    pub fn update(&mut self, delta: f32) {
        if self.fps <= 0.0 || !self.fps.is_finite() {
            return;
        }
        self.elapsed += delta;
        let frame_time = 1.0 / self.fps;
        while self.elapsed >= frame_time {
            self.elapsed -= frame_time;
            self.advance();
        }
    }

    fn advance(&mut self) {
        let next = self.frame + 1;
        if next < self.frames {
            self.frame = next;
        } else if self.looping {
            self.frame = 0;
        }
        // otherwise hold the last frame
    }

    /// Returns the active frame index.
    pub fn current_frame(&self) -> usize {
        self.frame
    }

    /// Whether a non-looping animation played through its last frame.
    /// Looping animations are never finished.
    pub fn is_finished(&self) -> bool {
        !self.looping && self.frame + 1 >= self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 128x64 sheet, 4 columns x 2 rows -> 32x32 frames
    const SHEET: (u32, u32) = (128, 64);
    const GRID: (u32, u32) = (4, 2);

    #[test]
    fn frame_rect_maps_row_major_frames_to_pixel_rects() {
        assert_eq!(frame_rect(SHEET, GRID, 0), Rect::new(0.0, 0.0, 32.0, 32.0));
        assert_eq!(
            frame_rect(SHEET, GRID, 5),
            Rect::new(32.0, 32.0, 32.0, 32.0)
        );
        assert_eq!(
            frame_rect(SHEET, GRID, 7),
            Rect::new(96.0, 32.0, 32.0, 32.0)
        );
    }

    #[test]
    fn frame_uv_returns_top_left_and_bottom_right_corners() {
        // frame 5 -> column 1 of 4, row 1 of 2
        assert_eq!(
            frame_uv(SHEET, GRID, 5),
            (Vec2::new(0.25, 0.5), Vec2::new(0.5, 1.0))
        );
        assert_eq!(
            frame_uv(SHEET, GRID, 0),
            (Vec2::new(0.0, 0.0), Vec2::new(0.25, 0.5))
        );
    }

    #[test]
    fn update_advances_one_frame_per_frame_duration_of_delta_time() {
        // 3 frames at 10 fps -> each frame lasts 0.1 s
        let mut anim = Animation::new(3, 10.0, true);
        anim.update(0.05);
        assert_eq!(anim.current_frame(), 0);
        anim.update(0.05);
        assert_eq!(anim.current_frame(), 1);
        anim.update(0.15);
        assert_eq!(anim.current_frame(), 2);
    }

    #[test]
    fn looping_animation_wraps_back_to_the_first_frame() {
        // 0.35 s is 3.5 frame durations at 10 fps: three advances land back
        // on frame 0 with half a frame of carry-over
        let mut anim = Animation::new(3, 10.0, true);
        anim.update(0.35);
        assert_eq!(anim.current_frame(), 0);
    }

    #[test]
    fn playback_is_identical_in_one_big_step_or_many_small_ones() {
        // same total delta time must reach the same frame regardless of how
        // many update calls delivered it (framerate independence); 0.75 s at
        // 10 fps is 7.5 frame durations -> 7 advances, safely off a boundary
        let mut coarse = Animation::new(8, 10.0, true);
        coarse.update(0.75);

        let mut fine = Animation::new(8, 10.0, true);
        for _ in 0..45 {
            fine.update(0.75 / 45.0);
        }

        assert_eq!(coarse.current_frame(), 7);
        assert_eq!(fine.current_frame(), coarse.current_frame());
    }

    #[test]
    fn non_looping_animation_holds_and_finishes_at_the_last_frame() {
        // 1.0 s at 5 fps would be 5 advances over a 4-frame sequence
        let mut anim = Animation::new(4, 5.0, false);
        assert!(!anim.is_finished());
        anim.update(1.0);
        assert_eq!(anim.current_frame(), 3);
        assert!(anim.is_finished());

        // extra time keeps it parked on the last frame
        anim.update(1.0);
        assert_eq!(anim.current_frame(), 3);
    }

    #[test]
    fn looping_animations_never_report_finished() {
        let mut anim = Animation::new(4, 5.0, true);
        anim.update(10.0);
        assert!(!anim.is_finished());
    }

    #[test]
    fn non_positive_fps_freezes_playback_instead_of_hanging() {
        // a misconfigured fps must never spin update() forever
        let mut frozen = Animation::new(4, 0.0, true);
        frozen.update(100.0);
        assert_eq!(frozen.current_frame(), 0);

        let mut reversed = Animation::new(4, -5.0, true);
        reversed.update(100.0);
        assert_eq!(reversed.current_frame(), 0);
    }

    #[test]
    fn degenerate_lengths_report_finished_without_playing() {
        // nothing to play means already done; a single frame completes it
        assert!(Animation::new(0, 5.0, false).is_finished());
        assert!(Animation::new(1, 5.0, false).is_finished());
        assert!(!Animation::new(2, 5.0, false).is_finished());
    }
}
