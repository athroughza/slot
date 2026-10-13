use slot_store::{
    read_slot_state, write_slot_state, Bindings, Control, Key, Platform, Profile, SlotState,
};

const BEFORE_CONTROLS: &str =
    "cart=\ncart_platform=\nbrightness=5\nblue_light=0\nvolume=35\nmuted=0\nclock_set=1\nutc_offset_min=0\n";

fn root() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    d
}

#[test]
fn the_defaults_are_the_buttons_printed_on_the_device() {
    let b = Bindings::default();
    let pairs: Vec<(Control, Key)> = Control::ALL.iter().map(|c| (*c, b.key(*c))).collect();
    assert_eq!(
        pairs,
        [
            (Control::A, Key::A),
            (Control::B, Key::B),
            (Control::L, Key::L1),
            (Control::R, Key::R1),
            (Control::Start, Key::Start),
            (Control::Select, Key::Select),
            (Control::TurboA, Key::X),
            (Control::TurboB, Key::Y),
        ]
    );
}

#[test]
fn each_profile_round_trips_apart_from_the_other() {
    let d = root();
    let mut s = SlotState::default();
    s.controls_gba.bind(Control::A, Key::R1);
    s.controls_gb.bind(Control::B, Key::X);
    write_slot_state(d.path(), &s).unwrap();
    let back = read_slot_state(d.path());
    assert_eq!(back.controls_gba.key(Control::A), Key::R1);
    assert_eq!(back.controls_gba.key(Control::B), Key::B);
    assert_eq!(back.controls_gb.key(Control::B), Key::X);
    assert_eq!(back.controls_gb.key(Control::A), Key::A);
}

#[test]
fn two_controls_may_share_one_key() {
    let d = root();
    let mut s = SlotState::default();
    s.controls_gba.bind(Control::A, Key::X);
    s.controls_gba.bind(Control::B, Key::X);
    write_slot_state(d.path(), &s).unwrap();
    let back = read_slot_state(d.path());
    assert_eq!(
        back.controls_gba.controls_on(Key::X).collect::<Vec<_>>(),
        [Control::A, Control::B, Control::TurboA]
    );
}

#[test]
fn a_state_from_before_controls_gets_the_defaults() {
    let d = root();
    std::fs::write(d.path().join("Config/slot.state"), BEFORE_CONTROLS).unwrap();
    let s = read_slot_state(d.path());
    assert_eq!(s.controls_gba, Bindings::default());
    assert_eq!(s.controls_gb, Bindings::default());
}

#[test]
fn a_binding_slot_cannot_read_keeps_its_default_and_spares_the_rest() {
    let d = root();
    std::fs::write(
        d.path().join("Config/slot.state"),
        format!("{BEFORE_CONTROLS}bind_gba_a=l2\nbind_gba_b=start\nbind_gba_zz=a\nbind_n64_a=b\n"),
    )
    .unwrap();
    let s = read_slot_state(d.path());
    assert_eq!(s.volume, 35, "a bad binding threw away the whole state");
    assert_eq!(s.controls_gba.key(Control::A), Key::A);
    assert_eq!(s.controls_gba.key(Control::B), Key::Start);
}

#[test]
fn gb_and_gbc_share_a_profile_and_gba_has_its_own() {
    assert_eq!(Profile::of(Platform::Gba), Profile::Gba);
    assert_eq!(Profile::of(Platform::Gb), Profile::Gb);
    assert_eq!(Profile::of(Platform::Gbc), Profile::Gb);
    assert!(!Profile::Gb.controls().contains(&Control::L));
    assert!(!Profile::Gb.controls().contains(&Control::R));
    assert_eq!(Profile::Gba.controls().len(), 8);
}
