use crate::platform::Platform;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Key {
    A,
    B,
    X,
    Y,
    L1,
    R1,
    Start,
    Select,
}

impl Key {
    pub const ALL: [Key; 8] = [
        Key::A,
        Key::B,
        Key::X,
        Key::Y,
        Key::L1,
        Key::R1,
        Key::Start,
        Key::Select,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Key::A => "A",
            Key::B => "B",
            Key::X => "X",
            Key::Y => "Y",
            Key::L1 => "L1",
            Key::R1 => "R1",
            Key::Start => "Start",
            Key::Select => "Select",
        }
    }

    fn parse(value: &str) -> Option<Key> {
        Key::ALL
            .into_iter()
            .find(|k| value.eq_ignore_ascii_case(k.name()))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Control {
    A,
    B,
    L,
    R,
    Start,
    Select,
    TurboA,
    TurboB,
}

impl Control {
    pub const ALL: [Control; 8] = [
        Control::A,
        Control::B,
        Control::L,
        Control::R,
        Control::Start,
        Control::Select,
        Control::TurboA,
        Control::TurboB,
    ];

    pub const GB: [Control; 6] = [
        Control::A,
        Control::B,
        Control::Start,
        Control::Select,
        Control::TurboA,
        Control::TurboB,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Control::A => "A",
            Control::B => "B",
            Control::L => "L",
            Control::R => "R",
            Control::Start => "Start",
            Control::Select => "Select",
            Control::TurboA => "Turbo A",
            Control::TurboB => "Turbo B",
        }
    }

    fn key_name(self) -> &'static str {
        match self {
            Control::A => "a",
            Control::B => "b",
            Control::L => "l",
            Control::R => "r",
            Control::Start => "start",
            Control::Select => "select",
            Control::TurboA => "turbo_a",
            Control::TurboB => "turbo_b",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Profile {
    Gba,
    Gb,
}

impl Profile {
    pub fn of(platform: Platform) -> Profile {
        match platform {
            Platform::Gba => Profile::Gba,
            Platform::Gb | Platform::Gbc => Profile::Gb,
        }
    }

    pub fn controls(self) -> &'static [Control] {
        match self {
            Profile::Gba => &Control::ALL,
            Profile::Gb => &Control::GB,
        }
    }

    fn key_name(self) -> &'static str {
        match self {
            Profile::Gba => "gba",
            Profile::Gb => "gb",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Bindings([Option<Key>; Control::ALL.len()]);

impl Default for Bindings {
    fn default() -> Self {
        Bindings(
            [
                Key::A,
                Key::B,
                Key::L1,
                Key::R1,
                Key::Start,
                Key::Select,
                Key::X,
                Key::Y,
            ]
            .map(Some),
        )
    }
}

impl Bindings {
    pub fn key(&self, control: Control) -> Option<Key> {
        self.0[control.index()]
    }

    pub fn bind(&mut self, control: Control, key: Key) {
        self.0[control.index()] = Some(key);
    }

    pub fn unbind(&mut self, control: Control) {
        self.0[control.index()] = None;
    }

    pub fn controls_on(&self, key: Key) -> impl Iterator<Item = Control> + '_ {
        Control::ALL
            .into_iter()
            .filter(move |c| self.key(*c) == Some(key))
    }
}

pub(crate) fn bindings_lines(profile: Profile, b: &Bindings) -> String {
    Control::ALL
        .iter()
        .map(|c| {
            format!(
                "bind_{}_{}={}\n",
                profile.key_name(),
                c.key_name(),
                b.key(*c).map_or("none", Key::name).to_ascii_lowercase()
            )
        })
        .collect()
}

pub(crate) fn bind_line(key: &str, value: &str) -> Option<(Profile, Control, Option<Key>)> {
    let rest = key.strip_prefix("bind_")?;
    let (profile, control) = [Profile::Gba, Profile::Gb]
        .into_iter()
        .find_map(|p| Some((p, rest.strip_prefix(p.key_name())?.strip_prefix('_')?)))?;
    let control = Control::ALL.into_iter().find(|c| c.key_name() == control)?;
    let key = match value.eq_ignore_ascii_case("none") {
        true => None,
        false => Some(Key::parse(value)?),
    };
    Some((profile, control, key))
}
