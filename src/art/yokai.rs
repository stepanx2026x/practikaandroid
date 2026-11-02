//! Ёкаи: милые монстры японского фольклора. Холст 300×300, «пол» на y≈290.

use std::fmt::Write;

use super::chibi::mirror;
use super::*;
use crate::data::{Kind, MonsterDef};

pub const SIZE: f32 = 300.0;
const DARK: u32 = 0x2B1B24;

#[derive(Clone, Copy, PartialEq)]
enum Mood {
    Cute,
    Angry,
    Sleepy,
}

pub fn monster_svg(m: &MonsterDef) -> String {
    let mut s = Svg::new(SIZE, SIZE);
    match m.kind {
        Kind::Kappa => kappa(&mut s, m),
        Kind::Tanuki => tanuki(&mut s, m),
        Kind::Umbrella => umbrella(&mut s, m),
        Kind::Oni => oni(&mut s, m),
        Kind::Cat => cat(&mut s, m),
        Kind::Weasel => weasel(&mut s, m),
        Kind::Tengu => tengu(&mut s, m),
        Kind::Wall => wall(&mut s, m),
        Kind::Ghost => ghost(&mut s, m),
        Kind::Spider => spider(&mut s, m),
        Kind::Lantern => lantern(&mut s, m),
        Kind::Faceless => faceless(&mut s, m),
        Kind::OneEye => one_eye(&mut s, m),
        Kind::Nue => nue(&mut s, m),
        Kind::SnowLady => snow_lady(&mut s, m),
        Kind::SnowChild => snow_child(&mut s, m),
        Kind::Icicle => icicle(&mut s, m),
        Kind::Fireball => fireball(&mut s, m),
        Kind::Wheel => wheel(&mut s, m),
        Kind::Raiju => raiju(&mut s, m),
        Kind::Cloud => cloud(&mut s, m),
        Kind::Dragon => dragon(&mut s, m),
        Kind::Orochi => orochi(&mut s, m),
    }
    s.finish()
}

// ---------------------------------------------------------------------------
// Общие детали
// ---------------------------------------------------------------------------

/// Заливка с объёмом (свет сверху слева) и контуром.
fn shaded(s: &mut Svg, d: &str, c: u32, cx: f32, cy: f32, r: f32) {
    let g = s.rad(
        cx - r * 0.35,
        cy - r * 0.45,
        r * 1.5,
        &[
            (0.0, tint(c, 0.4), 1.0),
            (0.55, c, 1.0),
            (1.0, shade(c, 0.74), 1.0),
        ],
    );
    s.path(d, &g, ink(c), 3.2);
}

fn shaded_ellipse(s: &mut Svg, cx: f32, cy: f32, rx: f32, ry: f32, c: u32) {
    let g = s.rad(
        cx - rx * 0.35,
        cy - ry * 0.45,
        rx.max(ry) * 1.5,
        &[
            (0.0, tint(c, 0.4), 1.0),
            (0.55, c, 1.0),
            (1.0, shade(c, 0.74), 1.0),
        ],
    );
    s.ellipse(cx, cy, rx, ry, &g, Some((ink(c), 3.2)));
}

fn eyes(s: &mut Svg, cx: f32, cy: f32, dx: f32, r: f32, iris: u32, mood: Mood) {
    for side in [-1.0f32, 1.0] {
        let x = cx + side * dx;
        if mood == Mood::Sleepy {
            s.line(
                &format!("M{} {} Q{} {} {} {}", x - r, cy, x, cy + r * 0.7, x + r, cy),
                DARK,
                r * 0.28,
                1.0,
            );
            continue;
        }
        s.ellipse(x, cy, r * 0.9, r, "#ffffff", Some((DARK, 2.6)));
        let g = s.lin(
            0.0,
            cy - r,
            0.0,
            cy + r,
            &[(0.0, shade(iris, 0.35), 1.0), (1.0, tint(iris, 0.35), 1.0)],
        );
        s.ellipse(
            x + side * r * 0.08,
            cy + r * 0.14,
            r * 0.62,
            r * 0.76,
            &g,
            None,
        );
        s.ellipse(
            x + side * r * 0.08,
            cy + r * 0.18,
            r * 0.3,
            r * 0.4,
            &hex(shade(iris, 0.18)),
            None,
        );
        s.circle(x - r * 0.26, cy - r * 0.3, r * 0.25, "#ffffff", None);
        s.circle(x + r * 0.24, cy + r * 0.42, r * 0.11, "#ffffff", None);
        if mood == Mood::Angry {
            let (ox, ix) = (x + side * r * 1.05, x - side * r * 0.75);
            s.line(
                &format!("M{} {} L{} {}", ox, cy - r * 1.45, ix, cy - r * 0.95),
                DARK,
                r * 0.3,
                1.0,
            );
        }
    }
}

fn slit_eyes(s: &mut Svg, cx: f32, cy: f32, dx: f32, r: f32, iris: u32) {
    for side in [-1.0f32, 1.0] {
        let x = cx + side * dx;
        let g = s.rad(x, cy, r, &[(0.0, tint(iris, 0.6), 1.0), (1.0, iris, 1.0)]);
        s.path(
            &format!(
                "M{} {} Q{} {} {} {} Q{} {} {} {} Z",
                x - r,
                cy,
                x,
                cy - r * 1.3,
                x + r,
                cy,
                x,
                cy + r * 1.3,
                x - r,
                cy
            ),
            &g,
            DARK,
            2.4,
        );
        s.ellipse(x, cy, r * 0.18, r * 0.8, &hex(DARK), None);
        s.circle(x - r * 0.35, cy - r * 0.3, r * 0.18, "#ffffff", None);
    }
}

fn one_big_eye(s: &mut Svg, cx: f32, cy: f32, r: f32, iris: u32, angry: bool) {
    s.ellipse(cx, cy, r, r * 1.05, "#ffffff", Some((DARK, 3.0)));
    let g = s.lin(
        0.0,
        cy - r,
        0.0,
        cy + r,
        &[(0.0, shade(iris, 0.35), 1.0), (1.0, tint(iris, 0.35), 1.0)],
    );
    s.circle(cx, cy + r * 0.12, r * 0.62, &g, None);
    s.circle(cx, cy + r * 0.15, r * 0.3, &hex(shade(iris, 0.15)), None);
    s.circle(cx - r * 0.25, cy - r * 0.22, r * 0.22, "#ffffff", None);
    s.circle(cx + r * 0.25, cy + r * 0.4, r * 0.1, "#ffffff", None);
    if angry {
        s.line(
            &format!(
                "M{} {} Q{} {} {} {}",
                cx - r * 1.1,
                cy - r * 1.0,
                cx,
                cy - r * 1.5,
                cx + r * 1.1,
                cy - r * 1.0
            ),
            DARK,
            r * 0.2,
            1.0,
        );
    }
}

fn blush(s: &mut Svg, cx: f32, cy: f32, dx: f32, r: f32) {
    let b = s.blur(r * 0.25);
    for side in [-1.0f32, 1.0] {
        s.raw(&format!(
            r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#ff7a8a" opacity="0.45" filter="{b}"/>"##,
            cx + side * dx,
            cy,
            r,
            r * 0.5
        ));
    }
}

fn smile(s: &mut Svg, cx: f32, cy: f32, w: f32) {
    s.line(
        &format!(
            "M{} {} Q{} {} {} {}",
            cx - w / 2.0,
            cy,
            cx,
            cy + w * 0.45,
            cx + w / 2.0,
            cy
        ),
        DARK,
        3.0,
        1.0,
    );
}

fn cat_mouth(s: &mut Svg, cx: f32, cy: f32, w: f32) {
    s.line(
        &format!(
            "M{} {} Q{} {} {} {} Q{} {} {} {}",
            cx - w / 2.0,
            cy,
            cx - w / 4.0,
            cy + w * 0.35,
            cx,
            cy,
            cx + w / 4.0,
            cy + w * 0.35,
            cx + w / 2.0,
            cy
        ),
        DARK,
        2.6,
        1.0,
    );
}

