//! Героини в стиле чиби: SVG собирается из слоёв (волосы, кимоно, лицо, аксессуары).

use std::fmt::Write;

use super::*;
use crate::data::{Hair, Look, Weapon};

/// Размер холста героини.
pub const W: f32 = 240.0;
pub const H: f32 = 330.0;
/// Где рука с оружием (для отдельного спрайта катаны Сакуры).
pub const HAND: (f32, f32) = (186.0, 246.0);
/// Область портрета (x, y, размер).
pub const PORTRAIT: (f32, f32, f32) = (30.0, 4.0, 180.0);

const SKIN: u32 = 0xFFE9DD;
const SKIN_SHADOW: u32 = 0xF7CDBB;
const SKIN_LINE: u32 = 0xD4927E;
const LASH: u32 = 0x3A2230;

/// Отражает абсолютный путь (M/L/C/Q/Z) по горизонтали относительно x = axis.
pub fn mirror(d: &str, axis: f32) -> String {
    let mut out = String::with_capacity(d.len() + 16);
    let mut is_x = true;
    let mut num = String::new();
    let flush = |num: &mut String, out: &mut String, is_x: &mut bool| {
        if num.is_empty() {
            return;
        }
        if let Ok(v) = num.parse::<f32>() {
            let v = if *is_x { 2.0 * axis - v } else { v };
            let _ = write!(out, "{:.2} ", v);
            *is_x = !*is_x;
        }
        num.clear();
    };
    for ch in d.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
        } else if ch == '-' {
            flush(&mut num, &mut out, &mut is_x);
            num.push(ch);
        } else if ch.is_ascii_alphabetic() {
            flush(&mut num, &mut out, &mut is_x);
            out.push(ch);
            out.push(' ');
            is_x = true;
        } else {
            flush(&mut num, &mut out, &mut is_x);
        }
    }
    flush(&mut num, &mut out, &mut is_x);
    out
}

struct Pal {
    hair: u32,
    hair_light: u32,
    hair_dark: u32,
    hair_line: u32,
}

impl Pal {
    fn new(look: &Look) -> Self {
        Pal {
            hair: look.hair,
            hair_light: tint(look.hair, 0.3),
            hair_dark: shade(look.hair, 0.72),
            hair_line: ink(look.hair),
        }
    }
}

/// Полная героиня. `with_weapon`: рисовать ли оружие в руке (у Сакуры оно отдельным спрайтом).
pub fn heroine(look: &Look, blink: bool, with_weapon: bool) -> Svg {
    let mut s = Svg::new(W, H);
    let p = Pal::new(look);
    back_layer(&mut s, look, &p);
    body(&mut s, look, with_weapon);
    head(&mut s, look, &p, blink);
    s
}

pub fn heroine_svg(look: &Look, blink: bool, with_weapon: bool) -> String {
    heroine(look, blink, with_weapon).finish()
}

pub fn portrait_svg(look: &Look) -> String {
    let (x, y, size) = PORTRAIT;
    heroine(look, false, false).finish_view(x, y, size, size)
}

// ---------------------------------------------------------------------------
// Задний план: длинные волосы, хвост лисы, шарф, нимб
// ---------------------------------------------------------------------------

