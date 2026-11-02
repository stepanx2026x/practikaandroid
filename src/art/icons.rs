//! Иконки интерфейса и спрайты частиц. Холст иконок 64×64.

use std::fmt::Write;

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Icon {
    Coin,
    Soul,
    Sword,
    Party,
    Lock,
    Storm,
    Blessing,
    Fan,
    Strike,
    TalBlade,
    TalSpirit,
    TalWealth,
    TalLuck,
    TalHourglass,
    TabTeam,
    TabWardrobe,
    TabTemple,
    TabChronicle,
    Crown,
    Gift,
}

pub fn icon_svg(icon: Icon) -> String {
    let mut s = Svg::new(64.0, 64.0);
    match icon {
        Icon::Coin => {
            let g = s.rad(
                26.0,
                22.0,
                34.0,
                &[
                    (0.0, 0xFFF6C2, 1.0),
                    (0.55, 0xFFC93C, 1.0),
                    (1.0, 0xD9930D, 1.0),
                ],
            );
            s.circle(32.0, 32.0, 27.0, &g, Some((0x9A6206, 3.0)));
            s.circle(32.0, 32.0, 20.0, "none", Some((0xE0A520, 2.0)));
            s.raw(r##"<rect x="25" y="25" width="14" height="14" rx="2" fill="#8a5a06" stroke="#6b4504" stroke-width="1.5"/>"##);
            s.line("M14 26 Q20 14 32 11", 0xFFFFFF, 3.0, 0.7);
        }
        Icon::Soul => {
            let b = s.blur(3.0);
            s.raw(&format!(
                r##"<circle cx="32" cy="38" r="20" fill="#d1a6ff" opacity="0.7" filter="{b}"/>"##
            ));
            let g = s.rad(
                32.0,
                42.0,
                26.0,
                &[
                    (0.0, 0xFFFFFF, 1.0),
                    (0.45, 0xF3C6FF, 1.0),
                    (1.0, 0x9B5DE5, 1.0),
                ],
            );
            s.path("M32 6 C38 20 50 26 50 40 C50 52 42 58 32 58 C22 58 14 52 14 40 C14 30 22 26 26 18 C28 26 32 26 32 6 Z", &g, 0x6A3BB5, 2.0);
            s.circle(26.0, 40.0, 2.6, "#4a2a7a", None);
            s.circle(38.0, 40.0, 2.6, "#4a2a7a", None);
        }
        Icon::Sword => {
            s.open_group(r#"transform="rotate(40 32 32)""#);
            let steel = s.lin(
                28.0,
                0.0,
                36.0,
                0.0,
                &[(0.0, 0xFFFFFF, 1.0), (1.0, 0x90A4AE, 1.0)],
            );
            s.path(
                "M29 44 L35 44 Q38 20 32 4 Q28 20 29 44 Z",
                &steel,
                0x546E7A,
                1.8,
            );
            s.ellipse(32.0, 45.0, 9.0, 3.5, "#3e2723", Some((0xF2C14E, 1.4)));
            s.raw(r##"<rect x="28.5" y="47" width="7" height="14" rx="2" fill="#263147" stroke="#141a26" stroke-width="1.2"/>"##);
            s.close_group();
        }
        Icon::Party => {
            for a in [-35.0f32, 35.0] {
                s.open_group(&format!(r#"transform="rotate({a} 32 34)""#));
                let steel = s.lin(
                    28.0,
                    0.0,
                    36.0,
                    0.0,
                    &[(0.0, 0xFFFFFF, 1.0), (1.0, 0x90A4AE, 1.0)],
                );
                s.path(
                    "M29.5 44 L34.5 44 Q37 22 32 6 Q28 22 29.5 44 Z",
                    &steel,
                    0x546E7A,
                    1.6,
                );
                s.ellipse(32.0, 45.0, 8.0, 3.0, "#3e2723", Some((0xF2C14E, 1.2)));
                s.raw(r##"<rect x="29" y="47" width="6" height="12" rx="2" fill="#7b1fa2"/>"##);
                s.close_group();
            }
        }
        Icon::Lock => {
            s.path(
                "M20 30 L20 22 C20 8 44 8 44 22 L44 30",
                "none",
                0x7A6A72,
                5.0,
            );
            let g = s.vgrad(28.0, 56.0, 0xFFD966, 0xD99A10);
            s.raw(&format!(r##"<rect x="14" y="28" width="36" height="28" rx="6" fill="{g}" stroke="#8a5a06" stroke-width="2.4"/>"##));
            s.circle(32.0, 40.0, 4.0, "#6b4504", None);
            s.raw(r##"<rect x="30.5" y="41" width="3" height="8" rx="1.5" fill="#6b4504"/>"##);
        }
        Icon::Storm => {
            let b = s.blur(2.0);
            s.raw(&format!(
                r##"<circle cx="32" cy="32" r="24" fill="#ffd1e3" opacity="0.8" filter="{b}"/>"##
            ));
            s.line(
                "M32 32 m-4 0 a4 4 0 1 1 8 0 a10 10 0 1 1 -18 -2 a16 16 0 1 1 30 6",
                0xF06292,
                3.5,
                1.0,
            );
            for (x, y, r) in [
                (14.0, 18.0, 0.4),
                (50.0, 16.0, 1.2),
                (52.0, 46.0, 2.0),
                (12.0, 48.0, 2.8),
            ] {
                petal(&mut s, x, y, 7.0, r);
            }
        }
        Icon::Blessing => {
            let b = s.blur(4.0);
            s.raw(&format!(r##"<rect x="16" y="6" width="32" height="52" rx="3" fill="#fff59d" opacity="0.8" filter="{b}"/>"##));
            s.raw(r##"<rect x="20" y="8" width="24" height="48" rx="2" fill="#fffdf2" stroke="#c9b8a8" stroke-width="1.6"/>"##);
            s.line(
                "M32 14 L32 50 M25 22 L39 22 M25 30 L39 30 M26 40 L38 46 M38 40 L26 46",
                0xD32F2F,
                2.4,
                0.9,
            );
        }
        Icon::Fan => {
            let g = s.rad(
                32.0,
                52.0,
                44.0,
                &[
                    (0.0, 0xFFF8E1, 1.0),
                    (0.6, 0xFFD54F, 1.0),
                    (1.0, 0xE0A520, 1.0),
                ],
            );
            s.path("M32 54 L6 26 A36 36 0 0 1 58 26 Z", &g, 0x9A6206, 2.2);
            let mut ribs = String::new();
            for i in 0..8 {
                let a = (-138.0 + i as f32 * 13.5f32).to_radians();
                let _ = write!(
                    ribs,
                    "M32 54 L{:.1} {:.1} ",
                    32.0 + a.cos() * 38.0,
                    54.0 + a.sin() * 38.0
                );
            }
            s.line(&ribs, 0xB07A10, 1.2, 0.7);
            s.circle(32.0, 54.0, 3.5, "#6b4504", None);
            s.line("M14 22 Q32 8 50 22", 0xFFFFFF, 2.0, 0.6);
        }
        Icon::Strike => {
            let b = s.blur(3.0);
            s.raw(&format!(r##"<path d="M38 4 L16 36 L30 36 L24 60 L50 24 L35 24 Z" fill="#80d8ff" opacity="0.8" filter="{b}"/>"##));
            let g = s.vgrad(4.0, 60.0, 0xFFFFFF, 0xFFE066);
            s.path(
                "M38 4 L16 36 L30 36 L24 60 L50 24 L35 24 Z",
                &g,
                0xC79100,
                2.0,
            );
        }
        Icon::TalBlade => {
            let b = s.blur(3.0);
            s.raw(&format!(
                r##"<circle cx="32" cy="32" r="22" fill="#ffccbc" opacity="0.7" filter="{b}"/>"##
            ));
            s.open_group(r#"transform="rotate(-40 32 32)""#);
            let steel = s.lin(
                28.0,
                0.0,
                36.0,
                0.0,
                &[(0.0, 0xFFFFFF, 1.0), (1.0, 0x90A4AE, 1.0)],
            );
            s.path(
                "M29 44 L35 44 Q38 20 32 4 Q28 20 29 44 Z",
                &steel,
                0x546E7A,
                1.8,
            );
            s.ellipse(32.0, 45.0, 9.0, 3.5, "#3e2723", Some((0xF2C14E, 1.4)));
            s.raw(r##"<rect x="28.5" y="47" width="7" height="14" rx="2" fill="#c62828"/>"##);
            s.close_group();
        }
        Icon::TalSpirit => {
            let b = s.blur(3.0);
            s.raw(&format!(
                r##"<circle cx="32" cy="36" r="20" fill="#80d8ff" opacity="0.7" filter="{b}"/>"##
            ));
            let g = s.rad(
                32.0,
                42.0,
                26.0,
                &[
                    (0.0, 0xFFFFFF, 1.0),
                    (0.45, 0xB3E5FC, 1.0),
                    (1.0, 0x29B6F6, 1.0),
                ],
            );
            s.path("M32 6 C38 20 50 26 50 40 C50 52 42 58 32 58 C22 58 14 52 14 40 C14 30 22 26 26 18 C28 26 32 26 32 6 Z", &g, 0x0277BD, 2.0);
        }
        Icon::TalWealth => {
            // манэки-нэко
            s.path(
                "M14 26 L16 8 L28 18 Z M50 26 L48 8 L36 18 Z",
                "#ffffff",
                0x9E9E9E,
                2.0,
            );
            s.circle(32.0, 34.0, 22.0, "#ffffff", Some((0x9E9E9E, 2.2)));
            s.path("M18 16 L20 10 L26 16 Z", "#f48fb1", 0xF48FB1, 0.0);
            s.ellipse(25.0, 32.0, 3.0, 3.5, "#3e2723", None);
            s.ellipse(39.0, 32.0, 3.0, 3.5, "#3e2723", None);
            s.line("M28 40 Q32 44 36 40", 0x3E2723, 2.0, 1.0);
            s.circle(32.0, 52.0, 5.0, "#ffc107", Some((0x9A6206, 1.4)));
            s.line("M14 50 Q32 58 50 50", 0xD32F2F, 4.0, 1.0);
        }
        Icon::TalLuck => {
            let g = s.vgrad(12.0, 58.0, 0xF05454, 0xB71C1C);
            s.path(
                "M18 22 Q18 12 32 12 Q46 12 46 22 L48 52 Q32 60 16 52 Z",
                &g,
                0x7F0000,
                2.2,
            );
            s.raw(r##"<rect x="24" y="24" width="16" height="22" rx="2" fill="#fff3e0" stroke="#d4a017" stroke-width="1.4"/>"##);
            s.line("M32 28 L32 42 M27 34 L37 34", 0xD4A017, 2.0, 1.0);
            s.line("M26 12 Q32 2 38 12", 0xFFD54F, 2.4, 1.0);
        }
        Icon::TalHourglass => {
            s.raw(r##"<rect x="14" y="6" width="36" height="6" rx="3" fill="#8d5a3b"/><rect x="14" y="52" width="36" height="6" rx="3" fill="#8d5a3b"/>"##);
            s.path(
                "M18 12 L46 12 Q46 28 34 32 Q46 36 46 52 L18 52 Q18 36 30 32 Q18 28 18 12 Z",
                "#e3f2fd",
                0x90A4AE,
                2.0,
            );
            s.path(
                "M22 46 Q32 38 42 46 L42 50 L22 50 Z M26 16 L38 16 Q36 24 32 28 Q28 24 26 16 Z",
                "#ffc107",
                0xD4A017,
                1.0,
            );
        }
        Icon::TabTeam => {
            blossom(&mut s, 32.0, 32.0, 26.0, 0xFFFFFF, 0xF06292, 0.0);
        }
        Icon::TabWardrobe => {
            let g = s.vgrad(10.0, 56.0, 0xFFFFFF, 0xF8D7E5);
            s.path(
                "M24 10 L32 22 L40 10 L54 16 L58 34 L46 36 L46 56 L18 56 L18 36 L6 34 L10 16 Z",
                &g,
                0xC2185B,
                2.4,
            );
            s.line("M24 10 L34 30 M40 10 L30 26", 0xC2185B, 2.0, 1.0);
            s.raw(r##"<rect x="18" y="34" width="28" height="7" fill="#e91e63"/>"##);
        }
        Icon::TabTemple => {
            torii_icon(&mut s);
        }
        Icon::TabChronicle => {
            let paper = s.vgrad(10.0, 54.0, 0xFFFFFF, 0xF3E6C8);
            s.raw(&format!(r##"<rect x="12" y="10" width="40" height="44" rx="4" fill="{paper}" stroke="#b9a27a" stroke-width="2"/>"##));
            s.line(
                "M20 22 L44 22 M20 30 L44 30 M20 38 L36 38",
                0xC2185B,
                2.6,
                0.9,
            );
            s.raw(r##"<rect x="10" y="8" width="8" height="48" rx="2" fill="#c2185b"/>"##);
        }
        Icon::Crown => {
            let g = s.vgrad(14.0, 50.0, 0xFFF0A0, 0xE0A520);
            s.path(
                "M8 48 L10 18 L22 32 L32 12 L42 32 L54 18 L56 48 Z",
                &g,
                0x9A6206,
                2.4,
            );
            s.circle(32.0, 38.0, 5.0, "#e91e63", Some((0x880E4F, 1.2)));
        }
        Icon::Gift => {
            blossom(&mut s, 32.0, 32.0, 26.0, 0xFFE4EE, 0xF06292, 0.4);
        }
    }
    s.finish()
}

fn torii_icon(s: &mut Svg) {
    let g = s.vgrad(8.0, 58.0, 0xF05454, 0xB71C1C);
    s.raw(&format!(
        r##"<rect x="16" y="18" width="6" height="40" fill="{g}"/><rect x="42" y="18" width="6" height="40" fill="{g}"/><rect x="12" y="24" width="40" height="5" fill="{g}"/><path d="M4 12 Q32 18 60 12 L58 18 Q32 22 6 18 Z" fill="{g}"/><rect x="4" y="8" width="56" height="5" rx="2" fill="#2b1d1d"/>"##
    ));
}

fn petal(s: &mut Svg, x: f32, y: f32, r: f32, rot: f32) {
    let (sn, cs) = rot.sin_cos();
    let p = |dx: f32, dy: f32| (x + dx * cs - dy * sn, y + dx * sn + dy * cs);
    let a = p(0.0, r);
    let b = p(r * 0.9, -r * 0.2);
    let c = p(r * 0.35, -r);
    let d = p(0.0, -r * 0.7);
    let e = p(-r * 0.35, -r);
    let f = p(-r * 0.9, -r * 0.2);
    let g = s.rad(x, y, r * 1.2, &[(0.0, 0xFFFFFF, 1.0), (1.0, 0xF48FB1, 1.0)]);
    s.path(
        &format!("M{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1} Z", a.0, a.1, b.0, b.1, c.0, c.1, d.0, d.1, e.0, e.1, f.0, f.1, a.0, a.1),
        &g, 0xE57399, 0.8,
    );
}

/// Спрайты частиц: (svg, ширина, высота).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Particle {
    Petal,
    Leaf,
    Sparkle,
    Slash,
}

pub fn particle_svg(p: Particle) -> String {
    match p {
        Particle::Petal => {
            let mut s = Svg::new(32.0, 32.0);
            petal(&mut s, 16.0, 16.0, 13.0, 0.0);
            s.finish()
        }
        Particle::Leaf => {
            let mut s = Svg::new(32.0, 32.0);
            let g = s.lin(
                4.0,
                4.0,
                28.0,
                28.0,
                &[(0.0, 0xC5E1A5, 1.0), (1.0, 0x558B2F, 1.0)],
            );
            s.path("M4 28 Q8 8 28 4 Q24 24 4 28 Z", &g, 0x33691E, 1.0);
            s.line("M4 28 L24 8", 0x33691E, 1.0, 0.7);
            s.finish()
        }
        Particle::Sparkle => {
            let mut s = Svg::new(48.0, 48.0);
            let b = s.blur(3.0);
            s.raw(&format!(
                r##"<circle cx="24" cy="24" r="12" fill="#fff59d" opacity="0.8" filter="{b}"/>"##
            ));
            s.path(
                "M24 2 Q27 21 46 24 Q27 27 24 46 Q21 27 2 24 Q21 21 24 2 Z",
                "#ffffff",
                0xFFE082,
                1.0,
            );
            s.finish()
        }
        Particle::Slash => {
            let mut s = Svg::new(256.0, 128.0);
            let b = s.blur(6.0);
            let g = s.lin(
                0.0,
                0.0,
                256.0,
                0.0,
                &[
                    (0.0, 0xF48FB1, 0.0),
                    (0.5, 0xFFFFFF, 1.0),
                    (1.0, 0xF48FB1, 0.0),
                ],
            );
            let d = "M8 100 Q128 -10 248 100 Q128 30 8 100 Z";
            s.raw(&format!(
                r##"<path d="{d}" fill="#ff80ab" opacity="0.6" filter="{b}"/>"##
            ));
            s.fill(d, &g);
            s.finish()
        }
    }
}
