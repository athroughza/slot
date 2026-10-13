use slot_store::{Bindings, Control, Key, Profile};

use crate::draw::{Draw, TexId, OUT_H, OUT_W};
use crate::plate::{centred_hints, hint_face, UndoFace, HINT_H, LEGEND_GAP};
use crate::quick_menu::{
    push, quick_value_face, BAR_INSET, LEGEND_Y, QUICK_EDGE, QUICK_PITCH, TYPE_DROP,
};
use crate::slot_chrome::{edge, opening};

pub const CONTROLS_SHOWN: usize = 7;
pub const RESET_LABEL: &str = "Reset to Default";
pub const LISTENING_TEXT: &str = "Press a button";
pub const UNBOUND_TEXT: &str = "None";
pub const GB_CONTROLS_LABEL: &str = "GB Controls";

pub fn controls_rows(profile: Profile) -> usize {
    profile.controls().len() + 1
}

pub fn controls_first(at: usize, rows: usize) -> usize {
    if rows <= CONTROLS_SHOWN {
        return 0;
    }
    at.saturating_sub(CONTROLS_SHOWN / 2)
        .min(rows - CONTROLS_SHOWN)
}

pub fn controls_label_faces() -> Vec<UndoFace> {
    Control::ALL
        .iter()
        .map(|c| c.label())
        .chain([RESET_LABEL])
        .map(|l| quick_value_face(l, true))
        .collect()
}

pub fn controls_key_faces() -> Vec<[UndoFace; 2]> {
    Key::ALL
        .iter()
        .map(|k| k.name())
        .chain([LISTENING_TEXT, UNBOUND_TEXT])
        .map(|t| [false, true].map(|lit| quick_value_face(t, lit && t != LISTENING_TEXT)))
        .collect()
}

pub fn controls_legend_faces() -> [UndoFace; 5] {
    [
        hint_face("B", "Back"),
        hint_face("A", "Set"),
        hint_face("A", "Reset"),
        hint_face("MENU", "Cancel"),
        hint_face("X", "Clear"),
    ]
}

pub struct ControlsFaces {
    pub labels: Vec<(TexId, u32, u32)>,
    pub keys: Vec<[(TexId, u32, u32); 2]>,
    pub legend: [(TexId, u32); 5],
    pub gb_title: (TexId, u32, u32),
}

pub struct ControlsMenu<'a> {
    pub profile: Profile,
    pub bindings: Bindings,
    pub at: usize,
    pub listening: bool,
    pub faces: Option<&'a ControlsFaces>,
}

impl ControlsMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let controls = self.profile.controls();
        let rows = controls_rows(self.profile);
        let shown = rows.min(CONTROLS_SHOWN);
        let first = controls_first(self.at, rows);
        let top = (OUT_H as f32 - QUICK_PITCH * shown as f32) / 2.0;
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: opening(),
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: top + QUICK_PITCH * (self.at - first) as f32 + BAR_INSET,
            w: OUT_W as f32,
            h: QUICK_PITCH - 2.0 * BAR_INSET,
            colour: edge(),
        });
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (OUT_W as f32 - QUICK_EDGE, crate::MENU_PAD as f32);
        for (shown_at, row) in (first..first + shown).enumerate() {
            let y = top + QUICK_PITCH * shown_at as f32 + TYPE_DROP;
            let lit = row == self.at;
            let control = controls.get(row);
            let label = match control {
                Some(c) => faces.labels.get(c.index()),
                None => faces.labels.get(Control::ALL.len()),
            };
            if let Some(&(tex, w, h)) = label {
                push(out, tex, QUICK_EDGE - pad, y, w, h);
            }
            let Some(control) = control else {
                continue;
            };
            let at = match lit && self.listening {
                true => Key::ALL.len(),
                false => key_at(self.bindings.key(*control)),
            };
            if let Some(v) = faces.keys.get(at) {
                let (tex, w, h) = v[lit as usize];
                push(out, tex, right + pad - w as f32, y, w, h);
            }
        }
        let [back, set, reset, cancel, clear] = faces.legend;
        let hints: &[(TexId, u32)] = match (self.listening, self.at == rows - 1) {
            (true, _) => &[cancel],
            (false, true) => &[back, reset],
            (false, false) => &[back, clear, set],
        };
        for (tex, w, x) in centred_hints(hints, LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y, w, HINT_H);
        }
    }
}

fn key_at(key: Option<Key>) -> usize {
    match key {
        Some(key) => Key::ALL.iter().position(|k| *k == key).unwrap_or(0),
        None => Key::ALL.len() + 1,
    }
}