/// Открытый рот, по желанию с языком.
fn open_mouth(s: &mut Svg, cx: f32, cy: f32, w: f32, tongue: bool) {
    let d = format!(
        "M{} {} Q{} {} {} {} Q{} {} {} {} Z",
        cx - w / 2.0,
        cy,
        cx,
        cy + w * 0.8,
        cx + w / 2.0,
        cy,
        cx,
        cy + w * 0.12,
        cx - w / 2.0,
        cy
    );
    s.path(&d, "#8e2a3c", DARK, 2.4);
    if tongue {
        s.ellipse(cx, cy + w * 0.38, w * 0.22, w * 0.12, "#f48a9b", None);
    }
}

/// Улыбка с клыками.
fn fang_grin(s: &mut Svg, cx: f32, cy: f32, w: f32, up: bool) {
    let d = format!(
        "M{} {} Q{} {} {} {} Q{} {} {} {} Z",
        cx - w / 2.0,
        cy,
        cx,
        cy + w * 0.55,
        cx + w / 2.0,
        cy,
        cx,
        cy + w * 0.1,
        cx - w / 2.0,
        cy
    );
    s.path(&d, "#8e2a3c", DARK, 2.4);
    let fy = if up { cy + w * 0.3 } else { cy + w * 0.04 };
    let tip = if up { -w * 0.2 } else { w * 0.2 };
    for fx in [cx - w * 0.28, cx + w * 0.28] {
        s.path(
            &format!(
                "M{} {} L{} {} L{} {} Z",
                fx - w * 0.08,
                fy,
                fx,
                fy + tip,
                fx + w * 0.08,
                fy
            ),
            "#ffffff",
            DARK,
            1.6,
        );
    }
}

/// Рог: от основания (x, y) под углом (градусы, 0 — вверх).
fn horn(s: &mut Svg, x: f32, y: f32, len: f32, w: f32, angle: f32, c: u32) {
    let d = format!(
        "M{} {} Q{} {} {} {} Q{} {} {} {} Z",
        x - w / 2.0,
        y,
        x - w * 0.3,
        y - len * 0.6,
        x + w * 0.15,
        y - len,
        x + w * 0.45,
        y - len * 0.5,
        x + w / 2.0,
        y
    );
    let g = s.vgrad(y - len, y, tint(c, 0.4), shade(c, 0.85));
    s.open_group(&format!(r#"transform="rotate({angle} {x} {y})""#));
    s.path(&d, &g, ink(c), 2.4);
    s.close_group();
}

fn feet(s: &mut Svg, cx: f32, dx: f32, y: f32, r: f32, c: u32) {
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, cx + side * dx, y, r, r * 0.6, shade(c, 0.9));
    }
}

fn hitodama(s: &mut Svg, x: f32, y: f32, r: f32, c: u32) {
    let b = s.blur(r * 0.5);
    s.raw(&format!(
        r##"<circle cx="{x}" cy="{y}" r="{}" fill="{}" opacity="0.6" filter="{b}"/>"##,
        r * 1.3,
        hex(c)
    ));
    let g = s.rad(
        x,
        y + r * 0.3,
        r * 1.2,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.5, tint(c, 0.4), 1.0),
            (1.0, c, 1.0),
        ],
    );
    s.path(
        &format!(
            "M{} {} C{} {} {} {} {} {} C{} {} {} {} {} {} Z",
            x,
            y - r * 2.2,
            x + r * 0.4,
            y - r * 1.1,
            x + r,
            y - r * 0.6,
            x + r,
            y + r * 0.2,
            x + r,
            y + r * 1.2,
            x - r,
            y + r * 1.2,
            x - r,
            y + r * 0.2
        ),
        &g,
        shade(c, 0.7),
        1.4,
    );
}

