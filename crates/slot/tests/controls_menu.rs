mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use slot::app::{App, Phase};
use slot::session::Session;
use slot::video_mode::{video_mode_for, VideoMode};
use slot_input::{Action, Btn, RawEvent};
use slot_store::{read_slot_state, write_slot_state, Bindings, Control, Key, Profile, SlotState};
use slot_ui::{QuickRow, Toast};
use tempfile::TempDir;

fn on_shelf(d: TempDir) -> (TempDir, App) {
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .expect("write slot.state");
    let (a, _clock) = common::app_booting_at(d.path(), common::CLOCK_IS_SET);
    assert!(
        matches!(a.phase(), Phase::Shelf),
        "not on the carousel: {:?}",
        a.phase()
    );
    (d, a)
}

fn on_gba_shelf() -> (TempDir, App) {
    on_shelf(common::tmp_root_with_carts(&["Emerald", "Fusion"]))
}

fn on_gb_shelf() -> (TempDir, App) {
    on_shelf(common::tmp_root_with_gb_carts(&["Tetris", "Zzz"]))
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn open_controls(a: &mut App) {
    a.apply(Action::QuickMenu);
    while a.quick_menu() != Some(QuickRow::Controls) {
        press(a, Btn::Down);
        assert_ne!(
            a.quick_menu(),
            Some(QuickRow::MAIN[0]),
            "Controls is not on the main page"
        );
    }
    press(a, Btn::A);
}

fn at(a: &App) -> (Profile, usize, bool) {
    a.controls_menu()
        .unwrap_or_else(|| panic!("not on the Controls page: {:?}", a.phase()))
}

fn down_to(a: &mut App, row: usize) {
    while at(a).1 != row {
        press(a, Btn::Down);
    }
}

#[test]
fn controls_sits_between_gameplay_and_date_and_time() {
    assert_eq!(
        QuickRow::MAIN,
        [
            QuickRow::Screen,
            QuickRow::Game,
            QuickRow::Controls,
            QuickRow::DateTime,
            QuickRow::About
        ]
    );
}

#[test]
fn the_shelf_you_are_on_picks_the_profile() {
    let (_d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    assert_eq!(at(&a), (Profile::Gba, 0, false));
    let (_d, mut a) = on_gb_shelf();
    open_controls(&mut a);
    assert_eq!(at(&a), (Profile::Gb, 0, false));
}

#[test]
fn up_and_down_wrap_through_the_rows_and_the_reset() {
    let (_d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    press(&mut a, Btn::Up);
    assert_eq!(at(&a).1, 8, "up from the top did not land on Reset");
    press(&mut a, Btn::Down);
    assert_eq!(at(&a).1, 0);
}

#[test]
fn a_listens_and_the_next_key_is_bound_and_saved() {
    let (d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    a.apply(Action::GbaDown(Btn::A));
    assert!(at(&a).2, "A did not start listening");
    a.apply(Action::GbaUp(Btn::A));
    assert!(at(&a).2, "letting go of the A that started it bound A");
    press(&mut a, Btn::R1);
    assert!(!at(&a).2, "binding did not stop the listening");
    assert_eq!(a.controls(Profile::Gba).key(Control::A), Key::R1);
    assert_eq!(
        read_slot_state(d.path()).controls_gba.key(Control::A),
        Key::R1,
        "the binding never reached the card"
    );
}

#[test]
fn a_and_b_can_be_bound_while_listening() {
    let (_d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    down_to(&mut a, 1);
    press(&mut a, Btn::A);
    press(&mut a, Btn::A);
    assert_eq!(a.controls(Profile::Gba).key(Control::B), Key::A);
    press(&mut a, Btn::A);
    press(&mut a, Btn::B);
    assert_eq!(a.controls(Profile::Gba).key(Control::B), Key::B);
    assert!(
        a.controls_menu().is_some(),
        "B left the page while listening"
    );
}

#[test]
fn menu_cancels_listening_and_changes_nothing() {
    let (_d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    press(&mut a, Btn::A);
    a.apply(Action::QuickMenu);
    assert_eq!(at(&a), (Profile::Gba, 0, false));
    assert_eq!(*a.controls(Profile::Gba), Bindings::default());
}

#[test]
fn the_dpad_and_reserved_keys_do_not_bind() {
    let (_d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    press(&mut a, Btn::A);
    for btn in [Btn::Down, Btn::Up, Btn::L2, Btn::R2, Btn::Menu, Btn::VolUp] {
        press(&mut a, btn);
        assert!(at(&a).2, "{btn:?} ended the listening");
    }
    assert_eq!(at(&a).1, 0, "the bar moved while listening");
    assert_eq!(*a.controls(Profile::Gba), Bindings::default());
}

#[test]
fn reset_restores_the_defaults_and_says_so() {
    let (d, mut a) = on_gba_shelf();
    open_controls(&mut a);
    press(&mut a, Btn::A);
    press(&mut a, Btn::X);
    press(&mut a, Btn::Up);
    press(&mut a, Btn::A);
    assert_eq!(*a.controls(Profile::Gba), Bindings::default());
    assert_eq!(a.toast(), Some(Toast::ControlsReset));
    assert_eq!(read_slot_state(d.path()).controls_gba, Bindings::default());
}

#[test]
fn b_goes_back_to_the_controls_row() {
    for back in [Action::GbaDown(Btn::B), Action::QuickMenu] {
        let (_d, mut a) = on_gba_shelf();
        open_controls(&mut a);
        a.apply(back);
        assert_eq!(a.quick_menu(), Some(QuickRow::Controls), "{back:?}");
    }
}

#[test]
fn the_gb_profile_has_no_shoulder_rows_and_leaves_gba_alone() {
    let (_d, mut a) = on_gb_shelf();
    open_controls(&mut a);
    press(&mut a, Btn::Up);
    assert_eq!(
        at(&a).1,
        6,
        "the GB page is not A, B, Start, Select, two turbos and Reset"
    );
    down_to(&mut a, 0);
    press(&mut a, Btn::A);
    press(&mut a, Btn::Y);
    assert_eq!(a.controls(Profile::Gb).key(Control::A), Key::Y);
    assert_eq!(*a.controls(Profile::Gba), Bindings::default());
}

fn session_playing(root: &Path) -> Session {
    common::clocked(root);
    let mut s = Session::boot(root.to_path_buf());
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    s
}

#[test]
fn a_gb_game_keeps_both_picture_shortcuts_by_default() {
    let d = common::tmp_root_with_gb_carts(&["Tetris"]);
    let s = session_playing(d.path());
    assert_eq!(s.app().taken_buttons(), [Btn::L1, Btn::R1]);
}

#[test]
fn a_shoulder_bound_in_the_gb_profile_goes_to_the_game() {
    let d = common::tmp_root_with_gb_carts(&["Tetris"]);
    let mut state = SlotState::default();
    state.controls_gb.bind(Control::A, Key::L1);
    write_slot_state(d.path(), &state).unwrap();
    let mut s = session_playing(d.path());
    assert_eq!(s.app().taken_buttons(), [Btn::R1]);
    s.feed([RawEvent::Down(Btn::L1)], 100);
    s.feed([RawEvent::Up(Btn::L1)], 120);
    assert_eq!(
        video_mode_for(d.path(), "Tetris"),
        VideoMode::Actual,
        "L1 still changed the picture"
    );
    s.feed([RawEvent::Down(Btn::R1)], 200);
    s.feed([RawEvent::Up(Btn::R1)], 220);
    s.feed([RawEvent::Down(Btn::L1)], 300);
    assert_eq!(video_mode_for(d.path(), "Tetris"), VideoMode::Actual);
}
