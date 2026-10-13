use slot_input::{Action, Btn};
use slot_retro::ButtonMask;
use slot_store::{Bindings, Control, Key, Profile};

pub struct Pad {
    held: u32,
    profile: Profile,
    bindings: Bindings,
}

impl Default for Pad {
    fn default() -> Self {
        Pad {
            held: 0,
            profile: Profile::Gba,
            bindings: Bindings::default(),
        }
    }
}

impl Pad {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::GbaDown(b) => self.held |= held_bit(b),
            Action::GbaUp(b) => self.held &= !held_bit(b),
            _ => {}
        }
    }

    pub fn bind(&mut self, profile: Profile, bindings: Bindings) {
        self.profile = profile;
        self.bindings = bindings;
    }

    pub fn mask(&self) -> ButtonMask {
        let mut mask = DPAD
            .iter()
            .filter(|(btn, _)| self.holds(*btn))
            .fold(0, |m, (_, bit)| m | bit);
        for &control in self.profile.controls() {
            if self.holds(key_btn(self.bindings.key(control))) {
                mask |= control_bit(control);
            }
        }
        ButtonMask(mask)
    }

    pub fn clear(&mut self) {
        self.held = 0;
    }

    fn holds(&self, btn: Btn) -> bool {
        self.held & held_bit(btn) != 0
    }
}

const DPAD: [(Btn, u16); 4] = [
    (Btn::Up, ButtonMask::UP),
    (Btn::Down, ButtonMask::DOWN),
    (Btn::Left, ButtonMask::LEFT),
    (Btn::Right, ButtonMask::RIGHT),
];

fn held_bit(btn: Btn) -> u32 {
    1 << btn as u32
}

pub fn key_btn(key: Key) -> Btn {
    match key {
        Key::A => Btn::A,
        Key::B => Btn::B,
        Key::X => Btn::X,
        Key::Y => Btn::Y,
        Key::L1 => Btn::L1,
        Key::R1 => Btn::R1,
        Key::Start => Btn::Start,
        Key::Select => Btn::Select,
    }
}

pub fn btn_key(btn: Btn) -> Option<Key> {
    Key::ALL.into_iter().find(|k| key_btn(*k) == btn)
}

fn control_bit(control: Control) -> u16 {
    match control {
        Control::A => ButtonMask::A,
        Control::B => ButtonMask::B,
        Control::L => ButtonMask::L,
        Control::R => ButtonMask::R,
        Control::Start => ButtonMask::START,
        Control::Select => ButtonMask::SELECT,
        Control::TurboA => ButtonMask::X,
        Control::TurboB => ButtonMask::Y,
    }
}
