//! Пейзажи локаций. Каждая локация — два слоя: задний (небо, горы, земля)
//! и передний (ветки сверху и трава снизу), между ними стоят герои.

use std::fmt::Write;

use super::*;

/// Детерминированный генератор случайных чисел для расстановки деталей.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            | 1)
    }
    pub fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

/// Светящаяся точка, которую сцена анимирует поверх фона (фонари, лава, окна).
#[derive(Clone, Copy)]
pub struct Light {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub color: u32,
}

/// Гладкий хребет гор: замкнутый контур до низа холста.
fn ridge(w: f32, h: f32, base: f32, amp: f32, n: usize, rng: &mut Rng) -> String {
    let mut pts = Vec::with_capacity(n + 3);
    for i in 0..=n + 2 {
        let x = -w * 0.1 + (w * 1.2) * (i as f32 / (n + 2) as f32);
        let y = base - amp * (0.35 + 0.65 * rng.f());
        pts.push((x, y));
    }
    let mut d = format!(
        "M{:.1} {:.1} L{:.1} {:.1} ",
        pts[0].0, h, pts[0].0, pts[0].1
    );
    for i in 0..pts.len() - 1 {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < pts.len() { pts[i + 2] } else { p2 };
        let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        let _ = write!(
            d,
            "C{:.1} {:.1} {:.1} {:.1} {:.1} {:.1} ",
            c1.0, c1.1, c2.0, c2.1, p2.0, p2.1
        );
    }
    let _ = write!(d, "L{:.1} {:.1} Z", pts[pts.len() - 1].0, h);
    d
}

/// Острый горный хребет (ломаная).
fn peaks(w: f32, h: f32, base: f32, amp: f32, n: usize, rng: &mut Rng) -> String {
    let mut d = format!("M{:.1} {:.1} ", -10.0, h);
    for i in 0..=n {
        let x = -10.0 + (w + 20.0) * (i as f32 / n as f32);
        let y = if i % 2 == 0 {
            base - amp * rng.range(0.1, 0.35)
        } else {
            base - amp * rng.range(0.6, 1.0)
        };
        let _ = write!(d, "L{:.1} {:.1} ", x, y);
    }
    let _ = write!(d, "L{:.1} {:.1} Z", w + 10.0, h);
    d
}

fn sky(s: &mut Svg, w: f32, h: f32, stops: &[(f32, u32, f32)]) {
    let g = s.lin(0.0, 0.0, 0.0, h * 0.7, stops);
    s.raw(&format!(
        r#"<rect x="0" y="0" width="{w}" height="{h}" fill="{g}"/>"#
    ));
}

fn glow(s: &mut Svg, cx: f32, cy: f32, r: f32, c: u32, a: f32) {
    let g = s.rad(cx, cy, r, &[(0.0, c, a), (0.4, c, a * 0.45), (1.0, c, 0.0)]);
    s.circle(cx, cy, r, &g, None);
}

fn haze(s: &mut Svg, w: f32, y: f32, hh: f32, c: u32, a: f32) {
    let g = s.lin(
        0.0,
        y - hh,
        0.0,
        y + hh,
        &[(0.0, c, 0.0), (0.5, c, a), (1.0, c, 0.0)],
    );
    s.raw(&format!(
        r#"<rect x="0" y="{}" width="{w}" height="{}" fill="{g}"/>"#,
        y - hh,
        hh * 2.0
    ));
}

fn ground(s: &mut Svg, w: f32, h: f32, y: f32, top: u32, bottom: u32) {
    let g = s.vgrad(y, h, top, bottom);
    s.raw(&format!(
        r#"<path d="M0 {y} Q{} {} {w} {y} L{w} {h} L0 {h} Z" fill="{g}"/>"#,
        w * 0.5,
        y - 10.0
    ));
}

/// Облако из кругов с мягкими краями.
pub fn cloud_svg(c: u32) -> String {
    let mut s = Svg::new(200.0, 100.0);
    let b = s.blur(3.0);
    let g = s.vgrad(20.0, 90.0, 0xFFFFFF, c);
    s.raw(&format!(
        r#"<g filter="{b}" fill="{g}"><circle cx="60" cy="60" r="28"/><circle cx="96" cy="46" r="34"/><circle cx="136" cy="58" r="28"/><circle cx="160" cy="70" r="18"/><rect x="40" y="58" width="130" height="28" rx="14"/></g>"#
    ));
    s.finish()
}

fn sakura_tree(s: &mut Svg, x: f32, base: f32, size: f32, petal: u32, rng: &mut Rng) {
    let trunk = s.lin(
        x - size * 0.1,
        0.0,
        x + size * 0.1,
        0.0,
        &[(0.0, 0x6D4C41, 1.0), (1.0, 0x4E342E, 1.0)],
    );
    let d = format!(
        "M{} {} C{} {} {} {} {} {} L{} {} C{} {} {} {} {} {} Z",
        x - size * 0.08,
        base,
        x - size * 0.02,
        base - size * 0.4,
        x - size * 0.16,
        base - size * 0.6,
        x - size * 0.02,
        base - size * 0.82,
        x + size * 0.06,
        base - size * 0.8,
        x + size * 0.0,
        base - size * 0.6,
        x + size * 0.1,
        base - size * 0.4,
        x + size * 0.08,
        base
    );
    s.path(&d, &trunk, 0x3E2723, 1.5);
    s.line(
        &format!(
            "M{} {} Q{} {} {} {} M{} {} Q{} {} {} {}",
            x,
            base - size * 0.6,
            x - size * 0.2,
            base - size * 0.7,
            x - size * 0.34,
            base - size * 0.78,
            x,
            base - size * 0.66,
            x + size * 0.2,
            base - size * 0.74,
            x + size * 0.32,
            base - size * 0.86
        ),
        0x4E342E,
        size * 0.035,
        1.0,
    );
    let cols = [
        petal,
        tint(petal, 0.35),
        tint(petal, 0.6),
        shade(petal, 0.9),
    ];
    let mut blobs = String::new();
    for _ in 0..34 {
        let a = rng.f() * std::f32::consts::TAU;
        let dist = rng.f().sqrt();
        let cx = x + a.cos() * dist * size * 0.5;
        let cy = base - size * 0.86 + a.sin() * dist * size * 0.28;
        let r = size * rng.range(0.07, 0.14);
        let _ = write!(
            blobs,
            r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{}" opacity="0.92"/>"#,
            hex(cols[(rng.f() * 4.0) as usize % 4])
        );
    }
    s.raw(&blobs);
}

