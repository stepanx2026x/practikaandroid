//! Процедурная графика: SVG собирается в коде, растеризуется resvg в текстуры.
//! Так картинки остаются чёткими при любом размере окна.

pub mod chibi;
pub mod icons;
pub mod scenery;
pub mod yokai;

use std::collections::HashMap;
use std::fmt::Write;

use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// Цвета
// ---------------------------------------------------------------------------

pub fn hex(c: u32) -> String {
    format!("#{:06x}", c & 0xFF_FFFF)
}

fn channels(c: u32) -> (f32, f32, f32) {
    (
        ((c >> 16) & 0xFF) as f32,
        ((c >> 8) & 0xFF) as f32,
        (c & 0xFF) as f32,
    )
}

fn pack(r: f32, g: f32, b: f32) -> u32 {
    let cl = |v: f32| v.round().clamp(0.0, 255.0) as u32;
    (cl(r) << 16) | (cl(g) << 8) | cl(b)
}

/// Темнее: f < 1.
pub fn shade(c: u32, f: f32) -> u32 {
    let (r, g, b) = channels(c);
    pack(r * f, g * f, b * f)
}

/// Светлее: t — доля белого.
pub fn tint(c: u32, t: f32) -> u32 {
    mix(c, 0xFFFFFF, t)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let (r1, g1, b1) = channels(a);
    let (r2, g2, b2) = channels(b);
    pack(r1 + (r2 - r1) * t, g1 + (g2 - g1) * t, b1 + (b2 - b1) * t)
}

/// Контур: темнее и чуть насыщеннее.
pub fn ink(c: u32) -> u32 {
    mix(shade(c, 0.5), 0x3A1F2B, 0.35)
}

// ---------------------------------------------------------------------------
// Сборщик SVG
// ---------------------------------------------------------------------------

pub struct Svg {
    pub w: f32,
    pub h: f32,
    defs: String,
    body: String,
    next_id: u32,
}

impl Svg {
    pub fn new(w: f32, h: f32) -> Self {
        Svg {
            w,
            h,
            defs: String::new(),
            body: String::new(),
            next_id: 0,
        }
    }

    fn id(&mut self, p: &str) -> String {
        self.next_id += 1;
        format!("{}{}", p, self.next_id)
    }

    /// Линейный градиент, stops: (offset, цвет, прозрачность). Возвращает `url(#id)`.
    pub fn lin(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, stops: &[(f32, u32, f32)]) -> String {
        let id = self.id("l");
        let _ = write!(
            self.defs,
            r#"<linearGradient id="{id}" gradientUnits="userSpaceOnUse" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">"#
        );
        for (o, c, a) in stops {
            let _ = write!(
                self.defs,
                r#"<stop offset="{o}" stop-color="{}" stop-opacity="{a}"/>"#,
                hex(*c)
            );
        }
        self.defs.push_str("</linearGradient>");
        format!("url(#{id})")
    }

    /// Вертикальный градиент между двумя цветами.
    pub fn vgrad(&mut self, y1: f32, y2: f32, top: u32, bottom: u32) -> String {
        self.lin(0.0, y1, 0.0, y2, &[(0.0, top, 1.0), (1.0, bottom, 1.0)])
    }

    pub fn rad(&mut self, cx: f32, cy: f32, r: f32, stops: &[(f32, u32, f32)]) -> String {
        let id = self.id("r");
        let _ = write!(
            self.defs,
            r#"<radialGradient id="{id}" gradientUnits="userSpaceOnUse" cx="{cx}" cy="{cy}" r="{r}">"#
        );
        for (o, c, a) in stops {
            let _ = write!(
                self.defs,
                r#"<stop offset="{o}" stop-color="{}" stop-opacity="{a}"/>"#,
                hex(*c)
            );
        }
        self.defs.push_str("</radialGradient>");
        format!("url(#{id})")
    }

    /// Фильтр размытия. Возвращает `url(#id)`.
    pub fn blur(&mut self, sd: f32) -> String {
        let id = self.id("b");
        let _ = write!(
            self.defs,
            r#"<filter id="{id}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="{sd}"/></filter>"#
        );
        format!("url(#{id})")
    }

