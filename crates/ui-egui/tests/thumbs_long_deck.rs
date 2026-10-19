//! A deck with more slides than the thumbnail cache holds: the panes that show thumbnails must
//! settle instead of re-rendering the whole deck for ever (#969).

use deckcraft_engine::Session;
use deckcraft_ui_egui::{Services, SlideApp};
use egui::vec2;
use egui_kittest::Harness;
use serde_json::json;

/// More slides than the cache's 600, as the report's deck has.
const SLIDES: usize = 900;

fn long_deck() -> Harness<'static, SlideApp> {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    for _ in 0..SLIDES {
        s.execute("slide.new", &json!({"layout": "blank"})).unwrap();
    }
    let app = SlideApp::new(s, Services::default());
    let mut h = Harness::builder().with_size(vec2(1400.0, 900.0)).with_step_dt(1.0 / 60.0).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(3);
    h
}

#[test]
fn the_thumbnail_bar_settles_on_a_deck_longer_than_its_cache() {
    let mut h = long_deck();
    assert!(h.state().session.documents().first().is_some_and(|d| d.doc.slides.len() > 600), "the deck outgrows the cache");

    // Long enough to have rendered the whole deck at the old three-per-frame rate.
    h.run_steps(400);
    let (thumbs, renders) = (h.state().textures.thumb_count(), h.state().textures.thumb_renders);
    assert!(thumbs < 100, "only the rows on screen are cached, not the whole deck: {thumbs}");

    // And it stays settled: nothing re-renders while nothing moves. Rendering every slide used to
    // overrun the cache, evict what had just been drawn and start again, so this kept climbing.
    h.run_steps(200);
    assert_eq!(h.state().textures.thumb_renders, renders, "nothing re-rendered while the view stood still");
    assert_eq!(h.state().textures.thumb_count(), thumbs);
}
