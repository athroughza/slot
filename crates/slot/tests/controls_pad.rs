use slot::input::Pad;
use slot_input::{Action, Btn};
use slot_retro::ButtonMask;
use slot_store::{Bindings, Control, Key, Profile};

fn pad(profile: Profile, bindings: Bindings) -> Pad {
    let mut pad = Pad::default();
    pad.bind(profile, bindings);
    pad
}

fn held(pad: &mut Pad, btns: &[Btn]) -> u16 {
    pad.clear();
    for b in btns {
        pad.apply(Action::GbaDown(*b));
    }
    pad.mask().0
}

#[test]
fn the_default_bindings_send_what_slot_always_sent() {
    let mut p = pad(Profile::Gba, Bindings::default());
    for (btn, bit) in [
        (Btn::A, ButtonMask::A),
        (Btn::B, ButtonMask::B),
        (Btn::L1, ButtonMask::L),
        (Btn::R1, ButtonMask::R),
        (Btn::Start, ButtonMask::START),
        (Btn::Select, ButtonMask::SELECT),
        (Btn::X, ButtonMask::X),
        (Btn::Y, ButtonMask::Y),
        (Btn::Up, ButtonMask::UP),
        (Btn::Left, ButtonMask::LEFT),
    ] {
        assert_eq!(held(&mut p, &[btn]), bit, "{btn:?} sent the wrong bit");
    }
}

#[test]
fn a_rebound_key_sends_its_new_control_and_not_its_old_one() {
    let mut b = Bindings::default();
    b.bind(Control::A, Key::R1);
    let mut p = pad(Profile::Gba, b);
    assert_eq!(
        held(&mut p, &[Btn::R1]),
        ButtonMask::A | ButtonMask::R,
        "R1 should press both A and the R it still drives"
    );
    assert_eq!(held(&mut p, &[Btn::A]), 0, "A still pressed the game's A");
}

#[test]
fn one_key_can_press_several_controls() {
    let mut b = Bindings::default();
    b.bind(Control::A, Key::X);
    b.bind(Control::B, Key::X);
    let mut p = pad(Profile::Gba, b);
    assert_eq!(
        held(&mut p, &[Btn::X]),
        ButtonMask::A | ButtonMask::B | ButtonMask::X
    );
}

#[test]
fn turbo_follows_the_key_it_is_bound_to() {
    let mut b = Bindings::default();
    b.bind(Control::TurboA, Key::L1);
    let mut p = pad(Profile::Gba, b);
    let mask = ButtonMask(held(&mut p, &[Btn::L1]));
    assert_eq!(mask.turbo(0).0, ButtonMask::A | ButtonMask::L);
    assert_eq!(mask.turbo(3).0, ButtonMask::L);
    assert_eq!(held(&mut p, &[Btn::X]), 0, "X still fired turbo A");
}

#[test]
fn the_dpad_cannot_be_rebound_away() {
    let mut b = Bindings::default();
    for c in Control::ALL {
        b.bind(c, Key::Start);
    }
    let mut p = pad(Profile::Gba, b);
    assert_eq!(
        held(&mut p, &[Btn::Up, Btn::Right]),
        ButtonMask::UP | ButtonMask::RIGHT
    );
}

#[test]
fn the_gb_profile_never_sends_l_or_r() {
    let mut p = pad(Profile::Gb, Bindings::default());
    assert_eq!(held(&mut p, &[Btn::L1, Btn::R1]), 0);
    let mut b = Bindings::default();
    b.bind(Control::A, Key::L1);
    let mut p = pad(Profile::Gb, b);
    assert_eq!(held(&mut p, &[Btn::L1]), ButtonMask::A);
}

#[test]
fn letting_go_of_a_shared_key_releases_every_control_on_it() {
    let mut b = Bindings::default();
    b.bind(Control::Start, Key::A);
    let mut p = pad(Profile::Gba, b);
    p.apply(Action::GbaDown(Btn::A));
    assert_eq!(p.mask().0, ButtonMask::A | ButtonMask::START);
    p.apply(Action::GbaUp(Btn::A));
    assert_eq!(p.mask().0, 0);
}

#[test]
fn an_unbound_control_is_never_pressed() {
    let mut b = Bindings::default();
    b.unbind(Control::TurboA);
    let mut p = pad(Profile::Gba, b);
    assert_eq!(held(&mut p, &[Btn::X]), 0, "X still fired turbo A");
    assert_eq!(held(&mut p, &[Btn::Y]), ButtonMask::Y);
}