fn stripes(s: &mut Svg, clip_d: &str, lines: &str, c: u32, w: f32) {
    let clip = s.clip(clip_d);
    s.open_group(&format!(r#"clip-path="{clip}""#));
    s.line(lines, c, w, 0.9);
    s.close_group();
}

// ---------------------------------------------------------------------------
// Сакуровая роща
// ---------------------------------------------------------------------------

fn kappa(s: &mut Svg, m: &MonsterDef) {
    // панцирь за спиной
    let shell = s.rad(
        150.0,
        180.0,
        110.0,
        &[(0.0, 0xB08A5E, 1.0), (1.0, 0x6D4C30, 1.0)],
    );
    s.ellipse(150.0, 196.0, 102.0, 84.0, &shell, Some((0x4A3220, 3.2)));
    s.line(
        "M58 196 L242 196 M90 136 L118 196 L96 262 M210 136 L182 196 L204 262",
        0x4A3220,
        2.4,
        0.6,
    );
    // руки
    for side in [-1.0f32, 1.0] {
        let x = 150.0 + side * 92.0;
        shaded_ellipse(s, x, 206.0, 18.0, 24.0, m.body);
        s.line(
            &format!(
                "M{} 226 L{} 236 M{} 228 L{} 238",
                x - 6.0,
                x - 9.0,
                x + 6.0,
                x + 9.0
            ),
            ink(m.body),
            2.2,
            1.0,
        );
    }
    feet(s, 150.0, 30.0, 284.0, 22.0, m.body);
    let body = "M74 204 C70 124 230 124 226 204 C228 262 194 288 150 288 C106 288 72 262 74 204 Z";
    shaded(s, body, m.body, 150.0, 180.0, 90.0);
    s.ellipse(150.0, 240.0, 48.0, 36.0, "#f5ebc0", Some((0xC9B57A, 2.0)));
    s.line("M110 232 L190 232 M114 248 L186 248", 0xC9B57A, 1.6, 0.8);
    // волосы-бахрома и тарелочка на макушке
    let mut fringe = String::from("M96 150 ");
    for i in 0..9 {
        let x = 100.0 + i as f32 * 12.5;
        let _ = write!(
            fringe,
            "L{} {} L{} {} ",
            x + 6.0,
            128.0 - (i % 2) as f32 * 4.0,
            x + 12.5,
            146.0
        );
    }
    fringe.push_str("L204 150 Q150 120 96 150 Z");
    s.path(&fringe, &hex(shade(m.body, 0.65)), ink(m.body), 2.2);
    s.ellipse(150.0, 134.0, 34.0, 11.0, "#e3f4ff", Some((0x7FA9C4, 2.4)));
    s.ellipse(146.0, 132.0, 18.0, 4.0, "#ffffff", None);
    eyes(s, 150.0, 178.0, 30.0, 17.0, 0x3E2723, Mood::Cute);
    blush(s, 150.0, 200.0, 50.0, 11.0);
    // клюв
    s.path(
        "M134 202 Q150 196 166 202 Q158 218 150 220 Q142 218 134 202 Z",
        &hex(m.accent),
        ink(m.accent),
        2.2,
    );
    s.line("M136 205 Q150 210 164 205", ink(m.accent), 1.6, 0.8);
}

fn tanuki(s: &mut Svg, m: &MonsterDef) {
    let dark = m.accent;
    // полосатый хвост
    let tail =
        "M206 250 C250 252 272 214 262 176 C254 148 230 146 226 162 C236 182 236 214 204 230 Z";
    shaded(s, tail, m.body, 236.0, 200.0, 50.0);
    stripes(
        s,
        tail,
        "M226 166 L262 174 M232 190 L266 196 M228 212 L262 226",
        dark,
        8.0,
    );
    for x in [104.0, 196.0] {
        shaded_ellipse(s, x, 134.0, 22.0, 20.0, m.body);
        s.ellipse(x, 136.0, 11.0, 10.0, &hex(dark), None);
    }
    feet(s, 150.0, 34.0, 284.0, 22.0, m.body);
    let body = "M80 206 C74 128 226 128 220 206 C226 264 196 290 150 290 C104 290 74 264 80 206 Z";
    shaded(s, body, m.body, 150.0, 180.0, 90.0);
    s.ellipse(
        150.0,
        242.0,
        50.0,
        42.0,
        "#f3e3cf",
        Some((shade(0xF3E3CF, 0.8), 2.0)),
    );
    // маска вокруг глаз
    for side in [-1.0f32, 1.0] {
        s.open_group(&format!(
            r#"transform="rotate({} {} 178)""#,
            side * -14.0,
            150.0 + side * 28.0
        ));
        s.ellipse(150.0 + side * 28.0, 178.0, 27.0, 19.0, &hex(dark), None);
        s.close_group();
    }
    eyes(s, 150.0, 176.0, 28.0, 14.0, 0x6D4C41, Mood::Cute);
    s.ellipse(
        150.0,
        206.0,
        26.0,
        16.0,
        "#f6e7d6",
        Some((shade(0xF6E7D6, 0.8), 1.8)),
    );
    s.ellipse(150.0, 199.0, 8.0, 5.5, &hex(DARK), None);
    cat_mouth(s, 150.0, 209.0, 18.0);
    blush(s, 150.0, 204.0, 48.0, 10.0);
    // листик на макушке
    s.open_group(r#"transform="rotate(-24 150 120)""#);
    s.path(
        "M150 92 C172 104 172 128 150 140 C128 128 128 104 150 92 Z",
        "#7cc36a",
        0x3E7B32,
        2.4,
    );
    s.line("M150 96 L150 138", 0x3E7B32, 1.8, 0.9);
    s.close_group();
}

fn umbrella(s: &mut Svg, m: &MonsterDef) {
    // нога и гэта
    s.line("M150 168 L150 262", 0xF2C9A8, 14.0, 1.0);
    s.line("M150 168 L150 262", 0xD9A07E, 2.0, 0.6);
    s.raw(r##"<rect x="122" y="262" width="56" height="12" rx="4" fill="#8d6e63" stroke="#4e342e" stroke-width="2.4"/><rect x="130" y="274" width="8" height="12" fill="#5d4037"/><rect x="162" y="274" width="8" height="12" fill="#5d4037"/>"##);
    s.line("M136 262 L150 252 L164 262", 0xD32F2F, 3.0, 1.0);
    // купол зонтика
    let canopy = "M150 36 C108 66 66 118 44 170 Q68 180 92 170 Q120 184 150 172 Q180 184 208 170 Q232 180 256 170 C234 118 192 66 150 36 Z";
    let g = s.rad(
        130.0,
        80.0,
        170.0,
        &[
            (0.0, tint(m.body, 0.4), 1.0),
            (0.6, m.body, 1.0),
            (1.0, shade(m.body, 0.7), 1.0),
        ],
    );
    s.path(canopy, &g, ink(m.body), 3.2);
    stripes(
        s,
        canopy,
        "M60 146 Q150 118 240 146",
        tint(m.accent, 0.4),
        12.0,
    );
    s.line(
        "M150 36 L92 170 M150 36 L150 172 M150 36 L208 170 M150 36 L60 160 M150 36 L240 160",
        ink(m.body),
        1.6,
        0.5,
    );
    s.raw(r##"<rect x="142" y="22" width="16" height="18" rx="4" fill="#5d4037" stroke="#3e2723" stroke-width="2"/>"##);
    one_big_eye(s, 150.0, 108.0, 24.0, 0x5E35B1, false);
    // рот и длинный язык
    s.path(
        "M126 142 Q150 156 174 142 Q150 150 126 142 Z",
        "#8e2a3c",
        DARK,
        2.2,
    );
    s.path(
        "M140 146 C138 176 152 196 170 206 C168 186 162 162 160 146 Z",
        "#f48a9b",
        0xC2185B,
        2.2,
    );
    s.line("M150 150 C150 170 156 186 164 198", 0xC2185B, 1.4, 0.7);
    blush(s, 150.0, 132.0, 46.0, 10.0);
}

fn oni(s: &mut Svg, m: &MonsterDef) {
    let id = m.id;
    let hair = match id {
        "ice_oni" => 0xF4FBFF,
        "fire_oni" => 0xFFC107,
        "stone_oni" => 0x5B6770,
        _ => 0x3A2A2A,
    };
    // дубина-канабо
    s.open_group(r#"transform="rotate(16 236 220)""#);
    let club_c = match id {
        "ice_oni" => 0xB3E5FC,
        "fire_oni" => 0x5D4037,
        "stone_oni" => 0x78909C,
        _ => 0x6D4C41,
    };
    if id == "fire_oni" {
        let b = s.blur(6.0);
        s.raw(&format!(r##"<ellipse cx="236" cy="96" rx="26" ry="44" fill="#ff9800" opacity="0.7" filter="{b}"/>"##));
    }
    let g = s.lin(
        222.0,
        0.0,
        252.0,
        0.0,
        &[
            (0.0, tint(club_c, 0.3), 1.0),
            (1.0, shade(club_c, 0.75), 1.0),
        ],
    );
    s.path(
        "M228 286 L230 150 Q236 70 246 70 Q256 70 258 150 L248 286 Z",
        &g,
        ink(club_c),
        3.0,
    );
    let mut studs = String::new();
    for (x, y) in [
        (238.0, 100.0),
        (250.0, 116.0),
        (236.0, 134.0),
        (250.0, 152.0),
        (238.0, 170.0),
    ] {
        let _ = write!(
            studs,
            r##"<circle cx="{x}" cy="{y}" r="4.5" fill="#f2c14e" stroke="#8d6e1a" stroke-width="1.4"/>"##
        );
    }
    s.raw(&studs);
    s.close_group();
    // руки
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 84.0, 214.0, 20.0, 26.0, m.body);
    }
    feet(s, 150.0, 32.0, 284.0, 24.0, m.body);
    let body = "M82 214 C74 138 226 138 218 214 C224 270 194 290 150 290 C106 290 76 270 82 214 Z";
    shaded(s, body, m.body, 150.0, 190.0, 92.0);
    // набедренная повязка из тигровой шкуры
    let cloth = "M84 236 Q150 254 216 236 L214 262 Q150 280 86 262 Z";
    s.path(cloth, "#f2c14e", 0x8D6E1A, 2.4);
    stripes(
        s,
        cloth,
        "M100 236 L106 270 M126 242 L130 276 M150 244 L150 278 M174 242 L170 276 M200 236 L194 270",
        0x3E2723,
        4.0,
    );
    // рога и шевелюра
    let horn_c = if id == "ice_oni" { 0xE1F5FE } else { 0xFFF3D6 };
    if id == "stone_oni" {
        horn(s, 150.0, 140.0, 46.0, 24.0, 0.0, horn_c);
    } else {
        horn(s, 112.0, 146.0, 44.0, 22.0, -24.0, horn_c);
        horn(s, 188.0, 146.0, 44.0, 22.0, 24.0, horn_c);
    }
    let mut mane = String::from("M88 168 ");
    for i in 0..10 {
        let x = 92.0 + i as f32 * 12.4;
        let _ = write!(
            mane,
            "L{} {} L{} {} ",
            x + 6.0,
            136.0 + ((i * 7) % 3) as f32 * 4.0,
            x + 12.4,
            156.0
        );
    }
    mane.push_str("L212 168 Q150 134 88 168 Z");
    if id == "fire_oni" {
        let g = s.vgrad(130.0, 170.0, 0xFFF59D, 0xFF6F00);
        s.path(&mane, &g, 0xBF360C, 2.2);
    } else {
        s.path(&mane, &hex(hair), ink(hair), 2.2);
    }
    eyes(s, 150.0, 190.0, 30.0, 16.0, 0xF9A825, Mood::Angry);
    fang_grin(s, 150.0, 216.0, 40.0, true);
    blush(s, 150.0, 210.0, 52.0, 10.0);
    if id == "stone_oni" {
        s.line(
            "M100 176 L114 196 L106 214 M196 230 L184 248",
            shade(m.body, 0.6),
            2.0,
            0.7,
        );
        s.ellipse(110.0, 160.0, 18.0, 8.0, "#7cb342", Some((0x33691E, 1.6)));
    }
}

// ---------------------------------------------------------------------------
// Бамбуковый лес
// ---------------------------------------------------------------------------

fn cat(s: &mut Svg, m: &MonsterDef) {
    // два хвоста с призрачными огоньками
    for (d, fx, fy) in [
        (
            "M196 252 C248 252 262 196 240 150 C234 138 226 140 228 152 C244 190 230 232 192 236 Z",
            234.0,
            132.0,
        ),
        (
            "M104 252 C52 252 38 196 60 150 C66 138 74 140 72 152 C56 190 70 232 108 236 Z",
            66.0,
            132.0,
        ),
    ] {
        shaded(s, d, m.body, fx, 200.0, 60.0);
        hitodama(s, fx, fy, 11.0, 0x7FD4FF);
    }
    shaded_ellipse(s, 150.0, 240.0, 72.0, 50.0, m.body);
    s.ellipse(150.0, 252.0, 40.0, 30.0, &hex(tint(m.body, 0.25)), None);
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 30.0, 282.0, 20.0, 12.0, tint(m.body, 0.1));
        let ear = "M96 132 L92 70 L138 104 Z";
        let d = if side > 0.0 {
            mirror(ear, 150.0)
        } else {
            ear.to_string()
        };
        shaded(s, &d, m.body, 110.0, 100.0, 30.0);
        let inner = "M102 120 L100 86 L126 106 Z";
        let di = if side > 0.0 {
            mirror(inner, 150.0)
        } else {
            inner.to_string()
        };
        s.fill(&di, "#f8a5b8");
    }
    shaded_ellipse(s, 150.0, 162.0, 66.0, 58.0, m.body);
    slit_eyes(s, 150.0, 160.0, 28.0, 15.0, m.accent);
    s.path("M144 182 L156 182 L150 189 Z", "#f48fb1", DARK, 1.4);
    cat_mouth(s, 150.0, 190.0, 22.0);
    s.line(
        "M96 180 L126 186 M94 194 L126 192 M204 180 L174 186 M206 194 L174 192",
        tint(m.body, 0.55),
        2.0,
        0.9,
    );
    blush(s, 150.0, 186.0, 44.0, 9.0);
}

fn weasel(s: &mut Svg, m: &MonsterDef) {
    // вихри ветра
    s.line(
        "M30 120 Q70 100 90 130 Q100 150 80 156 M230 96 Q270 110 262 140 M40 220 Q20 240 44 252",
        0xB2DFDB,
        4.0,
        0.8,
    );
    let tail = "M196 250 C246 250 270 206 250 160 C266 210 236 244 192 238 Z";
    shaded(s, tail, m.accent, 230.0, 210.0, 50.0);
    // серпы
    for side in [-1.0f32, 1.0] {
        let blade = "M84 196 C40 182 30 132 56 98 C54 140 70 168 96 182 Z";
        let d = if side > 0.0 {
            mirror(blade, 150.0)
        } else {
            blade.to_string()
        };
        let g = s.lin(
            40.0,
            100.0,
            90.0,
            190.0,
            &[(0.0, 0xFFFFFF, 1.0), (1.0, 0x90A4AE, 1.0)],
        );
        s.path(&d, &g, 0x546E7A, 2.6);
        shaded_ellipse(s, 150.0 + side * 60.0, 200.0, 14.0, 18.0, m.body);
    }
    feet(s, 150.0, 24.0, 284.0, 18.0, m.accent);
    let body =
        "M100 214 C94 150 206 150 200 214 C204 268 182 288 150 288 C118 288 96 268 100 214 Z";
    shaded(s, body, m.body, 150.0, 200.0, 80.0);
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 40.0, 116.0, 15.0, 15.0, m.accent);
    }
    shaded_ellipse(s, 150.0, 150.0, 58.0, 48.0, m.body);
    s.path(
        "M104 146 Q150 120 196 146 Q190 116 150 104 Q110 116 104 146 Z",
        &hex(m.accent),
        ink(m.accent),
        2.0,
    );
    eyes(s, 150.0, 148.0, 24.0, 12.0, 0x4E342E, Mood::Cute);
    s.ellipse(150.0, 168.0, 6.0, 4.0, &hex(DARK), None);
    cat_mouth(s, 150.0, 175.0, 16.0);
    blush(s, 150.0, 170.0, 38.0, 8.0);
}

fn tengu(s: &mut Svg, m: &MonsterDef) {
    let great = m.id == "tengu";
    // крылья
    for side in [-1.0f32, 1.0] {
        let wing = "M110 170 C70 120 30 120 14 150 C40 150 44 166 30 186 C54 180 60 196 48 214 C76 204 92 210 98 226 Z";
        let d = if side > 0.0 {
            mirror(wing, 150.0)
        } else {
            wing.to_string()
        };
        let c = if great { 0x37474F } else { 0x263238 };
        shaded(s, &d, c, 150.0 + side * 80.0, 170.0, 70.0);
        let f = "M100 182 L40 160 M98 198 L46 186 M100 212 L58 208";
        let df = if side > 0.0 {
            mirror(f, 150.0)
        } else {
            f.to_string()
        };
        s.line(&df, tint(c, 0.25), 2.0, 0.6);
    }
    feet(s, 150.0, 26.0, 284.0, 20.0, 0x5D4037);
    // одежда ямабуси
    let robe = "M92 210 C88 168 212 168 208 210 C214 266 188 290 150 290 C112 290 86 266 92 210 Z";
    let robe_c = if great { 0xF5F0E6 } else { 0x455A64 };
    shaded(s, robe, robe_c, 150.0, 210.0, 80.0);
    for (i, x) in [118.0, 134.0, 150.0, 166.0, 182.0].iter().enumerate() {
        let c = if i % 2 == 0 { "#ffffff" } else { "#e53935" };
        s.circle(
            *x,
            216.0 + ((i as f32 - 2.0).abs() * 4.0),
            7.0,
            c,
            Some((0x8E1B1B, 1.6)),
        );
    }
    // голова
    if great {
        let mut mane = String::from("M70 170 ");
        for i in 0..12 {
            let a = std::f32::consts::PI * (0.95 + i as f32 / 11.0 * 1.1);
            let (sn, cs) = a.sin_cos();
            let _ = write!(
                mane,
                "L{:.1} {:.1} L{:.1} {:.1} ",
                150.0 + cs * 92.0,
                150.0 + sn * 88.0,
                150.0 + cs * 70.0,
                150.0 + sn * 66.0
            );
        }
        mane.push('Z');
        s.path(&mane, "#ffffff", 0x9E9E9E, 2.4);
    }
    shaded_ellipse(s, 150.0, 150.0, 60.0, 54.0, m.body);
    eyes(s, 150.0, 146.0, 26.0, 14.0, 0xFFC107, Mood::Angry);
    if great {
        // длинный нос
        s.path(
            "M146 156 C170 150 206 150 222 162 C210 172 176 174 146 170 Z",
            &hex(shade(m.body, 0.92)),
            ink(m.body),
            2.6,
        );
        s.line("M150 186 Q160 192 170 186", DARK, 3.0, 1.0);
    } else {
        // клюв ворона
        s.path(
            "M128 160 Q150 152 172 160 L150 196 Z",
            &hex(m.accent),
            ink(m.accent),
            2.6,
        );
        s.line("M132 166 L150 172 L168 166", ink(m.accent), 1.6, 0.7);
    }
    blush(s, 150.0, 166.0, 42.0, 8.0);
    // шапочка токин
    s.path(
        "M132 104 L150 82 L168 104 Q150 110 132 104 Z",
        "#212121",
        0x000000,
        2.0,
    );
    if great {
        // веер из перьев
        s.open_group(r#"transform="rotate(-18 240 214)""#);
        s.path(
            "M240 214 C214 170 214 130 240 110 C266 130 266 170 240 214 Z",
            "#7cb342",
            0x33691E,
            2.4,
        );
        s.line("M240 120 L240 210 M240 140 L226 128 M240 140 L254 128 M240 166 L224 152 M240 166 L256 152", 0x33691E, 1.6, 0.8);
        s.close_group();
    }
}

// ---------------------------------------------------------------------------
// Горный храм
// ---------------------------------------------------------------------------

fn wall(s: &mut Svg, m: &MonsterDef) {
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 96.0, 214.0, 16.0, 20.0, m.body);
    }
    feet(s, 150.0, 40.0, 286.0, 22.0, m.body);
    let d = "M70 92 Q70 70 92 70 L208 70 Q230 70 230 92 L232 268 Q232 284 214 284 L86 284 Q68 284 68 268 Z";
    shaded(s, d, m.body, 150.0, 160.0, 110.0);
    stripes(
        s, d,
        "M68 120 L232 120 M68 170 L232 170 M68 220 L232 220 M68 260 L232 260 M120 70 L120 120 M180 70 L180 120 M100 120 L100 170 M160 120 L160 170 M210 120 L210 170 M130 220 L130 260 M190 220 L190 260 M110 260 L110 284 M170 260 L170 284",
        shade(m.body, 0.75), 2.2,
    );
    // мох сверху
    let mut moss = String::from("M70 92 ");
    for i in 0..9 {
        let x = 74.0 + i as f32 * 18.0;
        let _ = write!(
            moss,
            "Q{} {} {} {} ",
            x + 9.0,
            64.0 - (i % 2) as f32 * 6.0,
            x + 18.0,
            84.0
        );
    }
    moss.push_str("L230 100 Q150 112 70 100 Z");
    s.path(&moss, &hex(m.accent), ink(m.accent), 2.2);
    s.ellipse(150.0, 172.0, 66.0, 44.0, &hex(tint(m.body, 0.25)), None);
    eyes(s, 150.0, 166.0, 30.0, 15.0, 0x5D4037, Mood::Sleepy);
    smile(s, 150.0, 196.0, 22.0);
    blush(s, 150.0, 186.0, 48.0, 11.0);
}

fn ghost(s: &mut Svg, m: &MonsterDef) {
    let fire = m.id == "kaen";
    let b = s.blur(10.0);
    s.raw(&format!(
        r##"<ellipse cx="150" cy="170" rx="96" ry="110" fill="{}" opacity="0.35" filter="{b}"/>"##,
        hex(if fire { 0xFF9800 } else { 0x9FD8FF })
    ));
    if !fire {
        hitodama(s, 62.0, 120.0, 10.0, 0x7FD4FF);
        hitodama(s, 240.0, 150.0, 8.0, 0x7FD4FF);
    }
    let body = "M84 150 C84 70 216 70 216 150 L218 230 C222 252 238 262 252 268 C226 280 204 270 196 258 C186 274 170 282 150 278 C130 282 112 272 104 258 C94 270 74 278 52 268 C66 260 80 248 82 228 Z";
    if fire {
        let g = s.lin(
            0.0,
            60.0,
            0.0,
            280.0,
            &[
                (0.0, 0xFFF59D, 1.0),
                (0.45, m.body, 1.0),
                (1.0, 0xE64A19, 1.0),
            ],
        );
        s.path(body, &g, 0xBF360C, 3.0);
        let mut flames = String::from("M96 112 ");
        for i in 0..6 {
            let x = 100.0 + i as f32 * 20.0;
            let _ = write!(
                flames,
                "Q{} {} {} {} Q{} {} {} {} ",
                x + 4.0,
                70.0,
                x + 10.0,
                46.0 + (i % 2) as f32 * 14.0,
                x + 12.0,
                80.0,
                x + 20.0,
                104.0
            );
        }
        flames.push('Z');
        let g2 = s.vgrad(40.0, 110.0, 0xFFF9C4, 0xFF7043);
        s.path(&flames, &g2, 0xBF360C, 2.0);
    } else {
        let g = s.lin(
            0.0,
            70.0,
            0.0,
            280.0,
            &[
                (0.0, 0xFFFFFF, 0.97),
                (0.6, m.body, 0.95),
                (1.0, shade(m.body, 0.85), 0.85),
            ],
        );
        s.path(body, &g, ink(m.body), 3.0);
        // монашеская накидка и чётки
        s.path(
            "M90 196 Q150 230 210 196 L214 236 Q150 262 88 236 Z",
            &hex(m.accent),
            ink(m.accent),
            2.4,
        );
        let mut beads = String::new();
        for i in 0..11 {
            let t = i as f32 / 10.0;
            let x = 104.0 + t * 92.0;
            let y = 196.0 + (t * std::f32::consts::PI).sin() * 26.0;
            let _ = write!(
                beads,
                r##"<circle cx="{x:.1}" cy="{y:.1}" r="5" fill="#5d4037" stroke="#3e2723" stroke-width="1.2"/>"##
            );
        }
        s.raw(&beads);
    }
    eyes(
        s,
        150.0,
        140.0,
        28.0,
        15.0,
        if fire { 0xD84315 } else { 0x37474F },
        if fire { Mood::Cute } else { Mood::Sleepy },
    );
    if fire {
        open_mouth(s, 150.0, 166.0, 22.0, true);
    } else {
        smile(s, 150.0, 166.0, 18.0);
    }
    blush(s, 150.0, 160.0, 46.0, 10.0);
}

fn spider(s: &mut Svg, m: &MonsterDef) {
    // восемь полосатых лап
    for side in [-1.0f32, 1.0] {
        for i in 0..4 {
            let y0 = 186.0 + i as f32 * 16.0;
            let kx = 150.0 + side * (98.0 + i as f32 * 10.0);
            let ky = y0 - 70.0 + i as f32 * 14.0;
            let fx = 150.0 + side * (126.0 + i as f32 * 4.0);
            let fy = 284.0 - (3 - i) as f32 * 6.0;
            let d = format!(
                "M{} {} L{} {} L{} {}",
                150.0 + side * 40.0,
                y0,
                kx,
                ky,
                fx,
                fy
            );
            s.line(&d, ink(m.body), 15.0, 1.0);
            s.line(&d, m.body, 10.0, 1.0);
            s.raw(&format!(
                r##"<path d="{d}" fill="none" stroke="{}" stroke-width="10" stroke-dasharray="8 10" stroke-linejoin="round"/>"##,
                hex(0xF2C14E)
            ));
        }
    }
    let abd = s.rad(
        150.0,
        220.0,
        90.0,
        &[
            (0.0, tint(m.body, 0.3), 1.0),
            (1.0, shade(m.body, 0.7), 1.0),
        ],
    );
    s.ellipse(150.0, 226.0, 76.0, 58.0, &abd, Some((ink(m.body), 3.2)));
    s.path(
        "M150 186 L170 222 L150 262 L130 222 Z",
        "#f2c14e",
        0x8D6E1A,
        2.0,
    );
    // голова-они
    horn(s, 122.0, 118.0, 34.0, 18.0, -20.0, 0xFFF3D6);
    horn(s, 178.0, 118.0, 34.0, 18.0, 20.0, 0xFFF3D6);
    shaded_ellipse(s, 150.0, 150.0, 56.0, 46.0, tint(m.body, 0.15));
    for (x, y, r) in [
        (120.0, 126.0, 5.0),
        (180.0, 126.0, 5.0),
        (136.0, 120.0, 4.0),
        (164.0, 120.0, 4.0),
    ] {
        s.circle(x, y, r, &hex(m.accent), Some((DARK, 1.2)));
    }
    eyes(s, 150.0, 150.0, 24.0, 13.0, m.accent, Mood::Angry);
    fang_grin(s, 150.0, 172.0, 30.0, false);
}

// ---------------------------------------------------------------------------
// Город фонарей
// ---------------------------------------------------------------------------

fn lantern(s: &mut Svg, m: &MonsterDef) {
    let b = s.blur(16.0);
    s.raw(&format!(r##"<ellipse cx="150" cy="160" rx="110" ry="120" fill="#ffb74d" opacity="0.45" filter="{b}"/>"##));
    s.raw(r##"<rect x="118" y="30" width="64" height="20" rx="6" fill="#3e2723" stroke="#1b0f0b" stroke-width="2.4"/><rect x="118" y="262" width="64" height="18" rx="6" fill="#3e2723" stroke="#1b0f0b" stroke-width="2.4"/>"##);
    let body = "M150 46 C222 46 240 110 240 156 C240 210 216 266 150 266 C84 266 60 210 60 156 C60 110 78 46 150 46 Z";
    let g = s.rad(
        140.0,
        150.0,
        120.0,
        &[
            (0.0, 0xFFF8E1, 1.0),
            (0.5, tint(m.body, 0.25), 1.0),
            (1.0, m.body, 1.0),
        ],
    );
    s.path(body, &g, ink(m.body), 3.2);
    stripes(s, body, "M60 76 Q150 90 240 76 M60 106 Q150 120 240 106 M60 136 Q150 150 240 136 M60 186 Q150 200 240 186 M60 216 Q150 230 240 216 M60 246 Q150 258 240 246", shade(m.body, 0.75), 2.0);
    one_big_eye(s, 150.0, 118.0, 26.0, 0x6D4C41, false);
    // рот-разрыв и язык
    s.path(
        "M86 168 Q150 222 214 168 Q150 190 86 168 Z",
        "#5d1a22",
        DARK,
        2.4,
    );
    s.path(
        "M132 186 C130 222 146 246 168 252 C166 230 160 206 160 188 Z",
        "#f48a9b",
        0xC2185B,
        2.2,
    );
    blush(s, 150.0, 150.0, 58.0, 12.0);
}

fn faceless(s: &mut Svg, m: &MonsterDef) {
    // ноппэрапон: девочка-призрак без лица в кимоно
    let b = s.blur(8.0);
    s.raw(&format!(r##"<ellipse cx="150" cy="276" rx="70" ry="12" fill="#b39ddb" opacity="0.5" filter="{b}"/>"##));
    let kimono = "M104 170 C96 210 90 250 98 278 Q150 290 202 278 C210 250 204 210 196 170 Z";
    let g = s.vgrad(170.0, 284.0, tint(m.accent, 0.2), shade(m.accent, 0.8));
    s.path(kimono, &g, ink(m.accent), 3.0);
    s.path("M134 170 L150 200 L166 170 Z", "#ffffff", 0xBDBDBD, 1.6);
    s.raw(r##"<rect x="104" y="214" width="92" height="20" rx="5" fill="#f8bbd0" stroke="#ad1457" stroke-width="2"/>"##);
    for side in [-1.0f32, 1.0] {
        let sleeve = "M106 176 C84 184 74 210 76 240 Q92 250 110 242 Z";
        let d = if side > 0.0 {
            mirror(sleeve, 150.0)
        } else {
            sleeve.to_string()
        };
        s.path(&d, &g, ink(m.accent), 2.6);
    }
    // волосы-каре
    s.path(
        "M86 124 C82 50 218 50 214 124 L216 176 Q150 184 84 176 Z",
        "#2b2b3a",
        0x111111,
        2.6,
    );
    let face =
        "M104 116 C104 70 196 70 196 116 C196 150 176 170 150 170 C124 170 104 150 104 116 Z";
    let fg = s.rad(
        140.0,
        104.0,
        80.0,
        &[(0.0, 0xFFFFFF, 1.0), (1.0, m.body, 1.0)],
    );
    s.path(face, &fg, 0xD4A08E, 2.4);
    s.path(
        "M100 104 C104 60 196 60 200 104 Q176 90 150 96 Q124 90 100 104 Z",
        "#2b2b3a",
        0x111111,
        2.4,
    );
    blush(s, 150.0, 136.0, 26.0, 11.0);
    // «вопросительная» капля пота
    s.path(
        "M206 92 Q214 108 206 116 Q198 108 206 92 Z",
        "#90caf9",
        0x42A5F5,
        1.6,
    );
}

fn one_eye(s: &mut Svg, m: &MonsterDef) {
    let robe = "M98 196 C92 240 94 270 100 286 L200 286 C206 270 208 240 202 196 Z";
    let g = s.vgrad(196.0, 286.0, tint(m.accent, 0.2), shade(m.accent, 0.8));
    s.path(robe, &g, ink(m.accent), 3.0);
    s.path("M132 196 L150 224 L168 196 Z", "#ffffff", 0xBDBDBD, 1.6);
    s.raw(r##"<rect x="100" y="236" width="100" height="14" rx="5" fill="#ffd54f" stroke="#8d6e1a" stroke-width="2"/>"##);
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 62.0, 236.0, 16.0, 22.0, m.accent);
        shaded_ellipse(s, 150.0 + side * 64.0, 258.0, 9.0, 8.0, m.body);
    }
    // лысая голова
    shaded_ellipse(s, 150.0, 132.0, 72.0, 70.0, m.body);
    one_big_eye(s, 150.0, 124.0, 34.0, 0xC62828, false);
    open_mouth(s, 150.0, 170.0, 26.0, false);
    s.path(
        "M142 176 C140 200 152 212 164 216 C164 202 160 188 158 176 Z",
        "#f48a9b",
        0xC2185B,
        2.0,
    );
    blush(s, 150.0, 160.0, 50.0, 12.0);
}

fn nue(s: &mut Svg, m: &MonsterDef) {
    // тёмная туча под лапами
    let b = s.blur(6.0);
    s.raw(&format!(r##"<g filter="{b}" opacity="0.85"><circle cx="90" cy="276" r="30" fill="#3c3f58"/><circle cx="150" cy="282" r="36" fill="#3c3f58"/><circle cx="210" cy="276" r="30" fill="#3c3f58"/></g>"##));
    // хвост-змея
    s.line("M216 230 C262 220 270 170 244 140", 0x2E7D32, 18.0, 1.0);
    s.line("M216 230 C262 220 270 170 244 140", 0x7CB342, 12.0, 1.0);
    shaded_ellipse(s, 242.0, 132.0, 18.0, 14.0, 0x7CB342);
    s.circle(236.0, 128.0, 3.0, "#ffeb3b", Some((DARK, 1.0)));
    s.line(
        "M254 138 L266 142 M266 142 L272 138 M266 142 L272 146",
        0xD32F2F,
        2.0,
        1.0,
    );
    // тигровое тело
    let body = "M86 232 C82 190 218 190 214 232 C218 270 194 286 150 286 C106 286 82 270 86 232 Z";
    shaded(s, body, m.body, 150.0, 230.0, 80.0);
    stripes(s, body, "M100 204 L112 236 M130 196 L136 230 M170 196 L164 230 M200 204 L188 236 M96 250 L120 262 M204 250 L180 262", 0x3E2723, 6.0);
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 40.0, 282.0, 20.0, 11.0, m.body);
    }
    // морда обезьяны
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 62.0, 140.0, 16.0, 18.0, m.accent);
    }
    shaded_ellipse(s, 150.0, 146.0, 62.0, 56.0, m.accent);
    s.path(
        "M106 150 C104 112 196 112 194 150 C194 184 172 198 150 198 C128 198 106 184 106 150 Z",
        "#f3c9a8",
        0xB07A5A,
        2.2,
    );
    horn(s, 150.0, 96.0, 26.0, 14.0, 0.0, 0xFFF3D6);
    eyes(s, 150.0, 146.0, 22.0, 13.0, 0xFFC107, Mood::Angry);
    fang_grin(s, 150.0, 172.0, 26.0, false);
    blush(s, 150.0, 166.0, 36.0, 8.0);
}

// ---------------------------------------------------------------------------
// Снежный перевал
// ---------------------------------------------------------------------------

fn snow_lady(s: &mut Svg, m: &MonsterDef) {
    let b = s.blur(12.0);
    s.raw(&format!(r##"<ellipse cx="150" cy="190" rx="100" ry="110" fill="#b3e5fc" opacity="0.5" filter="{b}"/>"##));
    // длинные волосы
    s.path(
        "M86 120 C76 190 70 240 84 270 L216 270 C230 240 224 190 214 120 C210 56 90 56 86 120 Z",
        &hex(m.accent),
        0x111827,
        2.6,
    );
    // кимоно, растворяющееся в тумане
    let robe = "M104 170 C96 210 90 240 74 282 Q112 270 130 284 Q150 270 170 284 Q188 270 226 282 C210 240 204 210 196 170 Z";
    let g = s.lin(
        0.0,
        170.0,
        0.0,
        284.0,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.7, m.body, 0.95),
            (1.0, 0xB3E5FC, 0.6),
        ],
    );
    s.path(robe, &g, 0x90A4AE, 2.6);
    s.path("M134 170 L150 204 L166 170 Z", "#e3f2fd", 0x90A4AE, 1.6);
    s.raw(r##"<rect x="106" y="214" width="88" height="16" rx="5" fill="#90caf9" stroke="#1e88e5" stroke-width="2"/>"##);
    let face =
        "M106 118 C106 72 194 72 194 118 C194 152 174 172 150 172 C126 172 106 152 106 118 Z";
    s.path(face, "#f4fbff", 0x9FB8C8, 2.2);
    s.path(
        "M102 110 C104 60 196 60 198 110 Q172 92 150 102 Q128 92 102 110 Z",
        &hex(m.accent),
        0x111827,
        2.4,
    );
    eyes(s, 150.0, 128.0, 22.0, 11.0, 0x4FC3F7, Mood::Cute);
    smile(s, 150.0, 152.0, 14.0);
    blush(s, 150.0, 146.0, 30.0, 8.0);
    for (x, y, r) in [
        (60.0, 100.0, 9.0),
        (238.0, 80.0, 7.0),
        (250.0, 200.0, 10.0),
        (52.0, 220.0, 7.0),
    ] {
        snowflake(s, x, y, r);
    }
}

fn snowflake(s: &mut Svg, x: f32, y: f32, r: f32) {
    let mut d = String::new();
    for i in 0..3 {
        let a = i as f32 * std::f32::consts::PI / 3.0;
        let (sn, cs) = a.sin_cos();
        let _ = write!(
            d,
            "M{:.1} {:.1} L{:.1} {:.1} ",
            x - cs * r,
            y - sn * r,
            x + cs * r,
            y + sn * r
        );
    }
    s.line(&d, 0xFFFFFF, r * 0.25, 1.0);
    s.line(&d, 0x81D4FA, r * 0.1, 0.8);
}

fn snow_child(s: &mut Svg, m: &MonsterDef) {
    // ручки-веточки
    s.line(
        "M90 214 L50 186 M62 194 L54 176 M210 214 L250 186 M238 194 L246 176",
        0x6D4C41,
        5.0,
        1.0,
    );
    for (x, y) in [(48.0, 182.0), (252.0, 182.0)] {
        s.circle(x, y, 10.0, "#e53935", Some((0x8E1B1B, 2.0)));
    }
    shaded_ellipse(s, 150.0, 236.0, 78.0, 56.0, m.body);
    shaded_ellipse(s, 150.0, 150.0, 60.0, 54.0, m.body);
    // красный шарф
    s.path(
        "M94 182 Q150 204 206 182 L204 198 Q150 220 96 198 Z",
        "#e53935",
        0x8E1B1B,
        2.4,
    );
    s.path(
        "M176 196 L190 240 L170 236 L164 200 Z",
        "#e53935",
        0x8E1B1B,
        2.4,
    );
    // соломенная шляпа-каса
    let hat = s.vgrad(70.0, 116.0, 0xE8C27A, m.accent);
    s.path(
        "M62 116 L150 64 L238 116 Q150 128 62 116 Z",
        &hat,
        0x7A5A2B,
        2.6,
    );
    s.line(
        "M150 64 L106 118 M150 64 L194 118 M150 64 L150 122",
        0x7A5A2B,
        1.4,
        0.6,
    );
    eyes(s, 150.0, 150.0, 24.0, 12.0, 0x212121, Mood::Cute);
    s.path("M146 160 L176 168 L146 172 Z", "#ff9800", 0xBF5B00, 1.6);
    smile(s, 150.0, 180.0, 14.0);
    blush(s, 150.0, 168.0, 40.0, 9.0);
    for (x, y) in [(150.0, 226.0), (150.0, 252.0)] {
        s.circle(x, y, 5.0, "#37474f", None);
    }
}

fn icicle(s: &mut Svg, m: &MonsterDef) {
    let b = s.blur(10.0);
    s.raw(&format!(r##"<ellipse cx="150" cy="190" rx="90" ry="96" fill="#b3e5fc" opacity="0.6" filter="{b}"/>"##));
    let body = "M150 54 L198 92 L222 170 L204 260 L150 288 L96 260 L78 170 L102 92 Z";
    let g = s.lin(
        90.0,
        60.0,
        210.0,
        280.0,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.5, m.body, 1.0),
            (1.0, m.accent, 1.0),
        ],
    );
    s.path(body, &g, 0x0288D1, 3.0);
    s.line("M150 54 L150 120 L102 92 M150 120 L198 92 M150 120 L78 170 M150 120 L222 170 M150 220 L96 260 M150 220 L204 260 M150 220 L150 288", 0xE1F5FE, 2.0, 0.8);
    // сосульки на макушке
    for (x, h) in [(112.0, 40.0), (132.0, 56.0), (168.0, 50.0), (188.0, 36.0)] {
        s.path(
            &format!("M{} 100 L{} {} L{} 100 Z", x - 9.0, x, 100.0 - h, x + 9.0),
            "#e1f5fe",
            0x0288D1,
            2.0,
        );
    }
    eyes(s, 150.0, 170.0, 26.0, 14.0, 0x0277BD, Mood::Cute);
    smile(s, 150.0, 198.0, 16.0);
    blush(s, 150.0, 190.0, 44.0, 9.0);
    feet(s, 150.0, 30.0, 284.0, 16.0, 0x81D4FA);
}

// ---------------------------------------------------------------------------
// Огненный вулкан
// ---------------------------------------------------------------------------

fn fireball(s: &mut Svg, m: &MonsterDef) {
    let b = s.blur(16.0);
    s.raw(&format!(
        r##"<circle cx="150" cy="190" r="100" fill="#ff9800" opacity="0.55" filter="{b}"/>"##
    ));
    let flame = "M150 20 C170 70 204 70 214 40 C236 90 254 140 244 190 C236 252 198 284 150 284 C102 284 64 252 56 190 C48 140 70 100 92 70 C100 100 120 104 130 90 C122 64 132 40 150 20 Z";
    let g = s.lin(
        0.0,
        20.0,
        0.0,
        284.0,
        &[
            (0.0, m.accent, 1.0),
            (0.45, m.body, 1.0),
            (1.0, 0xD84315, 1.0),
        ],
    );
    s.path(flame, &g, 0xBF360C, 3.0);
    let inner = "M150 110 C176 140 200 160 196 200 C192 240 174 262 150 262 C126 262 108 240 104 200 C100 160 124 140 150 110 Z";
    let g2 = s.rad(
        150.0,
        214.0,
        70.0,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.5, 0xFFF59D, 1.0),
            (1.0, 0xFFB74D, 1.0),
        ],
    );
    s.path(inner, &g2, 0xFFB74D, 0.0);
    eyes(s, 150.0, 200.0, 24.0, 14.0, 0xBF360C, Mood::Cute);
    open_mouth(s, 150.0, 226.0, 20.0, true);
    blush(s, 150.0, 220.0, 40.0, 9.0);
}

fn wheel(s: &mut Svg, m: &MonsterDef) {
    // пламя вокруг колеса
    let b = s.blur(8.0);
    let mut fl = String::new();
    for i in 0..14 {
        let a = i as f32 * std::f32::consts::TAU / 14.0;
        let (sn, cs) = a.sin_cos();
        let _ = write!(
            fl,
            r##"<ellipse cx="{:.1}" cy="{:.1}" rx="26" ry="38" fill="#ff7043" transform="rotate({:.1} {:.1} {:.1})"/>"##,
            150.0 + cs * 118.0,
            156.0 + sn * 118.0,
            a.to_degrees() + 90.0,
            150.0 + cs * 118.0,
            156.0 + sn * 118.0
        );
    }
    s.raw(&format!(r##"<g filter="{b}" opacity="0.85">{fl}</g>"##));
    let rim = s.rad(
        150.0,
        156.0,
        130.0,
        &[
            (0.75, tint(m.body, 0.3), 1.0),
            (1.0, shade(m.body, 0.7), 1.0),
        ],
    );
    s.circle(150.0, 156.0, 120.0, "none", Some((ink(m.body), 30.0)));
    s.circle(150.0, 156.0, 120.0, "none", Some((m.body, 24.0)));
    let _ = rim;
    let mut spokes = String::new();
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        let (sn, cs) = a.sin_cos();
        let _ = write!(
            spokes,
            "M{:.1} {:.1} L{:.1} {:.1} ",
            150.0 + cs * 60.0,
            156.0 + sn * 60.0,
            150.0 + cs * 110.0,
            156.0 + sn * 110.0
        );
    }
    s.line(&spokes, ink(m.body), 12.0, 1.0);
    s.line(&spokes, tint(m.body, 0.2), 7.0, 1.0);
    // лицо-ступица
    shaded_ellipse(s, 150.0, 156.0, 66.0, 64.0, 0xFFCCBC);
    horn(s, 118.0, 106.0, 30.0, 16.0, -20.0, 0xFFF3D6);
    horn(s, 182.0, 106.0, 30.0, 16.0, 20.0, 0xFFF3D6);
    eyes(s, 150.0, 148.0, 26.0, 14.0, m.accent, Mood::Angry);
    fang_grin(s, 150.0, 178.0, 34.0, true);
}

// ---------------------------------------------------------------------------
// Небесный дворец
// ---------------------------------------------------------------------------

fn raiju(s: &mut Svg, m: &MonsterDef) {
    // хвост-молния
    s.path(
        "M204 236 L252 214 L236 196 L272 170 L246 186 L258 160 L214 212 L228 220 Z",
        &hex(m.accent),
        0x01579B,
        2.4,
    );
    let body = "M84 236 C80 190 220 190 216 236 C220 274 194 288 150 288 C106 288 80 274 84 236 Z";
    shaded(s, body, m.body, 150.0, 232.0, 80.0);
    stripes(
        s,
        body,
        "M108 204 L118 226 L104 236 M192 204 L182 226 L196 236 M150 196 L142 214 L156 220",
        m.accent,
        5.0,
    );
    for side in [-1.0f32, 1.0] {
        shaded_ellipse(s, 150.0 + side * 40.0, 282.0, 20.0, 11.0, m.body);
        let ear = "M100 128 L86 62 L136 104 Z";
        let d = if side > 0.0 {
            mirror(ear, 150.0)
        } else {
            ear.to_string()
        };
        shaded(s, &d, m.body, 110.0, 100.0, 30.0);
    }
    // пушистая грива
    let mut mane = String::from("M88 170 ");
    for i in 0..10 {
        let x = 92.0 + i as f32 * 12.4;
        let _ = write!(mane, "L{} {} L{} {} ", x + 6.0, 206.0, x + 12.4, 186.0);
    }
    mane.push_str("L212 170 Z");
    s.path(&mane, "#ffffff", 0xB0BEC5, 2.0);
    shaded_ellipse(s, 150.0, 150.0, 62.0, 54.0, m.body);
    eyes(s, 150.0, 146.0, 26.0, 14.0, m.accent, Mood::Cute);
    s.path("M144 168 L156 168 L150 175 Z", "#5d4037", DARK, 1.4);
    cat_mouth(s, 150.0, 176.0, 20.0);
    blush(s, 150.0, 170.0, 42.0, 9.0);
    s.line("M70 110 L84 124 L76 132 L92 148", m.accent, 4.0, 0.9);
    s.line("M230 104 L216 120 L226 126 L210 144", m.accent, 4.0, 0.9);
}

fn cloud(s: &mut Svg, m: &MonsterDef) {
    // капли дождя
    for (x, y) in [(110.0, 262.0), (150.0, 276.0), (190.0, 260.0)] {
        s.path(
            &format!(
                "M{x} {} Q{} {} {x} {} Q{} {} {x} {} Z",
                y - 14.0,
                x + 7.0,
                y,
                y + 6.0,
                x - 7.0,
                y,
                y - 14.0
            ),
            "#90caf9",
            0x42A5F5,
            1.6,
        );
    }
    let body = "M70 210 C40 210 36 166 66 158 C60 120 104 100 128 120 C140 86 196 86 204 126 C232 112 266 140 252 172 C278 186 264 226 234 222 C226 246 190 250 172 234 C156 252 116 252 104 230 C92 240 70 234 70 210 Z";
    let g = s.rad(
        140.0,
        140.0,
        140.0,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.7, m.body, 1.0),
            (1.0, m.accent, 1.0),
        ],
    );
    s.path(body, &g, 0x90A4AE, 3.0);
    eyes(s, 150.0, 172.0, 28.0, 14.0, 0x1E88E5, Mood::Sleepy);
    smile(s, 150.0, 194.0, 16.0);
    blush(s, 150.0, 186.0, 46.0, 11.0);
}

fn dragon(s: &mut Svg, m: &MonsterDef) {
    // змеиное тело кольцами
    let body = "M70 270 C40 240 70 200 120 214 C170 228 210 204 200 166 C192 136 160 130 150 150";
    s.line(body, ink(m.body), 46.0, 1.0);
    s.line(body, m.body, 40.0, 1.0);
    s.line(body, tint(m.body, 0.5), 14.0, 0.7);
    s.raw(&format!(r##"<path d="{body}" fill="none" stroke="{}" stroke-width="40" stroke-dasharray="3 14" opacity="0.5"/>"##, hex(shade(m.body, 0.8))));
    // грива
    s.path("M150 150 C110 120 112 82 138 66 C134 90 146 100 160 104 C164 80 186 68 204 74 C188 92 194 112 210 124 C192 136 172 148 150 150 Z", &hex(m.accent), ink(m.accent), 2.4);
    horn(s, 156.0, 80.0, 34.0, 12.0, -16.0, 0xFFF3D6);
    horn(s, 186.0, 84.0, 34.0, 12.0, 20.0, 0xFFF3D6);
    shaded_ellipse(s, 170.0, 116.0, 50.0, 40.0, m.body);
    s.path(
        "M196 120 C222 114 236 126 232 140 C220 146 202 144 190 138 Z",
        &hex(tint(m.body, 0.2)),
        ink(m.body),
        2.4,
    );
    s.line(
        "M226 124 C250 112 266 120 276 108 M224 136 C248 140 262 154 280 150",
        m.accent,
        3.0,
        1.0,
    );
    eyes(s, 168.0, 110.0, 18.0, 11.0, 0xD84315, Mood::Cute);
    blush(s, 168.0, 128.0, 30.0, 7.0);
    s.circle(226.0, 128.0, 2.6, &hex(DARK), None);
}

fn orochi(s: &mut Svg, m: &MonsterDef) {
    let heads = [
        (54.0, 156.0, -30.0),
        (96.0, 110.0, -16.0),
        (150.0, 88.0, 0.0),
        (204.0, 110.0, 16.0),
        (246.0, 156.0, 30.0),
    ];
    // шеи
    for (x, y, _) in heads {
        let bx = 150.0 + (x - 150.0) * 0.35;
        let d = format!(
            "M{bx} 282 C{bx} 232 {} {} {x} {}",
            x + (x - 150.0) * 0.15,
            y + 70.0,
            y + 14.0
        );
        s.line(&d, ink(m.body), 30.0, 1.0);
        s.line(&d, m.body, 24.0, 1.0);
        s.line(&d, m.accent, 8.0, 0.8);
    }
    // общее тело
    let base = "M54 290 C54 230 246 230 246 290 Z";
    shaded(s, base, m.body, 150.0, 262.0, 100.0);
    s.raw(&format!(r##"<path d="M70 288 C80 250 220 250 230 288" fill="none" stroke="{}" stroke-width="10" opacity="0.6"/>"##, hex(m.accent)));
    // головы
    for (x, y, a) in heads {
        s.open_group(&format!(r#"transform="rotate({a} {x} {y})""#));
        horn(s, x - 12.0, y - 14.0, 20.0, 8.0, -16.0, 0xFFF3D6);
        horn(s, x + 12.0, y - 14.0, 20.0, 8.0, 16.0, 0xFFF3D6);
        shaded_ellipse(s, x, y, 26.0, 22.0, m.body);
        s.path(
            &format!(
                "M{} {} Q{} {} {} {} Q{} {} {} {} Z",
                x - 17.0,
                y + 8.0,
                x,
                y + 30.0,
                x + 17.0,
                y + 8.0,
                x,
                y + 15.0,
                x - 17.0,
                y + 8.0
            ),
            &hex(tint(m.body, 0.25)),
            ink(m.body),
            2.0,
        );
        for side in [-1.0f32, 1.0] {
            s.ellipse(
                x + side * 10.5,
                y - 3.0,
                6.0,
                7.0,
                "#ffeb3b",
                Some((DARK, 1.6)),
            );
            s.ellipse(x + side * 10.5, y - 2.0, 1.8, 5.5, &hex(DARK), None);
        }
        s.line(
            &format!(
                "M{} {} L{} {} M{} {} L{} {}",
                x - 3.0,
                y + 24.0,
                x - 3.0,
                y + 34.0,
                x + 3.0,
                y + 24.0,
                x + 3.0,
                y + 34.0
            ),
            0xD32F2F,
            2.2,
            1.0,
        );
        s.close_group();
    }
}