    /// Обрезка по контуру. Возвращает `url(#id)`.
    pub fn clip(&mut self, d: &str) -> String {
        let id = self.id("c");
        let _ = write!(
            self.defs,
            r#"<clipPath id="{id}"><path d="{d}"/></clipPath>"#
        );
        format!("url(#{id})")
    }

    pub fn raw(&mut self, s: &str) {
        self.body.push_str(s);
    }

    /// Контур с заливкой и обводкой.
    pub fn path(&mut self, d: &str, fill: &str, stroke: u32, sw: f32) {
        let _ = write!(
            self.body,
            r#"<path d="{d}" fill="{fill}" stroke="{}" stroke-width="{sw}" stroke-linejoin="round" stroke-linecap="round"/>"#,
            hex(stroke)
        );
    }

    pub fn fill(&mut self, d: &str, fill: &str) {
        let _ = write!(self.body, r#"<path d="{d}" fill="{fill}"/>"#);
    }

    pub fn fill_a(&mut self, d: &str, fill: u32, opacity: f32) {
        let _ = write!(
            self.body,
            r#"<path d="{d}" fill="{}" opacity="{opacity}"/>"#,
            hex(fill)
        );
    }

    pub fn line(&mut self, d: &str, stroke: u32, sw: f32, opacity: f32) {
        let _ = write!(
            self.body,
            r#"<path d="{d}" fill="none" stroke="{}" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round" opacity="{opacity}"/>"#,
            hex(stroke)
        );
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, fill: &str, stroke: Option<(u32, f32)>) {
        match stroke {
            Some((c, w)) => {
                let _ = write!(
                    self.body,
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{}" stroke-width="{w}"/>"#,
                    hex(c)
                );
            }
            None => {
                let _ = write!(
                    self.body,
                    r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}"/>"#
                );
            }
        }
    }

    pub fn ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        fill: &str,
        stroke: Option<(u32, f32)>,
    ) {
        match stroke {
            Some((c, w)) => {
                let _ = write!(
                    self.body,
                    r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{}" stroke-width="{w}"/>"#,
                    hex(c)
                );
            }
            None => {
                let _ = write!(
                    self.body,
                    r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}"/>"#
                );
            }
        }
    }

    pub fn open_group(&mut self, attrs: &str) {
        let _ = write!(self.body, "<g {attrs}>");
    }

    pub fn close_group(&mut self) {
        self.body.push_str("</g>");
    }

    pub fn finish(&self) -> String {
        self.finish_view(0.0, 0.0, self.w, self.h)
    }

    pub fn finish_view(&self, x: f32, y: f32, w: f32, h: f32) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{x} {y} {w} {h}"><defs>{}</defs>{}</svg>"#,
            self.defs, self.body
        )
    }
}

/// Цветок сакуры из пяти лепестков.
pub fn blossom(s: &mut Svg, cx: f32, cy: f32, r: f32, petal: u32, center: u32, rot: f32) {
    let mut d = String::new();
    for i in 0..5 {
        let a = rot + i as f32 * std::f32::consts::TAU / 5.0;
        let (sn, cs) = a.sin_cos();
        // лепесток с выемкой на конце
        let p = |dx: f32, dy: f32| (cx + dx * cs - dy * sn, cy + dx * sn + dy * cs);
        let (x0, y0) = p(0.0, 0.0);
        let (x1, y1) = p(-0.62 * r, -0.55 * r);
        let (x2, y2) = p(-0.42 * r, -1.02 * r);
        let (x3, y3) = p(0.0, -0.86 * r);
        let (x4, y4) = p(0.42 * r, -1.02 * r);
        let (x5, y5) = p(0.62 * r, -0.55 * r);
        let _ = write!(d, "M{x0:.2} {y0:.2} Q{x1:.2} {y1:.2} {x2:.2} {y2:.2} L{x3:.2} {y3:.2} L{x4:.2} {y4:.2} Q{x5:.2} {y5:.2} {x0:.2} {y0:.2} Z ");
    }
    let g = s.rad(
        cx,
        cy,
        r,
        &[
            (0.0, tint(petal, 0.0), 1.0),
            (0.35, petal, 1.0),
            (1.0, tint(petal, 0.6), 1.0),
        ],
    );
    s.path(&d, &g, shade(petal, 0.8), r * 0.06);
    s.circle(cx, cy, r * 0.22, &hex(center), None);
    for i in 0..5 {
        let a = rot + (i as f32 + 0.5) * std::f32::consts::TAU / 5.0;
        let (sn, cs) = a.sin_cos();
        s.circle(
            cx - sn * r * 0.38,
            cy + cs * r * 0.38,
            r * 0.06,
            &hex(0xF9C846),
            None,
        );
    }
}