fn back_layer(s: &mut Svg, look: &Look, p: &Pal) {
    if look.ex.halo {
        let glow = s.rad(
            120.0,
            110.0,
            130.0,
            &[
                (0.0, 0xFFF3B0, 0.9),
                (0.55, 0xFFD54F, 0.35),
                (1.0, 0xFFD54F, 0.0),
            ],
        );
        s.circle(120.0, 110.0, 130.0, &glow, None);
        let mut rays = String::new();
        for i in 0..16 {
            let a = i as f32 * std::f32::consts::TAU / 16.0;
            let (sn, cs) = a.sin_cos();
            let _ = write!(
                rays,
                "M{:.1} {:.1} L{:.1} {:.1} ",
                120.0 + cs * 70.0,
                104.0 + sn * 70.0,
                120.0 + cs * 122.0,
                104.0 + sn * 122.0
            );
        }
        s.line(&rays, 0xFFE082, 5.0, 0.55);
    }
    if look.ex.fox {
        let g = s.lin(
            200.0,
            268.0,
            214.0,
            132.0,
            &[
                (0.0, p.hair, 1.0),
                (0.62, tint(p.hair, 0.15), 1.0),
                (0.8, 0xFFFFFF, 1.0),
                (1.0, 0xFFFFFF, 1.0),
            ],
        );
        s.path("M146 262 C194 266 232 232 228 178 C226 150 210 128 194 134 C208 158 210 196 184 216 C170 228 152 236 144 244 Z", &g, p.hair_line, 2.6);
        s.line(
            "M188 226 C200 214 208 196 206 176",
            shade(p.hair, 0.8),
            2.0,
            0.6,
        );
    }
    if look.ex.scarf {
        let g = s.lin(
            20.0,
            0.0,
            110.0,
            0.0,
            &[(0.0, shade(look.obi, 0.8), 1.0), (1.0, look.obi, 1.0)],
        );
        s.path(
            "M104 196 C76 194 52 176 22 186 C34 196 40 204 26 218 C54 214 78 220 108 210 Z",
            &g,
            ink(look.obi),
            2.4,
        );
    }
    if look.ex.headband {
        s.path(
            "M184 86 C206 90 226 104 234 126 C222 120 210 112 196 108 Z",
            "#ffffff",
            0xB0A8A8,
            2.0,
        );
        s.path(
            "M186 92 C206 102 216 122 216 146 C208 132 198 120 188 110 Z",
            "#ffffff",
            0xB0A8A8,
            2.0,
        );
    }
    if look.ex.hood {
        let g = s.vgrad(30.0, 210.0, tint(look.top, 0.25), shade(look.top, 0.85));
        s.path("M30 136 C18 30 222 30 210 136 C214 176 192 202 168 206 L72 206 C48 202 26 176 30 136 Z", &g, ink(look.top), 2.6);
        return;
    }

    let g = s.vgrad(40.0, 272.0, p.hair, p.hair_dark);
    match look.style {
        Hair::Long => {
            s.path(
                "M50 104 C40 150 40 210 46 266 Q58 254 70 268 Q82 256 94 270 L146 270 Q158 256 170 268 Q182 254 194 266 C200 210 200 150 190 104 C186 40 54 40 50 104 Z",
                &g, p.hair_line, 2.6,
            );
            s.line(
                "M64 150 C62 190 64 230 70 262 M176 150 C178 190 176 230 170 262",
                shade(p.hair, 0.62),
                2.0,
                0.5,
            );
        }
        Hair::Hime => {
            s.path("M50 104 C42 150 42 210 46 270 L194 270 C198 210 198 150 190 104 C186 40 54 40 50 104 Z", &g, p.hair_line, 2.6);
            s.line(
                "M66 150 L64 266 M174 150 L176 266",
                shade(p.hair, 0.62),
                2.0,
                0.5,
            );
        }
        Hair::Pony => {
            let tail = s.lin(
                190.0,
                50.0,
                226.0,
                240.0,
                &[(0.0, p.hair, 1.0), (1.0, p.hair_dark, 1.0)],
            );
            s.path("M166 52 C212 40 238 82 230 132 C224 172 216 202 226 240 C200 228 188 194 190 152 C192 112 184 82 160 72 Z", &tail, p.hair_line, 2.6);
            s.line(
                "M206 80 C216 110 212 150 208 190",
                shade(p.hair, 0.62),
                2.0,
                0.5,
            );
            s.path("M52 108 C46 150 56 178 78 188 L162 188 C184 178 194 150 188 108 C184 44 56 44 52 108 Z", &g, p.hair_line, 2.6);
        }
        Hair::Twin => {
            let left = "M62 60 C24 70 12 122 20 172 C26 212 16 238 30 266 C48 240 52 204 48 164 C44 124 54 94 72 80 Z";
            let tail = s.lin(
                30.0,
                60.0,
                40.0,
                266.0,
                &[(0.0, p.hair, 1.0), (1.0, p.hair_dark, 1.0)],
            );
            s.path(left, &tail, p.hair_line, 2.6);
            s.path(&mirror(left, 120.0), &tail, p.hair_line, 2.6);
            s.line(
                "M36 110 C30 150 34 200 32 240",
                shade(p.hair, 0.62),
                2.0,
                0.5,
            );
            s.line(
                &mirror("M36 110 C30 150 34 200 32 240", 120.0),
                shade(p.hair, 0.62),
                2.0,
                0.5,
            );
            s.path("M52 108 C46 150 56 176 78 186 L162 186 C184 176 194 150 188 108 C184 44 56 44 52 108 Z", &g, p.hair_line, 2.6);
        }
        Hair::Bob => {
            s.path(
                "M50 108 C44 150 50 182 66 196 Q90 188 104 196 L136 196 Q150 188 174 196 C190 182 196 150 190 108 C186 44 54 44 50 108 Z",
                &g, p.hair_line, 2.6,
            );
        }
        Hair::Buns => {
            s.path("M52 108 C48 146 56 170 74 180 L166 180 C184 170 192 146 188 108 C184 44 56 44 52 108 Z", &g, p.hair_line, 2.6);
            for cx in [70.0, 170.0] {
                let bun = s.rad(
                    cx - 6.0,
                    36.0,
                    30.0,
                    &[(0.0, p.hair_light, 1.0), (1.0, p.hair_dark, 1.0)],
                );
                s.circle(cx, 42.0, 25.0, &bun, Some((p.hair_line, 2.6)));
                s.line(
                    &format!(
                        "M{} 30 C{} 22 {} 34 {} 44",
                        cx - 12.0,
                        cx + 2.0,
                        cx + 14.0,
                        cx + 4.0
                    ),
                    shade(p.hair, 0.62),
                    2.0,
                    0.6,
                );
            }
        }
    }
    if look.ex.ribbon {
        // большой бант мико на затылке
        let red = s.vgrad(60.0, 120.0, 0xF05454, 0xC62828);
        s.path(
            "M188 86 C214 64 232 80 222 100 C214 112 198 104 188 94 Z",
            &red,
            0x8E1B1B,
            2.4,
        );
        s.path(
            "M188 94 C206 108 214 130 202 138 C192 142 186 120 186 100 Z",
            &red,
            0x8E1B1B,
            2.4,
        );
    }
}