fn pagoda(s: &mut Svg, x: f32, base: f32, size: f32, c: u32, a: f32) {
    let mut d = String::new();
    let tiers = 5;
    for i in 0..tiers {
        let y = base - size * (i as f32 * 0.19);
        let half = size * (0.34 - i as f32 * 0.04);
        let _ = write!(
            d,
            "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1} L{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1} Z ",
            x - half * 0.55, y, x - half * 0.55, y - size * 0.1, x + half * 0.55, y - size * 0.1, x + half * 0.55, y,
            x + half * 1.1, y - size * 0.1, x + half * 1.25, y - size * 0.14, x - half * 1.25, y - size * 0.14,
            x - half * 1.1, y - size * 0.1, x - half * 0.55, y
        );
    }
    let top = base - size * 0.95;
    let _ = write!(
        d,
        "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Z",
        x - size * 0.015,
        top + size * 0.12,
        x,
        top - size * 0.08,
        x + size * 0.015,
        top + size * 0.12
    );
    s.fill_a(&d, c, a);
}

pub fn torii(s: &mut Svg, cx: f32, base: f32, w: f32, c: u32) {
    let h = w * 0.82;
    let g = s.vgrad(base - h, base, tint(c, 0.15), shade(c, 0.8));
    s.raw(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{g}"/><rect x="{}" y="{}" width="{}" height="{}" fill="{g}"/><rect x="{}" y="{}" width="{}" height="{}" fill="{g}"/>"#,
        cx - w * 0.36, base - h, w * 0.08, h, cx + w * 0.28, base - h, w * 0.08, h, cx - w * 0.44, base - h * 0.76, w * 0.88, w * 0.06
    ));
    s.fill(
        &format!(
            "M{} {} Q{} {} {} {} L{} {} Q{} {} {} {} Z",
            cx - w * 0.58,
            base - h * 1.02,
            cx,
            base - h * 0.9,
            cx + w * 0.58,
            base - h * 1.02,
            cx + w * 0.52,
            base - h * 0.9,
            cx,
            base - h * 0.82,
            cx - w * 0.52,
            base - h * 0.9
        ),
        &g,
    );
    s.raw(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" rx="2" fill="#2b1d1d"/>"##,
        cx - w * 0.6,
        base - h * 1.08,
        w * 1.2,
        w * 0.05
    ));
}

fn lantern_string(
    s: &mut Svg,
    x1: f32,
    x2: f32,
    y: f32,
    sag: f32,
    n: usize,
    lights: &mut Vec<Light>,
) {
    s.line(
        &format!("M{x1} {y} Q{} {} {x2} {y}", (x1 + x2) / 2.0, y + sag * 2.0),
        0x1B1030,
        2.0,
        0.9,
    );
    for i in 1..n {
        let t = i as f32 / n as f32;
        let x = x1 + (x2 - x1) * t;
        let yy = y + 4.0 * sag * t * (1.0 - t) + 14.0;
        let g = s.rad(
            x - 2.0,
            yy - 3.0,
            14.0,
            &[
                (0.0, 0xFFF3C4, 1.0),
                (0.6, 0xFF7043, 1.0),
                (1.0, 0xC62828, 1.0),
            ],
        );
        s.ellipse(x, yy, 9.0, 11.5, &g, Some((0x5D1A1A, 1.2)));
        s.raw(&format!(
            r##"<rect x="{}" y="{}" width="8" height="3" fill="#1b1030"/>"##,
            x - 4.0,
            yy - 14.0
        ));
        lights.push(Light {
            x,
            y: yy,
            r: 26.0,
            color: 0xFFB74D,
        });
    }
}

