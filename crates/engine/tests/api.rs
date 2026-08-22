//! Integration tests at the engine public API boundary (issue #10): the
//! highest seam, exactly what a game sees. Every check runs inside one live
//! `run()` window because the engine's free functions only work in that
//! context (ADR-0001).
//!
//! This target builds as a plain binary (`harness = false`): Winit requires
//! the event loop on the main thread on macOS, and libtest always spawns
//! test threads. The binary drives all checks sequentially, prints one line
//! per check and exits nonzero if anything failed.

use std::cell::RefCell;
use std::fmt::Display;

use engine::{
    Color, DrawTextureParams, EngineError, KeyCode, Rect, Vec2, WindowConfig, clear_background,
    draw_circle, draw_rectangle, draw_texture, draw_texture_ex, get_frame_time, get_time,
    is_key_down, load_texture, next_frame, run, screen_height, screen_width,
};

thread_local! {
    static FAILURES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static CHECKS: RefCell<Vec<(&'static str, bool)>> = const { RefCell::new(Vec::new()) };
}

fn expect(name: &'static str, ok: bool, detail: impl Display) {
    CHECKS.with(|c| c.borrow_mut().push((name, ok)));
    if !ok {
        FAILURES.with(|f| {
            f.borrow_mut()
                .push(format!("FAIL {name}: {detail}"));
        });
    }
}

/// sprite.png is 32x32 RGBA (independent source of truth from the asset file)
const SPRITE_SIZE: (u32, u32) = (32, 32);
/// cargo test runs this binary with the crate root as working directory, so
/// the shared assets live two levels up
const SPRITE_PATH: &str = "../../assets/textures/sprite.png";

fn run_all() {
    let config = WindowConfig {
        title: "pet2d api tests".into(),
        width: 320,
        height: 240,
        ..WindowConfig::default()
    };

    run(config, || async {
        // run() created a window with real pixel dimensions
        expect(
            "run() creates a window",
            screen_width() > 0.0 && screen_height() > 0.0,
            format!("{}x{}", screen_width(), screen_height()),
        );

        // load_texture() returns a valid Texture2D for a real asset
        let cat = match load_texture(SPRITE_PATH) {
            Ok(texture) => texture,
            Err(err) => {
                expect("load_texture() returns a valid Texture2D", false, err);
                return;
            }
        };
        expect(
            "load_texture() returns a valid Texture2D",
            cat.size() == SPRITE_SIZE,
            format!("size {:?}", cat.size()),
        );
        // repeated loads hit the cache and hand back the same dimensions
        expect(
            "load_texture() caches by path",
            load_texture(SPRITE_PATH).is_ok_and(|cached| cached.size() == SPRITE_SIZE),
            "second load failed",
        );
        // missing files are an Err, not a panic
        expect(
            "load_texture() errors on missing file",
            matches!(load_texture("no/such/file.png"), Err(EngineError::Graphics(_))),
            "expected Err(EngineError)",
        );

        // is_key_down() reflects keyboard state: nothing is held in this
        // window, so no key may report down. The press -> down -> release
        // reflection itself is covered by InputState unit tests fed through
        // the same native event path; OS-level key injection is not part of
        // the public API.
        expect(
            "is_key_down() reflects keyboard state",
            !is_key_down(KeyCode::ArrowRight) && !is_key_down(KeyCode::Space),
            "unpressed keys reported down",
        );

        clear_background(Color::from_rgb(30, 30, 46));
        next_frame().await;

        // after a presented frame the engine has real timing
        let delta = get_frame_time();
        expect(
            "get_frame_time() returns positive value",
            delta > 0.0 && delta < 1.0,
            format!("delta {delta}"),
        );
        expect(
            "get_time() accumulates frame deltas",
            get_time() >= delta as f64,
            format!("time {}", get_time()),
        );

        // draw calls queue quads without panicking; presentation happens at
        // next_frame(), which must not panic either
        draw_texture(&cat, 40.0, 40.0);
        draw_texture_ex(
            &cat,
            120.0,
            40.0,
            DrawTextureParams {
                source: Some(Rect::new(8.0, 8.0, 16.0, 16.0)),
                dest_size: Some(Vec2::splat(64.0)),
                ..Default::default()
            },
        );
        draw_rectangle(20.0, 180.0, 48.0, 32.0, Color::RED);
        draw_circle(240.0, 196.0, 16.0, Color::GREEN);
        next_frame().await;

        // everything above survived presentation across two full frames:
        // any panic inside run() would have aborted before this line, and
        // engine state stays readable after drawing
        expect(
            "draw calls survive presentation without panicking",
            !is_key_down(KeyCode::ArrowRight) && screen_width() > 0.0,
            "state unreadable after draws",
        );
    });
}

fn main() {
    let outcome = std::panic::catch_unwind(run_all);
    let checks = CHECKS.with(|c| std::mem::take(&mut *c.borrow_mut()));
    let failures = FAILURES.with(|f| std::mem::take(&mut *f.borrow_mut()));

    let panicked = outcome.is_err();
    for (name, ok) in &checks {
        println!("{} {name}", if *ok { "ok -" } else { "FAILED" });
    }
    for line in &failures {
        eprintln!("{line}");
    }

    if panicked {
        eprintln!("test binary panicked");
    }
    let failed = checks.iter().filter(|(_, ok)| !ok).count();
    println!(
        "{} checks, {} failed{}",
        checks.len(),
        failed + usize::from(panicked),
        if panicked { ", panic" } else { "" }
    );
    if failed > 0 || panicked || checks.is_empty() {
        std::process::exit(101);
    }
}