// ---------------------------------------------------------------------------
// Тело: ноги, хакама, кимоно, рукава, пояс-оби
// ---------------------------------------------------------------------------

fn body(s: &mut Svg, look: &Look, with_weapon: bool) {
    // Обувь (дзори) и носки-таби
    for x in [104.0, 136.0] {
        s.raw(&format!(
            r##"<rect x="{}" y="319" width="26" height="7" rx="3.5" fill="#6d4c41" stroke="#3e2723" stroke-width="1.6"/>"##,
            x - 13.0
        ));
        s.ellipse(x, 316.0, 11.0, 7.0, "#ffffff", Some((0xB9A9A9, 1.8)));
        s.line(&format!("M{} 312 L{} 320", x, x), 0xD32F2F, 2.4, 0.9);
    }

    // Нижняя часть: хакама / подол
    let skirt = "M84 244 L156 244 C164 270 172 296 178 314 Q120 324 62 314 C68 296 76 270 84 244 Z";
    let g = s.vgrad(244.0, 320.0, tint(look.lower, 0.1), shade(look.lower, 0.82));
    s.path(skirt, &g, ink(look.lower), 2.6);
    s.line(
        "M104 250 L98 316 M120 250 L120 319 M136 250 L142 316",
        shade(look.lower, 0.7),
        1.8,
        0.65,
    );
    s.line("M66 305 Q120 315 174 305", tint(look.lower, 0.5), 3.0, 0.7);
    if look.ex.pattern {
        let clip = s.clip(skirt);
        s.open_group(&format!(r#"clip-path="{clip}""#));
        for (x, y, r, rot) in [
            (78.0, 300.0, 7.0, 0.2),
            (106.0, 290.0, 5.0, 1.0),
            (150.0, 302.0, 7.0, 0.6),
            (170.0, 284.0, 5.0, 1.4),
            (126.0, 306.0, 4.5, 0.1),
        ] {
            blossom(
                s,
                x,
                y,
                r,
                tint(look.lower, 0.75),
                shade(look.lower, 0.7),
                rot,
            );
        }
        s.close_group();
    }

    // Шея
    s.raw(&format!(
        r#"<rect x="110" y="166" width="20" height="26" rx="6" fill="{}"/>"#,
        hex(SKIN_SHADOW)
    ));

    // Верх кимоно
    let top = "M94 182 C82 184 74 194 74 208 L80 250 L160 250 L166 208 C166 194 158 184 146 182 Z";
    let g = s.vgrad(182.0, 250.0, tint(look.top, 0.12), shade(look.top, 0.88));
    s.path(top, &g, ink(look.top), 2.6);
    // ворот: белый нижний слой и отвороты
    s.path("M104 182 L120 214 L136 182 Z", "#ffffff", 0xC9B8B8, 1.6);
    s.line(
        "M100 182 L120 222 M140 182 L118 212",
        shade(look.top, 0.75),
        4.0,
        0.9,
    );
    if look.ex.pattern {
        let clip = s.clip(top);
        s.open_group(&format!(r#"clip-path="{clip}""#));
        for (x, y, r, rot) in [
            (88.0, 200.0, 6.0, 0.4),
            (150.0, 196.0, 5.5, 1.1),
            (98.0, 240.0, 5.0, 0.8),
            (146.0, 236.0, 6.0, 0.2),
        ] {
            blossom(s, x, y, r, tint(look.top, 0.7), shade(look.top, 0.72), rot);
        }
        s.close_group();
    }
    // тень от головы
    let b = s.blur(4.0);
    s.raw(&format!(r##"<ellipse cx="120" cy="188" rx="30" ry="7" fill="#5a2a3a" opacity="0.18" filter="{b}"/>"##));

    // Левый рукав (свисает)
    let left_sleeve = "M80 188 C62 192 52 206 50 228 L48 262 Q64 276 86 268 L94 226 Z";
    let g = s.vgrad(188.0, 272.0, tint(look.top, 0.15), shade(look.top, 0.8));
    s.path(left_sleeve, &g, ink(look.top), 2.6);
    s.ellipse(68.0, 266.0, 15.0, 5.0, &hex(shade(look.top, 0.6)), None);
    if look.ex.pattern {
        let clip = s.clip(left_sleeve);
        s.open_group(&format!(r#"clip-path="{clip}""#));
        blossom(
            s,
            66.0,
            236.0,
            7.0,
            tint(look.top, 0.7),
            shade(look.top, 0.72),
            0.7,
        );
        blossom(
            s,
            80.0,
            256.0,
            5.0,
            tint(look.top, 0.7),
            shade(look.top, 0.72),
            0.1,
        );
        s.close_group();
    }
    s.ellipse(90.0, 262.0, 7.5, 6.5, &hex(SKIN), Some((SKIN_LINE, 1.8)));

    // Пояс-оби
    let g = s.vgrad(226.0, 252.0, tint(look.obi, 0.18), shade(look.obi, 0.82));
    s.raw(&format!(
        r#"<rect x="78" y="226" width="84" height="26" rx="6" fill="{g}" stroke="{}" stroke-width="2.4"/>"#,
        hex(ink(look.obi))
    ));
    s.line("M80 239 L160 239", tint(look.obi, 0.6), 2.4, 0.9);
    s.circle(
        120.0,
        239.0,
        5.0,
        &hex(tint(look.obi, 0.75)),
        Some((ink(look.obi), 1.4)),
    );

    // Правый рукав (рука вперёд, держит оружие)
    let right_sleeve = "M160 188 C178 190 192 202 196 222 L204 246 Q192 262 172 256 L158 226 Z";
    let g = s.vgrad(188.0, 260.0, tint(look.top, 0.15), shade(look.top, 0.8));
    s.path(right_sleeve, &g, ink(look.top), 2.6);
    if look.ex.pattern {
        let clip = s.clip(right_sleeve);
        s.open_group(&format!(r#"clip-path="{clip}""#));
        blossom(
            s,
            178.0,
            214.0,
            6.0,
            tint(look.top, 0.7),
            shade(look.top, 0.72),
            0.3,
        );
        s.close_group();
    }
    if with_weapon {
        weapon(s, look, HAND.0, HAND.1, -0.42);
        s.ellipse(HAND.0, HAND.1, 9.0, 8.0, &hex(SKIN), Some((SKIN_LINE, 1.8)));
        s.line(
            &format!(
                "M{} {} L{} {}",
                HAND.0 - 4.0,
                HAND.1 - 3.0,
                HAND.0 + 4.0,
                HAND.1 - 3.0
            ),
            SKIN_LINE,
            1.2,
            0.7,
        );
    }
}

// ---------------------------------------------------------------------------
// Голова: лицо, глаза, чёлка, аксессуары
// ---------------------------------------------------------------------------

fn head(s: &mut Svg, look: &Look, p: &Pal, blink: bool) {
    // Лисьи ушки — под чёлкой, чтобы основания прятались в волосах
    if look.ex.fox {
        let left = "M70 66 L52 8 L106 40 Z";
        let inner = "M72 56 L62 22 L96 42 Z";
        for flip in [false, true] {
            let (o, i) = if flip {
                (mirror(left, 120.0), mirror(inner, 120.0))
            } else {
                (left.to_string(), inner.to_string())
            };
            s.path(&o, &hex(p.hair), p.hair_line, 2.6);
            s.fill(&i, "#ffe0e6");
        }
    }

    // Лицо
    let face = "M56 108 C56 60 184 60 184 108 C184 148 156 176 120 178 C84 176 56 148 56 108 Z";
    let g = s.vgrad(70.0, 178.0, 0xFFF3EA, SKIN);
    s.path(face, &g, SKIN_LINE, 2.2);

    // Румянец
    let b = s.blur(3.0);
    s.raw(&format!(r##"<ellipse cx="82" cy="151" rx="13" ry="6.5" fill="#ff7a8a" opacity="0.42" filter="{b}"/>"##));
    s.raw(&format!(r##"<ellipse cx="158" cy="151" rx="13" ry="6.5" fill="#ff7a8a" opacity="0.42" filter="{b}"/>"##));
    s.line("M76 152 L79 147 M82 153 L85 148 M88 152 L91 147 M152 152 L155 147 M158 153 L161 148 M164 152 L167 147", 0xF06272, 1.3, 0.45);

    // Глаза
    if blink {
        for (cx, side) in [(88.0, -1.0), (152.0, 1.0)] {
            s.line(
                &format!("M{} 130 Q{} 140 {} 130", cx - 15.0, cx, cx + 15.0),
                LASH,
                3.6,
                1.0,
            );
            s.line(
                &format!("M{} 131 L{} 127", cx + side * 15.0, cx + side * 20.0),
                LASH,
                2.6,
                1.0,
            );
        }
    } else {
        eye(s, 88.0, 128.0, look.eyes, -1.0);
        eye(s, 152.0, 128.0, look.eyes, 1.0);
    }

    // Нос и рот
    s.line("M119.5 145 L121 147.5", SKIN_LINE, 1.6, 0.6);
    if !look.ex.scarf {
        s.path(
            "M112 154 Q120 166 128 154 Q120 157 112 154 Z",
            "#c25468",
            0x8E3448,
            1.4,
        );
        s.ellipse(120.0, 160.0, 3.6, 2.0, "#f28b9b", None);
    } else {
        let g = s.vgrad(140.0, 186.0, tint(look.obi, 0.1), shade(look.obi, 0.8));
        s.path(
            "M54 146 C72 138 168 138 186 146 L182 170 C160 184 80 184 58 170 Z",
            &g,
            ink(look.obi),
            2.4,
        );
        s.line(
            "M64 158 C96 166 144 166 176 158",
            shade(look.obi, 0.7),
            1.6,
            0.6,
        );
    }

    // Чёлка
    let g = s.lin(
        0.0,
        24.0,
        0.0,
        140.0,
        &[
            (0.0, p.hair_light, 1.0),
            (0.55, p.hair, 1.0),
            (1.0, shade(p.hair, 0.86), 1.0),
        ],
    );
    if look.style == Hair::Hime {
        s.path(
            "M46 130 C34 66 80 24 120 24 C160 24 206 66 194 130 L186 132 C184 120 182 110 178 104 Q150 112 120 110 Q90 112 62 104 C58 110 56 120 54 132 Z",
            &g, p.hair_line, 2.6,
        );
        let lock = "M50 100 C46 130 44 166 46 196 L68 196 C66 166 66 130 66 106 Z";
        s.path(lock, &g, p.hair_line, 2.6);
        s.path(&mirror(lock, 120.0), &g, p.hair_line, 2.6);
    } else {
        s.path(
            "M46 132 C34 66 80 24 120 24 C160 24 206 66 194 132 Q192 104 182 88 Q178 104 170 120 Q166 100 158 84 Q154 102 146 114 Q140 94 132 80 Q130 98 122 114 Q114 96 108 80 Q100 98 94 114 Q88 100 84 84 Q76 102 70 120 Q62 104 58 88 Q50 104 46 132 Z",
            &g, p.hair_line, 2.6,
        );
        let lock = "M52 96 C42 128 38 166 46 198 C54 184 60 160 64 134 C66 118 62 104 58 96 Z";
        s.path(lock, &g, p.hair_line, 2.6);
        s.path(&mirror(lock, 120.0), &g, p.hair_line, 2.6);
    }
    // Блик на волосах
    s.raw(&format!(
        r#"<path d="M66 64 C92 46 148 46 174 64" fill="none" stroke="{}" stroke-width="6" stroke-linecap="round" stroke-dasharray="16 7" opacity="0.6"/>"#,
        hex(tint(p.hair, 0.6))
    ));
    s.line(
        "M100 34 C96 50 94 62 96 74 M142 34 C146 50 148 62 146 74",
        shade(p.hair, 0.75),
        1.6,
        0.45,
    );

    // Аксессуары
    if look.ex.headband {
        s.path(
            "M48 92 Q120 60 192 92 L190 102 Q120 72 50 102 Z",
            "#ffffff",
            0xB0A8A8,
            2.0,
        );
        s.circle(120.0, 78.0, 6.0, "#e53935", None);
    }
    if look.ex.hood {
        // меховая оторочка капюшона
        let mut fluff = String::new();
        for (r, col, dy) in [(13.0, "#dfe9f2", 2.5), (11.5, "#ffffff", 0.0)] {
            for i in 0..22 {
                let t = i as f32 / 21.0;
                let a = std::f32::consts::PI * (1.0 + t);
                let (sn, cs) = a.sin_cos();
                let wob = if i % 2 == 0 { 2.0 } else { -1.5 };
                let _ = write!(
                    fluff,
                    r##"<circle cx="{:.1}" cy="{:.1}" r="{}" fill="{}"/>"##,
                    120.0 + cs * (74.0 + wob),
                    128.0 + sn * (80.0 + wob) + dy,
                    r,
                    col
                );
            }
        }
        s.raw(&fluff);
    }
    if look.ex.kabuto {
        let iron = s.vgrad(36.0, 108.0, 0x5B6B75, 0x263238);
        s.path("M44 104 C40 30 200 30 196 104 Z", &iron, 0x161E22, 2.6);
        s.raw(r##"<rect x="38" y="92" width="164" height="16" rx="6" fill="#2b3438" stroke="#161e22" stroke-width="2"/>"##);
        for side in [false, true] {
            let flap = "M40 104 L26 138 Q46 146 62 134 L60 108 Z";
            let d = if side {
                mirror(flap, 120.0)
            } else {
                flap.to_string()
            };
            s.path(&d, "#b71c1c", 0x5d0f0f, 2.2);
        }
        s.line("M30 124 L58 118 M34 132 L60 126", 0xF2C14E, 1.6, 0.9);
        s.line(
            &mirror("M30 124 L58 118 M34 132 L60 126", 120.0),
            0xF2C14E,
            1.6,
            0.9,
        );
        let gold = s.vgrad(0.0, 70.0, 0xFFE58A, 0xD4A017);
        let horn = "M114 72 C100 56 86 36 74 12 C92 30 108 46 122 62 Z";
        s.path(horn, &gold, 0x8D6E1A, 2.0);
        s.path(&mirror(horn, 120.0), &gold, 0x8D6E1A, 2.0);
        s.circle(120.0, 74.0, 10.0, &gold, Some((0x8D6E1A, 2.0)));
        s.circle(120.0, 74.0, 4.0, "#c62828", None);
    }
    if look.ex.flower && !look.ex.kabuto && !look.ex.hood {
        s.line("M180 70 L188 100 M172 72 L176 94", 0xD4A017, 1.6, 0.9);
        s.circle(188.0, 102.0, 3.4, "#f06292", None);
        s.circle(176.0, 96.0, 2.8, "#ffffff", Some((0xF48FB1, 1.0)));
        blossom(s, 176.0, 62.0, 15.0, 0xFFE4EE, 0xE91E63, 0.3);
        blossom(s, 158.0, 52.0, 9.0, 0xFFF0F5, 0xF06292, 1.0);
    }
    if look.ex.halo {
        let b = s.blur(3.0);
        s.raw(&format!(r##"<ellipse cx="120" cy="16" rx="46" ry="11" fill="none" stroke="#fff59d" stroke-width="9" opacity="0.7" filter="{b}"/>"##));
        s.raw(r##"<ellipse cx="120" cy="16" rx="46" ry="11" fill="none" stroke="#ffc107" stroke-width="4.5"/>"##);
    }
}

fn eye(s: &mut Svg, cx: f32, cy: f32, iris: u32, side: f32) {
    let white = format!(
        "M{} {} C{} {} {} {} {} {} C{} {} {} {} {} {} Z",
        cx - 15.0,
        cy - 2.0,
        cx - 15.0,
        cy - 21.0,
        cx + 15.0,
        cy - 21.0,
        cx + 15.0,
        cy - 2.0,
        cx + 15.0,
        cy + 15.0,
        cx - 15.0,
        cy + 15.0,
        cx - 15.0,
        cy - 2.0
    );
    s.fill(&white, "#ffffff");
    let clip = s.clip(&white);
    s.open_group(&format!(r#"clip-path="{clip}""#));
    let g = s.lin(
        0.0,
        cy - 15.0,
        0.0,
        cy + 16.0,
        &[
            (0.0, shade(iris, 0.35), 1.0),
            (0.55, iris, 1.0),
            (1.0, tint(iris, 0.45), 1.0),
        ],
    );
    s.ellipse(cx, cy + 1.0, 12.5, 15.5, &g, Some((shade(iris, 0.35), 1.4)));
    s.ellipse(cx, cy + 2.0, 5.5, 8.0, &hex(shade(iris, 0.22)), None);
    s.ellipse(cx, cy + 10.0, 8.0, 3.5, &hex(tint(iris, 0.6)), None);
    s.close_group();
    s.ellipse(cx - 4.5, cy - 5.0, 4.8, 5.6, "#ffffff", None);
    s.circle(cx + 5.0, cy + 7.0, 2.3, "#ffffff", None);
    // Ресницы
    s.line(
        &format!(
            "M{} {} C{} {} {} {} {} {}",
            cx - 17.0,
            cy - 3.0,
            cx - 12.0,
            cy - 21.0,
            cx + 12.0,
            cy - 21.0,
            cx + 17.0,
            cy - 4.0
        ),
        LASH,
        4.4,
        1.0,
    );
    s.line(
        &format!(
            "M{} {} L{} {}",
            cx + side * 16.0,
            cy - 5.0,
            cx + side * 22.0,
            cy - 10.0
        ),
        LASH,
        3.0,
        1.0,
    );
    s.line(
        &format!(
            "M{} {} Q{} {} {} {}",
            cx - 6.0,
            cy + 14.5,
            cx,
            cy + 16.0,
            cx + 6.0,
            cy + 14.5
        ),
        0x8A5A64,
        1.4,
        0.7,
    );
}

// ---------------------------------------------------------------------------
// Оружие
// ---------------------------------------------------------------------------

/// Оружие в руке: (x, y) — кулак, angle — наклон в радианах.
pub fn weapon(s: &mut Svg, look: &Look, x: f32, y: f32, angle: f32) {
    s.open_group(&format!(
        r#"transform="translate({x} {y}) rotate({})""#,
        angle.to_degrees()
    ));
    match look.weapon {
        Weapon::Katana => katana(s, 150.0),
        Weapon::Naginata => {
            let wood = s.lin(
                -3.0,
                0.0,
                3.0,
                0.0,
                &[(0.0, 0x8D6E63, 1.0), (1.0, 0x5D4037, 1.0)],
            );
            s.raw(&format!(r##"<rect x="-3" y="-150" width="6" height="200" rx="3" fill="{wood}" stroke="#3e2723" stroke-width="1.6"/>"##));
            let steel = s.lin(
                -4.0,
                0.0,
                14.0,
                0.0,
                &[(0.0, 0xFFFFFF, 1.0), (1.0, 0x9FB2BF, 1.0)],
            );
            s.path(
                "M-3 -150 Q20 -170 6 -214 Q-4 -182 -3 -150 Z",
                &steel,
                0x607D8B,
                1.6,
            );
            s.raw(r##"<rect x="-6" y="-152" width="12" height="5" rx="2" fill="#f2c14e" stroke="#8d6e1a" stroke-width="1"/>"##);
        }
        Weapon::Bow => {
            s.line("M0 -112 Q-46 -20 0 72", 0x6D4C41, 6.0, 1.0);
            s.line("M0 -112 Q-46 -20 0 72", 0xA1887F, 2.0, 0.8);
            s.line("M0 -112 L0 72", 0xF5F5F5, 1.4, 0.9);
            s.raw(r##"<rect x="-24" y="-30" width="9" height="22" rx="3" fill="#c62828"/>"##);
        }
        Weapon::Ofuda => {
            for (i, a) in [-24.0f32, 0.0, 24.0].iter().enumerate() {
                s.open_group(&format!(r#"transform="rotate({a})""#));
                s.raw(r##"<rect x="-8" y="-62" width="16" height="48" rx="1.5" fill="#fffdf2" stroke="#d7ccc8" stroke-width="1.2"/>"##);
                s.line(
                    "M0 -56 L0 -26 M-4 -48 L4 -48 M-4 -38 L4 -38",
                    0xD32F2F,
                    1.8,
                    0.9,
                );
                if i == 1 {
                    s.circle(0.0, -20.0, 3.0, "#d32f2f", None);
                }
                s.close_group();
            }
        }
        Weapon::Kunai => {
            for a in [-14.0f32, 12.0] {
                s.open_group(&format!(r#"transform="rotate({a})""#));
                let steel = s.lin(
                    -6.0,
                    0.0,
                    6.0,
                    0.0,
                    &[(0.0, 0xECEFF1, 1.0), (1.0, 0x78909C, 1.0)],
                );
                s.path("M0 -64 L7 -26 L0 -18 L-7 -26 Z", &steel, 0x455A64, 1.4);
                s.raw(r##"<rect x="-2.5" y="-18" width="5" height="18" fill="#263238"/>"##);
                s.circle(0.0, 4.0, 4.5, "none", Some((0x263238, 2.2)));
                s.close_group();
            }
        }
        Weapon::Fan => {
            let g = s.rad(
                0.0,
                0.0,
                58.0,
                &[
                    (0.0, 0xFFFFFF, 1.0),
                    (0.5, tint(look.obi, 0.55), 1.0),
                    (1.0, look.obi, 1.0),
                ],
            );
            s.path(
                "M0 0 L-46 -38 A60 60 0 0 1 46 -38 Z",
                &g,
                ink(look.obi),
                2.0,
            );
            let mut ribs = String::new();
            for i in 0..9 {
                let a = (-140.0 + i as f32 * 12.5f32).to_radians();
                let _ = write!(ribs, "M0 0 L{:.1} {:.1} ", a.cos() * 58.0, a.sin() * 58.0);
            }
            s.line(&ribs, shade(look.obi, 0.7), 1.0, 0.5);
            blossom(s, 0.0, -40.0, 9.0, 0xFFFFFF, 0xF06292, 0.0);
            s.circle(0.0, 0.0, 3.0, "#5d4037", None);
        }
        Weapon::Lantern => {
            s.line("M0 8 L0 -46", 0x5D4037, 3.5, 1.0);
            let b = s.blur(8.0);
            s.raw(&format!(
                r##"<circle cx="0" cy="-66" r="30" fill="#ffd54f" opacity="0.5" filter="{b}"/>"##
            ));
            let g = s.rad(
                -3.0,
                -70.0,
                22.0,
                &[
                    (0.0, 0xFFF9C4, 1.0),
                    (0.6, 0xFFB74D, 1.0),
                    (1.0, 0xE65100, 1.0),
                ],
            );
            s.ellipse(0.0, -66.0, 15.0, 19.0, &g, Some((0x8D3A0F, 1.6)));
            s.line(
                "M-14 -72 Q0 -70 14 -72 M-15 -64 Q0 -62 15 -64 M-14 -57 Q0 -55 14 -57",
                0xBF5B1E,
                1.0,
                0.7,
            );
            s.raw(r##"<rect x="-8" y="-88" width="16" height="5" rx="2" fill="#3e2723"/><rect x="-8" y="-49" width="16" height="5" rx="2" fill="#3e2723"/>"##);
        }
        Weapon::Staff => {
            let wood = s.lin(
                -3.0,
                0.0,
                3.0,
                0.0,
                &[(0.0, 0xD7B98E, 1.0), (1.0, 0x8D6E63, 1.0)],
            );
            s.raw(&format!(r##"<rect x="-3" y="-120" width="6" height="168" rx="3" fill="{wood}" stroke="#5d4037" stroke-width="1.4"/>"##));
            let b = s.blur(6.0);
            s.raw(&format!(
                r##"<circle cx="0" cy="-134" r="22" fill="{}" opacity="0.55" filter="{b}"/>"##,
                hex(tint(look.obi, 0.3))
            ));
            let gold = s.vgrad(-150.0, -118.0, 0xFFE58A, 0xD4A017);
            s.circle(0.0, -134.0, 15.0, "none", Some((0xD4A017, 3.5)));
            let orb = s.rad(
                -4.0,
                -138.0,
                12.0,
                &[(0.0, 0xFFFFFF, 1.0), (1.0, look.obi, 1.0)],
            );
            s.circle(0.0, -134.0, 8.0, &orb, None);
            s.circle(-14.0, -120.0, 4.0, &gold, Some((0x8D6E1A, 1.0)));
            s.circle(14.0, -120.0, 4.0, &gold, Some((0x8D6E1A, 1.0)));
        }
    }
    s.close_group();
}

/// Катана рукоятью в (0,0), клинком вверх.
fn katana(s: &mut Svg, len: f32) {
    let steel = s.lin(
        -4.0,
        0.0,
        5.0,
        0.0,
        &[
            (0.0, 0xFFFFFF, 1.0),
            (0.5, 0xE3ECF2, 1.0),
            (1.0, 0x9FB2BF, 1.0),
        ],
    );
    s.path(
        &format!(
            "M-4.5 -14 L4.5 -14 Q9 {} -1 {} Q-5.5 {} -4.5 -14 Z",
            -len * 0.62,
            -len,
            -len * 0.6
        ),
        &steel,
        0x607D8B,
        1.6,
    );
    s.line(
        &format!("M1.2 -20 Q3.5 {} -0.8 {}", -len * 0.6, -len + 8.0),
        0xFFFFFF,
        1.2,
        0.9,
    );
    s.ellipse(0.0, -12.0, 10.0, 4.0, "#3e2723", Some((0xF2C14E, 1.6)));
    s.raw(r##"<rect x="-4" y="-9" width="8" height="34" rx="3" fill="#263147" stroke="#141a26" stroke-width="1.4"/>"##);
    s.line(
        "M-4 -4 L4 2 M-4 4 L4 10 M-4 12 L4 18 M4 -4 L-4 2 M4 4 L-4 10 M4 12 L-4 18",
        0xC9D3E6,
        1.2,
        0.8,
    );
    s.raw(r##"<rect x="-4.5" y="24" width="9" height="4" rx="1.5" fill="#f2c14e"/>"##);
}

/// Отдельный спрайт катаны Сакуры с кулаком. Кулак (точка вращения) — в `KATANA_PIVOT`.
pub const KATANA_W: f32 = 64.0;
pub const KATANA_H: f32 = 240.0;
pub const KATANA_PIVOT: (f32, f32) = (32.0, 204.0);

pub fn katana_svg() -> String {
    let mut s = Svg::new(KATANA_W, KATANA_H);
    s.open_group(&format!(
        r#"transform="translate({} {})""#,
        KATANA_PIVOT.0, KATANA_PIVOT.1
    ));
    katana(&mut s, 190.0);
    s.ellipse(0.0, 2.0, 10.0, 9.0, &hex(SKIN), Some((SKIN_LINE, 1.8)));
    s.line("M-5 -2 L5 -2 M-5 3 L5 3", SKIN_LINE, 1.2, 0.7);
    s.close_group();
    s.finish()
}