/// Задний слой локации и список светящихся точек.
pub fn backdrop(zone: usize, w: f32, h: f32) -> (String, Vec<Light>) {
    let mut s = Svg::new(w, h);
    let mut rng = Rng::new(zone as u64 * 977 + 13);
    let mut lights = Vec::new();
    let hz = h * 0.62;
    match zone {
        // ---------------- Сакуровая роща
        0 => {
            sky(
                &mut s,
                w,
                h,
                &[
                    (0.0, 0xB9D3F5, 1.0),
                    (0.55, 0xF9D3E3, 1.0),
                    (1.0, 0xFFF1E4, 1.0),
                ],
            );
            glow(&mut s, w * 0.8, h * 0.2, h * 0.3, 0xFFFFFF, 0.9);
            s.circle(w * 0.8, h * 0.2, h * 0.05, "#fffaf0", None);
            // гора с белой шапкой
            let fuji = format!(
                "M{} {hz} L{} {} Q{} {} {} {} L{} {hz} Z",
                w * 0.3,
                w * 0.56,
                h * 0.26,
                w * 0.6,
                h * 0.24,
                w * 0.64,
                h * 0.26,
                w * 0.92
            );
            let g = s.vgrad(h * 0.24, hz, 0xBBA7DE, 0xF2DDF0);
            s.fill(&fuji, &g);
            let cap = format!(
                "M{} {} Q{} {} {} {} L{} {} L{} {} L{} {} L{} {} L{} {} L{} {} Z",
                w * 0.56,
                h * 0.26,
                w * 0.6,
                h * 0.235,
                w * 0.64,
                h * 0.26,
                w * 0.69,
                h * 0.33,
                w * 0.66,
                h * 0.32,
                w * 0.635,
                h * 0.35,
                w * 0.6,
                h * 0.32,
                w * 0.565,
                h * 0.345,
                w * 0.53,
                h * 0.315
            );
            s.fill(&cap, "#ffffff");
            let far = ridge(w, h, h * 0.58, h * 0.08, 7, &mut rng);
            let g = s.vgrad(h * 0.48, hz, 0xD8C3EA, 0xF6E6F2);
            s.fill(&far, &g);
            haze(&mut s, w, h * 0.58, h * 0.05, 0xFFFFFF, 0.6);
            pagoda(&mut s, w * 0.17, h * 0.6, h * 0.24, 0x9C7FB8, 0.75);
            let hills = ridge(w, h, hz + 6.0, h * 0.05, 5, &mut rng);
            let g = s.vgrad(hz - h * 0.05, hz + 20.0, 0xF6B8CF, 0xE9A5C0);
            s.fill(&hills, &g);
            for i in 0..7 {
                let x = w * (0.05 + i as f32 * 0.15) + rng.range(-20.0, 20.0);
                sakura_tree(
                    &mut s,
                    x,
                    hz + 4.0,
                    h * rng.range(0.12, 0.18),
                    0xF7A8C4,
                    &mut rng,
                );
            }
            torii(&mut s, w * 0.86, hz + h * 0.05, h * 0.2, 0xE0453C);
            ground(&mut s, w, h, hz + h * 0.02, 0xD5ECB4, 0x9CCB78);
            let path = format!(
                "M{} {h} C{} {} {} {} {} {} L{} {} C{} {} {} {} {} {h} Z",
                w * 0.36,
                w * 0.42,
                h * 0.8,
                w * 0.5,
                h * 0.7,
                w * 0.52,
                hz + h * 0.03,
                w * 0.56,
                hz + h * 0.03,
                w * 0.58,
                h * 0.7,
                w * 0.7,
                h * 0.8,
                w * 0.78
            );
            let g = s.vgrad(hz, h, 0xF8EBD4, 0xE9D3B0);
            s.fill(&path, &g);
            grass_and_petals(&mut s, w, h, hz, 0x6FAF4F, 0xF48FB1, &mut rng);
        }
        // ---------------- Бамбуковый лес
        1 => {
            sky(&mut s, w, h, &[(0.0, 0xCFEBC6, 1.0), (1.0, 0xF4FBEE, 1.0)]);
            glow(&mut s, w * 0.5, h * 0.05, h * 0.5, 0xFFFFF0, 0.8);
            for (layer, (c, wmin, wmax, a, n)) in [
                (0xC6E3B5, 8.0, 14.0, 0.8, 22),
                (0x8CC57A, 14.0, 22.0, 0.95, 14),
                (0x5E9E4C, 20.0, 30.0, 1.0, 8),
            ]
            .iter()
            .enumerate()
            {
                for _ in 0..*n {
                    let x = rng.range(-20.0, w + 20.0);
                    let bw = rng.range(*wmin, *wmax);
                    let bottom = hz + 20.0 + layer as f32 * 30.0;
                    let g = s.lin(
                        x - bw / 2.0,
                        0.0,
                        x + bw / 2.0,
                        0.0,
                        &[
                            (0.0, tint(*c, 0.25), *a),
                            (0.6, *c, *a),
                            (1.0, shade(*c, 0.8), *a),
                        ],
                    );
                    s.raw(&format!(r#"<rect x="{:.1}" y="-10" width="{bw:.1}" height="{:.1}" rx="{:.1}" fill="{g}"/>"#, x - bw / 2.0, bottom + 10.0, bw / 2.0));
                    let mut joints = String::new();
                    let mut y = rng.range(20.0, 80.0);
                    while y < bottom {
                        let _ = write!(
                            joints,
                            "M{:.1} {:.1} L{:.1} {:.1} ",
                            x - bw / 2.0,
                            y,
                            x + bw / 2.0,
                            y
                        );
                        y += rng.range(70.0, 110.0);
                    }
                    s.line(&joints, shade(*c, 0.7), 2.5, *a);
                    // листья
                    let ly = rng.range(30.0, hz * 0.6);
                    let dir = if rng.f() > 0.5 { 1.0 } else { -1.0 };
                    s.fill_a(
                        &format!(
                            "M{x} {ly} Q{} {} {} {} Q{} {} {x} {ly} Z",
                            x + dir * 30.0,
                            ly - 14.0,
                            x + dir * 60.0,
                            ly + 6.0,
                            x + dir * 30.0,
                            ly + 4.0
                        ),
                        shade(*c, 0.85),
                        *a,
                    );
                }
                if layer == 0 {
                    haze(&mut s, w, hz - 20.0, h * 0.12, 0xF4FBEE, 0.8);
                }
            }
            ground(&mut s, w, h, hz + h * 0.04, 0xB8DB98, 0x7FB466);
            for i in 0..6 {
                let x = w * (0.3 + i as f32 * 0.09);
                let y = h * (0.74 + (i % 2) as f32 * 0.06 + i as f32 * 0.02);
                s.ellipse(
                    x,
                    y,
                    26.0 + i as f32 * 3.0,
                    9.0 + i as f32,
                    "#c9c2b6",
                    Some((0x9E968A, 1.5)),
                );
            }
            // каменный фонарь торо
            let x = w * 0.86;
            let b = hz + h * 0.06;
            s.raw(&format!(
                r##"<g fill="#b8b2a7" stroke="#7d776d" stroke-width="2"><rect x="{}" y="{}" width="40" height="10" rx="3"/><rect x="{}" y="{}" width="14" height="40"/><rect x="{}" y="{}" width="36" height="8" rx="3"/><rect x="{}" y="{}" width="28" height="24"/><path d="M{} {} L{} {} L{} {} Z"/></g><rect x="{}" y="{}" width="12" height="12" fill="#ffe082"/>"##,
                x - 20.0, b - 10.0, x - 7.0, b - 50.0, x - 18.0, b - 58.0, x - 14.0, b - 82.0, x - 26.0, b - 82.0, x, b - 106.0, x + 26.0, b - 82.0, x - 6.0, b - 76.0
            ));
            lights.push(Light {
                x,
                y: b - 70.0,
                r: 30.0,
                color: 0xFFE082,
            });
            grass_and_petals(&mut s, w, h, hz + h * 0.04, 0x4E8F3F, 0xA5D6A7, &mut rng);
        }
        // ---------------- Горный храм
        2 => {
            sky(
                &mut s,
                w,
                h,
                &[
                    (0.0, 0x8A7BD0, 1.0),
                    (0.5, 0xE7A7C9, 1.0),
                    (1.0, 0xFFD3B0, 1.0),
                ],
            );
            glow(&mut s, w * 0.3, h * 0.5, h * 0.35, 0xFFE0B2, 0.9);
            s.circle(w * 0.3, h * 0.5, h * 0.08, "#fff3e0", None);
            for (i, (c, base, amp)) in [
                (0xC3A7DF, 0.52, 0.22),
                (0x9B80C6, 0.58, 0.16),
                (0x725C9E, 0.64, 0.1),
            ]
            .iter()
            .enumerate()
            {
                let d = peaks(w, h, h * base, h * amp, 9 + i * 2, &mut rng);
                let g = s.vgrad(h * (base - amp), h * base, tint(*c, 0.1), *c);
                s.fill(&d, &g);
                haze(&mut s, w, h * (base - 0.01), h * 0.04, 0xFFE4F0, 0.55);
            }
            // храм на скале
            let x = w * 0.78;
            let b = h * 0.5;
            s.fill(
                &format!(
                    "M{} {} L{} {} L{} {} L{} {} Z",
                    x - 120.0,
                    h,
                    x - 70.0,
                    b + 20.0,
                    x + 90.0,
                    b + 10.0,
                    x + 160.0,
                    h
                ),
                "#4a3d63",
            );
            s.raw(&format!(
                r##"<rect x="{}" y="{}" width="120" height="44" fill="#c0392b"/><rect x="{}" y="{}" width="104" height="30" fill="#f6e6c8"/><path d="M{} {} L{} {} L{} {} L{} {} Q{} {} {} {} Z" fill="#2c2340" stroke="#d4a017" stroke-width="2"/>"##,
                x - 60.0, b - 30.0, x - 52.0, b - 24.0, x - 84.0, b - 30.0, x - 50.0, b - 62.0, x + 50.0, b - 62.0, x + 84.0, b - 30.0, x, b - 40.0, x - 84.0, b - 30.0
            ));
            lights.push(Light {
                x: x - 20.0,
                y: b - 10.0,
                r: 24.0,
                color: 0xFFCC80,
            });
            lights.push(Light {
                x: x + 20.0,
                y: b - 10.0,
                r: 24.0,
                color: 0xFFCC80,
            });
            torii(&mut s, w * 0.2, hz + h * 0.06, h * 0.22, 0xE0453C);
            ground(&mut s, w, h, hz + h * 0.03, 0xD7C8E6, 0xA898BE);
            let mut tiles = String::new();
            for i in 0..12 {
                let t = i as f32 / 11.0;
                let _ = write!(
                    tiles,
                    "M{:.1} {:.1} L{:.1} {h} ",
                    w * 0.5 + (t - 0.5) * w * 0.4,
                    hz + h * 0.04,
                    w * 0.5 + (t - 0.5) * w * 2.2
                );
            }
            for i in 0..6 {
                let y = hz + h * 0.04 + (h - hz) * (i as f32 / 6.0).powf(1.6);
                let _ = write!(tiles, "M0 {y:.1} L{w} {y:.1} ");
            }
            s.line(&tiles, 0x9A88B2, 1.5, 0.5);
            grass_and_petals(&mut s, w, h, hz + h * 0.03, 0x7E6AA6, 0xF8BBD0, &mut rng);
        }
        // ---------------- Город фонарей (ночь)
        3 => {
            sky(
                &mut s,
                w,
                h,
                &[
                    (0.0, 0x0D1136, 1.0),
                    (0.6, 0x2B2F73, 1.0),
                    (1.0, 0x5B4F9C, 1.0),
                ],
            );
            let mut stars = String::new();
            for _ in 0..90 {
                let _ = write!(
                    stars,
                    r##"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="#ffffff" opacity="{:.2}"/>"##,
                    rng.range(0.0, w),
                    rng.range(0.0, hz * 0.8),
                    rng.range(0.6, 1.8),
                    rng.range(0.3, 0.9)
                );
            }
            s.raw(&stars);
            glow(&mut s, w * 0.8, h * 0.17, h * 0.22, 0xFFF9C4, 0.5);
            let mg = s.rad(
                w * 0.79,
                h * 0.16,
                h * 0.06,
                &[(0.0, 0xFFFDE7, 1.0), (1.0, 0xFFF3B0, 1.0)],
            );
            s.circle(w * 0.8, h * 0.17, h * 0.06, &mg, None);
            s.circle(w * 0.81, h * 0.16, h * 0.012, "#f0e3a0", None);
            s.circle(w * 0.785, h * 0.185, h * 0.008, "#f0e3a0", None);
            // силуэты крыш
            for (layer, (c, base, scale)) in [(0x24275E, 0.5, 0.7), (0x1A1C47, 0.6, 1.0)]
                .iter()
                .enumerate()
            {
                let mut x = -20.0;
                while x < w + 40.0 {
                    let bw = rng.range(70.0, 130.0) * scale;
                    let bh = rng.range(50.0, 110.0) * scale;
                    let top = h * base - bh;
                    s.raw(&format!(r#"<rect x="{x:.1}" y="{top:.1}" width="{bw:.1}" height="{:.1}" fill="{}"/>"#, h - top, hex(*c)));
                    s.fill(&format!("M{:.1} {top:.1} L{:.1} {:.1} L{:.1} {:.1} L{:.1} {top:.1} Q{:.1} {:.1} {:.1} {top:.1} Z", x - 12.0, x + bw * 0.2, top - 22.0 * scale, x + bw * 0.8, top - 22.0 * scale, x + bw + 12.0, x + bw / 2.0, top - 6.0, x - 12.0), &hex(shade(*c, 0.8)));
                    for k in 0..(bw / 22.0) as i32 {
                        if rng.f() > 0.45 {
                            let wx = x + 8.0 + k as f32 * 22.0;
                            let wy = top + 16.0 + rng.range(0.0, bh * 0.4);
                            s.raw(&format!(r##"<rect x="{wx:.1}" y="{wy:.1}" width="12" height="14" fill="#ffca6b" opacity="{}"/>"##, 0.5 + layer as f32 * 0.4));
                            if layer == 1 {
                                lights.push(Light {
                                    x: wx + 6.0,
                                    y: wy + 7.0,
                                    r: 16.0,
                                    color: 0xFFB74D,
                                });
                            }
                        }
                    }
                    x += bw + rng.range(4.0, 20.0);
                }
            }
            lantern_string(&mut s, -10.0, w * 0.55, h * 0.3, 36.0, 6, &mut lights);
            lantern_string(&mut s, w * 0.45, w + 10.0, h * 0.24, 40.0, 6, &mut lights);
            ground(&mut s, w, h, hz + h * 0.02, 0x3D3557, 0x221D35);
            for i in 0..5 {
                let x = w * (0.1 + i as f32 * 0.2);
                glow(&mut s, x, h * 0.82, 70.0, 0xFF9E40, 0.18);
            }
            let mut cobbles = String::new();
            for _ in 0..60 {
                let _ = write!(
                    cobbles,
                    r##"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="#4a4266" opacity="0.6"/>"##,
                    rng.range(0.0, w),
                    rng.range(hz + 30.0, h),
                    rng.range(10.0, 22.0),
                    rng.range(4.0, 8.0)
                );
            }
            s.raw(&cobbles);
        }
        // ---------------- Снежный перевал
        4 => {
            sky(&mut s, w, h, &[(0.0, 0x9CC6EE, 1.0), (1.0, 0xEAF5FF, 1.0)]);
            glow(&mut s, w * 0.25, h * 0.15, h * 0.25, 0xFFFFFF, 0.8);
            for (i, (base, amp)) in [(0.5, 0.3), (0.58, 0.2)].iter().enumerate() {
                let d = peaks(w, h, h * base, h * amp, 7 + i * 3, &mut rng);
                let g = s.lin(
                    0.0,
                    h * (base - amp),
                    w,
                    h * base,
                    &[
                        (0.0, 0xFFFFFF, 1.0),
                        (0.5, 0xE3EEF9, 1.0),
                        (1.0, 0xB7CDE6, 1.0),
                    ],
                );
                s.fill(&d, &g);
                haze(&mut s, w, h * base, h * 0.05, 0xFFFFFF, 0.7);
            }
            // замёрзшее озеро
            let lake = s.vgrad(hz - 10.0, hz + 30.0, 0xBFE3FA, 0x8EC5EC);
            s.ellipse(w * 0.62, hz + 6.0, w * 0.34, 22.0, &lake, None);
            s.line(
                &format!(
                    "M{} {} L{} {} M{} {} L{} {}",
                    w * 0.45,
                    hz + 4.0,
                    w * 0.58,
                    hz + 4.0,
                    w * 0.66,
                    hz + 12.0,
                    w * 0.8,
                    hz + 12.0
                ),
                0xFFFFFF,
                2.0,
                0.8,
            );
            for layer in 0..2 {
                for _ in 0..(10 + layer * 6) {
                    let x = rng.range(-20.0, w + 20.0);
                    if x > w * 0.3 && x < w * 0.9 && layer == 1 {
                        continue;
                    }
                    let th = rng.range(50.0, 90.0) * (0.7 + layer as f32 * 0.3);
                    let b = hz + layer as f32 * 18.0 - 6.0;
                    pine(
                        &mut s,
                        x,
                        b,
                        th,
                        if layer == 0 { 0x5B7F8C } else { 0x2F5D50 },
                    );
                }
            }
            ground(&mut s, w, h, hz + h * 0.04, 0xF6FAFF, 0xD3E4F4);
            let mut drifts = String::new();
            for _ in 0..10 {
                let _ = write!(
                    drifts,
                    r##"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="#c3d8ee" opacity="0.5"/>"##,
                    rng.range(0.0, w),
                    rng.range(hz + 40.0, h),
                    rng.range(40.0, 90.0),
                    rng.range(6.0, 12.0)
                );
            }
            s.raw(&drifts);
        }
        // ---------------- Огненный вулкан
        5 => {
            sky(
                &mut s,
                w,
                h,
                &[
                    (0.0, 0x3A1010, 1.0),
                    (0.5, 0xA8331D, 1.0),
                    (1.0, 0xFF9A4D, 1.0),
                ],
            );
            let b = s.blur(18.0);
            let mut smoke = String::new();
            for _ in 0..14 {
                let _ = write!(
                    smoke,
                    r##"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="#3a2a2a" opacity="0.5"/>"##,
                    w * 0.62 + rng.range(-120.0, 160.0),
                    rng.range(h * 0.04, h * 0.3),
                    rng.range(40.0, 80.0)
                );
            }
            s.raw(&format!(r#"<g filter="{b}">{smoke}</g>"#));
            let far = peaks(w, h, h * 0.6, h * 0.12, 8, &mut rng);
            s.fill(&far, "#5a2a22");
            // вулкан
            let cx = w * 0.62;
            let cone = format!(
                "M{} {hz} L{} {} Q{cx} {} {} {} L{} {hz} Z",
                cx - w * 0.4,
                cx - w * 0.08,
                h * 0.3,
                h * 0.28,
                cx + w * 0.08,
                h * 0.3,
                cx + w * 0.42
            );
            let g = s.lin(
                cx - w * 0.3,
                0.0,
                cx + w * 0.3,
                0.0,
                &[(0.0, 0x2E1F1F, 1.0), (1.0, 0x4A2E26, 1.0)],
            );
            s.fill(&cone, &g);
            glow(&mut s, cx, h * 0.3, h * 0.12, 0xFF6F00, 0.9);
            let b2 = s.blur(3.0);
            let lava = s.vgrad(h * 0.3, hz, 0xFFEB3B, 0xFF3D00);
            for dx in [-0.05f32, 0.02, 0.06] {
                let x0 = cx + w * dx * 0.6;
                let d = format!(
                    "M{} {} C{} {} {} {} {} {hz}",
                    x0,
                    h * 0.31,
                    x0 + w * dx * 2.0,
                    h * 0.4,
                    x0 + w * dx * 4.0,
                    h * 0.5,
                    x0 + w * dx * 6.0
                );
                s.raw(&format!(r#"<path d="{d}" fill="none" stroke="{lava}" stroke-width="12" opacity="0.6" filter="{b2}"/><path d="{d}" fill="none" stroke="{lava}" stroke-width="5"/>"#));
            }
            lights.push(Light {
                x: cx,
                y: h * 0.3,
                r: 80.0,
                color: 0xFF9800,
            });
            ground(&mut s, w, h, hz + h * 0.02, 0x4A3430, 0x241816);
            let mut cracks = String::new();
            for _ in 0..8 {
                let x = rng.range(0.0, w);
                let y = rng.range(hz + 40.0, h);
                let _ = write!(
                    cracks,
                    "M{x:.1} {y:.1} l{:.1} {:.1} l{:.1} {:.1} l{:.1} {:.1} ",
                    rng.range(20.0, 50.0),
                    rng.range(-8.0, 8.0),
                    rng.range(10.0, 40.0),
                    rng.range(-10.0, 10.0),
                    rng.range(20.0, 40.0),
                    rng.range(-6.0, 6.0)
                );
            }
            s.raw(&format!(r##"<path d="{cracks}" fill="none" stroke="#ff7043" stroke-width="7" opacity="0.5" filter="{b2}"/><path d="{cracks}" fill="none" stroke="#ffcc80" stroke-width="2.4"/>"##));
            let mut rocks = String::new();
            for _ in 0..7 {
                let x = rng.range(0.0, w);
                let y = rng.range(hz + 20.0, h * 0.95);
                let r = rng.range(14.0, 34.0);
                let _ = write!(
                    rocks,
                    "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Z ",
                    x - r,
                    y,
                    x - r * 0.4,
                    y - r * 0.9,
                    x + r * 0.5,
                    y - r * 0.7,
                    x + r,
                    y
                );
            }
            s.fill(&rocks, "#2a1c1a");
        }
        // ---------------- Небесный дворец
        _ => {
            sky(
                &mut s,
                w,
                h,
                &[
                    (0.0, 0xFFD27A, 1.0),
                    (0.5, 0xFFEFC6, 1.0),
                    (1.0, 0xFFF8EA, 1.0),
                ],
            );
            let mut rays = String::new();
            for i in 0..12 {
                let a = std::f32::consts::PI * (0.1 + i as f32 * 0.07);
                let (sn, cs) = a.sin_cos();
                let _ = write!(
                    rays,
                    "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Z ",
                    w * 0.5,
                    -h * 0.1,
                    w * 0.5 + cs * h * 2.0 - sn * 40.0,
                    -h * 0.1 + sn * h * 2.0,
                    w * 0.5 + cs * h * 2.0 + sn * 40.0,
                    -h * 0.1 + sn * h * 2.0
                );
            }
            s.fill_a(&rays, 0xFFFFFF, 0.18);
            // радуга
            let b = s.blur(4.0);
            for (i, c) in [0xFF8A80, 0xFFD180, 0xFFFF8D, 0xB9F6CA, 0x80D8FF, 0xB388FF]
                .iter()
                .enumerate()
            {
                let r = h * 0.5 - i as f32 * 8.0;
                s.raw(&format!(r#"<path d="M{} {} A{r} {r} 0 0 1 {} {}" fill="none" stroke="{}" stroke-width="8" opacity="0.35" filter="{b}"/>"#, w * 0.2 - r * 0.5, hz, w * 0.2 + r * 1.5, hz, hex(*c)));
            }
            // парящие острова с дворцами
            for (x, y, sc) in [(w * 0.22, h * 0.36, 0.8), (w * 0.72, h * 0.26, 1.1)] {
                island(&mut s, x, y, sc, &mut lights);
            }
            // море облаков
            let b = s.blur(5.0);
            let mut sea = String::new();
            for layer in 0..3 {
                let y = hz - 30.0 + layer as f32 * 26.0;
                let mut x = -40.0;
                while x < w + 60.0 {
                    let r = rng.range(30.0, 60.0);
                    let c = [0xFFF3E0, 0xFFFFFF, 0xFFF8E1][layer];
                    let _ = write!(
                        sea,
                        r#"<circle cx="{x:.1}" cy="{:.1}" r="{r:.1}" fill="{}"/>"#,
                        y + rng.range(-8.0, 8.0),
                        hex(c)
                    );
                    x += r * 1.1;
                }
            }
            s.raw(&format!(r#"<g filter="{b}">{sea}</g>"#));
            ground(&mut s, w, h, hz + h * 0.05, 0xFFFBF2, 0xF5E6C8);
            let mut puffs = String::new();
            for _ in 0..16 {
                let _ = write!(
                    puffs,
                    r##"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="#ffffff" opacity="0.7"/>"##,
                    rng.range(0.0, w),
                    rng.range(hz + 50.0, h),
                    rng.range(16.0, 34.0)
                );
            }
            s.raw(&format!(r#"<g filter="{b}">{puffs}</g>"#));
        }
    }
    // мягкая виньетка у горизонта
    haze(
        &mut s,
        w,
        hz + h * 0.02,
        h * 0.03,
        0xFFFFFF,
        if zone == 3 || zone == 5 { 0.06 } else { 0.25 },
    );
    (s.finish(), lights)
}

fn pine(s: &mut Svg, x: f32, base: f32, th: f32, c: u32) {
    let mut d = String::new();
    for i in 0..3 {
        let y = base - th * (0.15 + i as f32 * 0.28);
        let half = th * (0.32 - i as f32 * 0.08);
        let _ = write!(
            d,
            "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Z ",
            x - half,
            y,
            x,
            y - th * 0.4,
            x + half,
            y
        );
    }
    s.fill(&d, &hex(c));
    let mut snow = String::new();
    for i in 0..3 {
        let y = base - th * (0.15 + i as f32 * 0.28);
        let half = th * (0.32 - i as f32 * 0.08);
        let _ = write!(
            snow,
            "M{:.1} {:.1} L{:.1} {:.1} L{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1} Z ",
            x - half * 0.55,
            y - th * 0.2,
            x,
            y - th * 0.4,
            x + half * 0.55,
            y - th * 0.2,
            x,
            y - th * 0.14,
            x - half * 0.55,
            y - th * 0.2
        );
    }
    s.fill(&snow, "#ffffff");
    s.raw(&format!(
        r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#4e342e"/>"##,
        x - th * 0.04,
        base - th * 0.16,
        th * 0.08,
        th * 0.16
    ));
}

fn island(s: &mut Svg, x: f32, y: f32, sc: f32, lights: &mut Vec<Light>) {
    let w = 160.0 * sc;
    let rock = s.vgrad(y, y + w * 0.6, 0xC9A27A, 0x8D6E63);
    s.fill(
        &format!(
            "M{} {y} L{} {y} L{} {} L{x} {} L{} {} Z",
            x - w / 2.0,
            x + w / 2.0,
            x + w * 0.2,
            y + w * 0.35,
            y + w * 0.6,
            x - w * 0.25,
            y + w * 0.3
        ),
        &rock,
    );
    s.raw(&format!(
        r##"<ellipse cx="{x}" cy="{y}" rx="{}" ry="{}" fill="#9ccc65"/>"##,
        w / 2.0 + 4.0,
        w * 0.07
    ));
    // дворец
    let pw = w * 0.5;
    s.raw(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#d84335"/><rect x="{}" y="{}" width="{}" height="{}" fill="#fff3e0"/>"##,
        x - pw * 0.4, y - pw * 0.42, pw * 0.8, pw * 0.42, x - pw * 0.32, y - pw * 0.36, pw * 0.64, pw * 0.26
    ));
    for (yy, ww) in [(y - pw * 0.42, pw * 1.2), (y - pw * 0.72, pw * 0.8)] {
        let gold = s.vgrad(yy - pw * 0.2, yy, 0xFFE58A, 0xD4A017);
        s.fill(
            &format!(
                "M{} {yy} Q{x} {} {} {yy} L{} {} L{} {} Z",
                x - ww / 2.0,
                yy - pw * 0.08,
                x + ww / 2.0,
                x + ww * 0.3,
                yy - pw * 0.22,
                x - ww * 0.3,
                yy - pw * 0.22
            ),
            &gold,
        );
    }
    s.raw(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#d84335"/>"##,
        x - pw * 0.25,
        y - pw * 0.72,
        pw * 0.5,
        pw * 0.3
    ));
    lights.push(Light {
        x,
        y: y - pw * 0.3,
        r: 40.0 * sc,
        color: 0xFFE082,
    });
}

fn grass_and_petals(s: &mut Svg, w: f32, h: f32, hz: f32, grass: u32, petal: u32, rng: &mut Rng) {
    let mut tufts = String::new();
    for _ in 0..40 {
        let x = rng.range(0.0, w);
        let y = rng.range(hz + 20.0, h);
        let sc = 0.6 + (y - hz) / (h - hz);
        let _ = write!(tufts, "M{x:.1} {y:.1} q{:.1} {:.1} {:.1} {:.1} M{x:.1} {y:.1} q{:.1} {:.1} {:.1} {:.1} M{x:.1} {y:.1} q{:.1} {:.1} {:.1} {:.1} ", -3.0 * sc, -8.0 * sc, -6.0 * sc, -14.0 * sc, 1.0 * sc, -9.0 * sc, 2.0 * sc, -17.0 * sc, 4.0 * sc, -7.0 * sc, 7.0 * sc, -12.0 * sc);
    }
    s.line(&tufts, grass, 2.0, 0.7);
    let mut petals = String::new();
    for _ in 0..40 {
        let _ = write!(
            petals,
            r#"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="{}" opacity="0.8" transform="rotate({:.0} {:.1} {:.1})"/>"#,
            rng.range(0.0, w),
            rng.range(hz + 10.0, h),
            rng.range(2.5, 4.5),
            rng.range(1.5, 2.5),
            hex(petal),
            rng.range(0.0, 180.0),
            0.0,
            0.0
        );
    }
    s.raw(&petals);
}

/// Передний слой: ветви сверху и кромка травы снизу.
pub fn foreground(zone: usize, w: f32, h: f32) -> String {
    let mut s = Svg::new(w, h);
    let mut rng = Rng::new(zone as u64 * 31 + 7);
    let (branch, bloom, grass) = match zone {
        0 => (0x4E342E, 0xF48FB1, 0x7CB342),
        1 => (0x33691E, 0x66BB6A, 0x4E8F3F),
        2 => (0x3E2723, 0xF8BBD0, 0x6A5A8E),
        3 => (0x1B1030, 0xFF8A65, 0x2A2440),
        4 => (0x37474F, 0xFFFFFF, 0xB0C4D8),
        5 => (0x1E1212, 0xFF7043, 0x3A2420),
        _ => (0x8D6E63, 0xFFD54F, 0xFFF3D6),
    };
    // ветвь из левого верхнего угла
    s.line(
        &format!(
            "M-20 {} C{} {} {} {} {} {}",
            h * 0.02,
            w * 0.12,
            h * 0.06,
            w * 0.24,
            h * 0.02,
            w * 0.4,
            h * 0.09
        ),
        branch,
        16.0,
        1.0,
    );
    s.line(
        &format!(
            "M{} {} C{} {} {} {} {} {}",
            w * 0.12,
            h * 0.05,
            w * 0.16,
            h * 0.12,
            w * 0.2,
            h * 0.14,
            w * 0.27,
            h * 0.17
        ),
        branch,
        8.0,
        1.0,
    );
    s.line(
        &format!(
            "M{} {} C{} {} {} {} {} {}",
            w * 0.28,
            h * 0.04,
            w * 0.32,
            h * 0.0,
            w * 0.36,
            h * 0.0,
            w * 0.42,
            h * 0.02
        ),
        branch,
        7.0,
        1.0,
    );
    // ветвь из правого верхнего угла
    s.line(
        &format!(
            "M{} {} C{} {} {} {} {} {}",
            w + 20.0,
            h * 0.0,
            w * 0.9,
            h * 0.05,
            w * 0.84,
            h * 0.02,
            w * 0.74,
            h * 0.06
        ),
        branch,
        12.0,
        1.0,
    );
    let spots: Vec<(f32, f32)> = vec![
        (0.02, 0.02),
        (0.08, 0.06),
        (0.14, 0.04),
        (0.2, 0.08),
        (0.18, 0.14),
        (0.26, 0.16),
        (0.24, 0.03),
        (0.32, 0.06),
        (0.4, 0.09),
        (0.36, 0.02),
        (0.06, 0.0),
        (0.98, 0.02),
        (0.9, 0.05),
        (0.84, 0.03),
        (0.76, 0.07),
        (0.8, 0.0),
    ];
    let mut blobs = String::new();
    for (fx, fy) in spots.iter() {
        for _ in 0..4 {
            let x = w * fx + rng.range(-24.0, 24.0);
            let y = h * fy + rng.range(-14.0, 14.0);
            let r = rng.range(10.0, 20.0);
            let c = [bloom, tint(bloom, 0.35), shade(bloom, 0.9)][(rng.f() * 3.0) as usize % 3];
            let _ = write!(
                blobs,
                r#"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{}" opacity="0.95"/>"#,
                hex(c)
            );
        }
    }
    s.raw(&blobs);
    if zone == 0 || zone == 2 || zone == 6 {
        for (fx, fy) in spots.iter().step_by(2) {
            blossom(
                &mut s,
                w * fx + rng.range(-10.0, 10.0),
                h * fy + rng.range(-8.0, 8.0),
                9.0,
                tint(bloom, 0.5),
                0xE91E63,
                rng.f() * 3.0,
            );
        }
    }
    if zone == 3 {
        // большие фонари у края
        for x in [w * 0.06, w * 0.94] {
            s.line(&format!("M{x} 0 L{x} {}", h * 0.08), 0x1B1030, 2.0, 1.0);
            let g = s.rad(
                x - 4.0,
                h * 0.13,
                40.0,
                &[
                    (0.0, 0xFFF3C4, 1.0),
                    (0.6, 0xFF7043, 1.0),
                    (1.0, 0xB71C1C, 1.0),
                ],
            );
            s.ellipse(x, h * 0.13, 22.0, 30.0, &g, Some((0x5D1A1A, 2.0)));
        }
    }
    // кромка травы / камней внизу
    let mut d = format!("M0 {h} ");
    let mut x = 0.0;
    while x <= w {
        let y = h - rng.range(4.0, 22.0);
        let _ = write!(
            d,
            "L{x:.1} {y:.1} L{:.1} {:.1} ",
            x + 6.0,
            h - rng.range(0.0, 6.0)
        );
        x += rng.range(8.0, 16.0);
    }
    let _ = write!(d, "L{w} {h} Z");
    s.fill_a(&d, grass, 0.9);
    s.finish()
}
