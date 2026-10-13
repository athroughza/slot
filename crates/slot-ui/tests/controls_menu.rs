use slot_store::{Bindings, Control, Key, Profile};
use slot_ui::{
    controls_first, controls_key_faces, controls_label_faces, quick_label_face, ControlsFaces,
    ControlsMenu, Draw, QuickRow, TexId, UndoFace, CONTROLS_SHOWN, MENU_PAD, OUT_W, QUICK_SPLIT,
};

const RESET: usize = Control::ALL.len();
const LISTENING: usize = Key::ALL.len();

fn label_tex(i: usize) -> TexId {
    TexId::from_raw(100 + i)
}

fn key_tex(i: usize, lit: bool) -> TexId {
    TexId::from_raw(200 + 2 * i + lit as usize)
}

fn legend_tex(i: usize) -> TexId {
    TexId::from_raw(300 + i)
}

fn faces() -> ControlsFaces {
    ControlsFaces {
        labels: (0..=RESET).map(|i| (label_tex(i), 120, 40)).collect(),
        keys: (0..=LISTENING + 1)
            .map(|i| [(key_tex(i, false), 60, 40), (key_tex(i, true), 60, 40)])
            .collect(),
        legend: [0, 1, 2, 3, 4].map(|i| (legend_tex(i), 90)),
        gb_title: (TexId::from_raw(400), 150, 40),
    }
}

fn drawn(profile: Profile, bindings: Bindings, at: usize, listening: bool) -> Vec<Draw> {
    let f = faces();
    let mut out = Vec::new();
    ControlsMenu {
        profile,
        bindings,
        at,
        listening,
        faces: Some(&f),
    }
    .draw(&mut out);
    out
}

fn texes(out: &[Draw]) -> Vec<TexId> {
    out.iter()
        .filter_map(|d| match d {
            Draw::Tex { tex, .. } => Some(*tex),
            _ => None,
        })
        .collect()
}

fn bar(out: &[Draw]) -> (f32, f32) {
    match out.get(1) {
        Some(Draw::Rect { y, h, .. }) => (*y, *h),
        other => panic!("the second draw is not the bar: {other:?}"),
    }
}

#[test]
fn the_gb_page_shows_every_row_without_l_or_r() {
    let t = texes(&drawn(Profile::Gb, Bindings::default(), 0, false));
    for c in Control::GB {
        assert!(t.contains(&label_tex(c.index())), "{c:?} is missing");
    }
    assert!(t.contains(&label_tex(RESET)), "Reset is missing");
    for c in [Control::L, Control::R] {
        assert!(
            !t.contains(&label_tex(c.index())),
            "{c:?} is on the GB page"
        );
    }
}

#[test]
fn the_gba_page_scrolls_to_keep_the_bar_in_view() {
    assert_eq!(controls_first(0, 9), 0);
    assert_eq!(controls_first(3, 9), 0);
    assert_eq!(controls_first(5, 9), 2);
    assert_eq!(controls_first(8, 9), 2);
    let t = texes(&drawn(Profile::Gba, Bindings::default(), 8, false));
    assert!(t.contains(&label_tex(RESET)), "Reset is off screen");
    assert!(
        !t.contains(&label_tex(Control::A.index())),
        "A is still drawn below the scrolled page"
    );
    let labels = t
        .iter()
        .filter(|x| (0..=RESET).map(label_tex).any(|l| l == **x))
        .count();
    assert_eq!(labels, CONTROLS_SHOWN);
}

fn legend(t: Vec<TexId>) -> Vec<TexId> {
    t.into_iter()
        .filter(|x| (0..5).map(legend_tex).any(|l| l == *x))
        .collect()
}

#[test]
fn the_bar_never_runs_into_the_legend() {
    for profile in [Profile::Gba, Profile::Gb] {
        for at in 0..=profile.controls().len() {
            let (y, h) = bar(&drawn(profile, Bindings::default(), at, false));
            assert!(
                y >= 0.0 && y + h <= QUICK_SPLIT,
                "{profile:?} row {at}: {y}+{h}"
            );
        }
    }
}

#[test]
fn each_row_shows_its_key_lit_only_under_the_bar() {
    let mut b = Bindings::default();
    b.bind(Control::B, Key::R1);
    let t = texes(&drawn(Profile::Gba, b, 1, false));
    assert!(
        t.contains(&key_tex(5, true)),
        "B's R1 is not lit under the bar"
    );
    assert!(
        t.contains(&key_tex(0, false)),
        "A's key is not dim off the bar"
    );
    assert!(!t.contains(&key_tex(0, true)));
}

#[test]
fn listening_asks_for_a_button_and_offers_only_cancel() {
    let t = texes(&drawn(Profile::Gba, Bindings::default(), 0, true));
    assert!(
        t.contains(&key_tex(LISTENING, true)),
        "no \"Press a button\""
    );
    assert!(!t.contains(&key_tex(0, true)), "the old key still shows");
    assert_eq!(legend(t), [legend_tex(3)]);
}

#[test]
fn the_legend_offers_clear_and_set_on_a_row_and_reset_on_the_reset() {
    let at = |row| legend(texes(&drawn(Profile::Gb, Bindings::default(), row, false)));
    assert_eq!(at(0), [legend_tex(0), legend_tex(4), legend_tex(1)]);
    assert_eq!(at(6), [legend_tex(0), legend_tex(2)]);
}

#[test]
fn the_page_is_set_in_the_quick_menus_type() {
    let quick = quick_label_face(QuickRow::Screen);
    let all: Vec<UndoFace> = controls_label_faces()
        .into_iter()
        .chain(controls_key_faces().into_iter().flatten())
        .collect();
    for f in &all {
        assert_eq!(f.h, quick.h, "a face is not the quick menu's height");
        let first = (0..f.w)
            .find(|&x| (0..f.h).any(|y| f.rgba[((y * f.w + x) * 4 + 3) as usize] > 128))
            .expect("an empty face");
        assert!(
            (MENU_PAD..MENU_PAD + 4).contains(&first),
            "ink starts at {first}"
        );
        assert!(f.w <= OUT_W);
    }
}

#[test]
fn an_unbound_row_reads_none() {
    let mut b = Bindings::default();
    b.unbind(Control::TurboB);
    let t = texes(&drawn(Profile::Gba, b, 7, false));
    assert!(
        t.contains(&key_tex(LISTENING + 1, true)),
        "no \"None\" on the unbound row"
    );
    assert!(
        !t.contains(&key_tex(3, true)),
        "Y still shows on the unbound row"
    );
}