// ---------------------------------------------------------------------------
// Растеризация и кеш текстур
// ---------------------------------------------------------------------------

/// Растеризует SVG в пиксели (RGBA, premultiplied alpha).
pub fn rasterize(svg: &str, px_w: u32, px_h: u32) -> Option<resvg::tiny_skia::Pixmap> {
    let opt = resvg::usvg::Options::default();
    let tree = match resvg::usvg::Tree::from_str(svg, &opt) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("SVG error: {e}");
            return None;
        }
    };
    let mut pm = resvg::tiny_skia::Pixmap::new(px_w.max(1), px_h.max(1))?;
    let size = tree.size();
    let tr = resvg::tiny_skia::Transform::from_scale(
        px_w as f32 / size.width(),
        px_h as f32 / size.height(),
    );
    resvg::render(&tree, tr, &mut pm.as_mut());
    Some(pm)
}

pub struct Sprite {
    pub tex: Texture2D,
    /// Размер в логических единицах экрана.
    pub w: f32,
    pub h: f32,
}

/// Кеш спрайтов. При изменении масштаба окна всё перерисовывается в новом разрешении.
pub struct Art {
    pub scale: f32,
    cache: HashMap<String, Sprite>,
}

impl Art {
    pub fn new(scale: f32) -> Self {
        Art {
            scale,
            cache: HashMap::new(),
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        if (scale - self.scale).abs() > 0.001 {
            self.scale = scale;
            self.cache.clear();
        }
    }

    /// Спрайт размера `w`×`h` логических единиц; `make` строит SVG при первом обращении.
    pub fn get(&mut self, key: &str, w: f32, h: f32, make: impl FnOnce() -> String) -> &Sprite {
        if !self.cache.contains_key(key) {
            let px_w = (w * self.scale).ceil().clamp(1.0, 4096.0) as u32;
            let px_h = (h * self.scale).ceil().clamp(1.0, 4096.0) as u32;
            let svg = make();
            let tex = match rasterize(&svg, px_w, px_h) {
                Some(pm) => Texture2D::from_rgba8(px_w as u16, px_h as u16, pm.data()),
                None => Texture2D::from_rgba8(1, 1, &[0, 0, 0, 0]),
            };
            tex.set_filter(FilterMode::Linear);
            self.cache.insert(key.to_string(), Sprite { tex, w, h });
        }
        &self.cache[key]
    }
}

/// Пиксели иконки окна (RGBA без premultiply).
pub fn icon_pixels<const N: usize>(size: u32) -> [u8; N] {
    let mut s = Svg::new(64.0, 64.0);
    let bg = s.rad(
        32.0,
        30.0,
        34.0,
        &[(0.0, 0xFFF0F5, 1.0), (1.0, 0xF8BBD0, 1.0)],
    );
    s.circle(32.0, 32.0, 31.0, &bg, Some((0xC2185B, 2.0)));
    blossom(&mut s, 32.0, 33.0, 24.0, 0xF48FB1, 0xC2185B, 0.0);
    let mut out = [0u8; N];
    if let Some(pm) = rasterize(&s.finish(), size, size) {
        for (i, p) in pm.pixels().iter().enumerate() {
            let c = p.demultiply();
            if i * 4 + 3 < N {
                out[i * 4] = c.red();
                out[i * 4 + 1] = c.green();
                out[i * 4 + 2] = c.blue();
                out[i * 4 + 3] = c.alpha();
            }
        }
    }
    out
}
