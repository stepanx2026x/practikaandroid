//! Низкоуровневая отрисовка: шейдеры, текст, скруглённые панели, свечение.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

use crate::art::Sprite;

const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

/// Текстуры с premultiplied alpha (так получаются из resvg) — без тёмной каймы по краям.
const FRAG_PREMUL: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 t = texture2D(Texture, uv);
    gl_FragColor = vec4(t.rgb * color.rgb, t.a) * color.a;
}"#;

/// Силуэт спрайта одним цветом (вспышка удара, тени, закрытые героини).
const FRAG_SILHOUETTE: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    float a = texture2D(Texture, uv).a * color.a;
    gl_FragColor = vec4(color.rgb * a, a);
}"#;

/// Аддитивное свечение.
const FRAG_ADD: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 t = texture2D(Texture, uv);
    gl_FragColor = vec4(t.rgb * color.rgb * color.a, t.a * color.a);
}"#;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Bold,
    Black,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

pub struct Gfx {
    premul: Material,
    silhouette: Material,
    additive: Material,
    bold: macroquad::text::Font,
    black: macroquad::text::Font,
    pub glow: Texture2D,
    /// Логические единицы → пиксели окна.
    pub k: f32,
    pub dpi: f32,
    pub canvas_w: f32,
    pub canvas_h: f32,
}

fn material(frag: &str, blend: BlendState) -> Material {
    load_material(
        ShaderSource::Glsl {
            vertex: VERT,
            fragment: frag,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(blend),
                alpha_blend: Some(blend),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("shader")
}

pub fn hexc(c: u32) -> Color {
    Color::from_rgba(
        ((c >> 16) & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        (c & 0xFF) as u8,
        255,
    )
}

pub fn hexa(c: u32, a: f32) -> Color {
    let mut col = hexc(c);
    col.a = a;
    col
}

pub fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a *= a;
    c
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

impl Gfx {
    pub fn new() -> Self {
        let premul_blend = BlendState::new(
            Equation::Add,
            BlendFactor::One,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        );
        let add_blend = BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::One);
        let bold = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Nunito-Bold.ttf"))
            .expect("font");
        let black = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Nunito-Black.ttf"))
            .expect("font");

        // мягкое круглое свечение (premultiplied)
        let n = 128usize;
        let mut px = vec![0u8; n * n * 4];
        for y in 0..n {
            for x in 0..n {
                let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                let d = (dx * dx + dy * dy).sqrt().min(1.0);
                let a = ((1.0 - d).powf(2.2) * 255.0) as u8;
                let i = (y * n + x) * 4;
                px[i..i + 4].copy_from_slice(&[a, a, a, a]);
            }
        }
        let glow = Texture2D::from_rgba8(n as u16, n as u16, &px);
        glow.set_filter(FilterMode::Linear);

        Gfx {
            premul: material(FRAG_PREMUL, premul_blend),
            silhouette: material(FRAG_SILHOUETTE, premul_blend),
            additive: material(FRAG_ADD, add_blend),
            bold,
            black,
            glow,
            k: 1.0,
            dpi: 1.0,
            canvas_w: 1280.0,
            canvas_h: 800.0,
        }
    }

    /// Пересчитывает масштаб под окно и ставит камеру в логических координатах.
    pub fn begin_frame(&mut self) {
        let (sw, sh) = (screen_width(), screen_height());
        self.dpi = screen_dpi_scale();
        self.k = (sw / 1280.0).min(sh / 800.0).max(0.1);
        self.canvas_w = sw / self.k;
        self.canvas_h = sh / self.k;
        let mut cam =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, self.canvas_w, self.canvas_h));
        cam.zoom.y = -cam.zoom.y;
        set_camera(&cam);
    }

    /// Физических пикселей на логическую единицу.
    pub fn pixel_scale(&self) -> f32 {
        self.k * self.dpi
    }

    pub fn mouse(&self) -> Vec2 {
        let (x, y) = mouse_position();
        vec2(x / self.k, y / self.k)
    }

    // ------------------------------------------------------------- спрайты

    #[allow(clippy::too_many_arguments)]
    pub fn sprite_ex(
        &self,
        s: &Sprite,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        rotation: f32,
        pivot: Option<Vec2>,
        flip_x: bool,
    ) {
        gl_use_material(&self.premul);
        draw_texture_ex(
            &s.tex,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                rotation,
                pivot,
                flip_x,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// Спрайт с центром в (cx, cy) и масштабом.
    pub fn sprite(&self, s: &Sprite, cx: f32, cy: f32, scale: f32, color: Color) {
        let (w, h) = (s.w * scale, s.h * scale);
        self.sprite_ex(s, cx - w / 2.0, cy - h / 2.0, w, h, color, 0.0, None, false);
    }

    pub fn silhouette_ex(
        &self,
        s: &Sprite,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
        flip_x: bool,
    ) {
        gl_use_material(&self.silhouette);
        draw_texture_ex(
            &s.tex,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                flip_x,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// Мягкое аддитивное свечение.
    pub fn glow(&self, cx: f32, cy: f32, r: f32, color: Color) {
        gl_use_material(&self.additive);
        draw_texture_ex(
            &self.glow,
            cx - r,
            cy - r,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(r * 2.0, r * 2.0)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn glow_ellipse(&self, cx: f32, cy: f32, rx: f32, ry: f32, color: Color) {
        gl_use_material(&self.additive);
        draw_texture_ex(
            &self.glow,
            cx - rx,
            cy - ry,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(rx * 2.0, ry * 2.0)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// Мягкое затемнение (тень) — то же свечение, но обычным смешиванием.
    pub fn soft_shadow(&self, cx: f32, cy: f32, rx: f32, ry: f32, alpha: f32) {
        gl_use_material(&self.premul);
        draw_texture_ex(
            &self.glow,
            cx - rx,
            cy - ry,
            Color::new(0.0, 0.0, 0.0, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(rx * 2.0, ry * 2.0)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    pub fn additive_mesh(&self, mesh: &Mesh) {
        gl_use_material(&self.additive);
        draw_mesh(mesh);
        gl_use_default_material();
    }

    // ------------------------------------------------------------- текст

    fn font(&self, f: Font) -> &macroquad::text::Font {
        match f {
            Font::Bold => &self.bold,
            Font::Black => &self.black,
        }
    }

    /// Размер растеризации шрифта в пикселях. Округляется до фиксированного набора:
    /// атлас глифов macroquad растёт без ограничений, и если каждый кадр просить новый
    /// размер (анимированные цифры), он переполняет максимальный размер текстуры.
    fn px(&self, size: f32) -> u16 {
        const SIZES: [u16; 16] = [
            10, 12, 14, 16, 18, 20, 23, 26, 30, 34, 40, 48, 58, 70, 86, 108,
        ];
        let target = size * self.k * self.dpi;
        for s in SIZES {
            if s as f32 >= target * 0.97 {
                return s;
            }
        }
        SIZES[SIZES.len() - 1]
    }

    pub fn measure(&self, text: &str, size: f32, f: Font) -> f32 {
        let px = self.px(size);
        measure_text(text, Some(self.font(f)), px, size / px as f32).width
    }

    /// Текст; `cy` — вертикальный центр строки.
    pub fn text(
        &self,
        text: &str,
        x: f32,
        cy: f32,
        size: f32,
        f: Font,
        color: Color,
        align: Align,
    ) -> f32 {
        let px = self.px(size);
        let scale = size / px as f32;
        let font = self.font(f);
        let w = measure_text(text, Some(font), px, scale).width;
        let x0 = match align {
            Align::Left => x,
            Align::Center => x - w / 2.0,
            Align::Right => x - w,
        };
        draw_text_ex(
            text,
            x0,
            cy + size * 0.36,
            TextParams {
                font: Some(font),
                font_size: px,
                font_scale: scale,
                color,
                ..Default::default()
            },
        );
        w
    }

    /// Текст с обводкой.
    #[allow(clippy::too_many_arguments)]
    pub fn text_outline(
        &self,
        text: &str,
        x: f32,
        cy: f32,
        size: f32,
        f: Font,
        color: Color,
        outline: Color,
        width: f32,
        align: Align,
    ) -> f32 {
        let steps = 12;
        for i in 0..steps {
            let a = i as f32 / steps as f32 * std::f32::consts::TAU;
            self.text(
                text,
                x + a.cos() * width,
                cy + a.sin() * width,
                size,
                f,
                outline,
                align,
            );
        }
        self.text(text, x, cy, size, f, color, align)
    }

    /// Переносит текст по словам в ширину `max_w`. Возвращает высоту.
    #[allow(clippy::too_many_arguments)]
    pub fn text_wrap(
        &self,
        text: &str,
        x: f32,
        top: f32,
        max_w: f32,
        size: f32,
        f: Font,
        color: Color,
        line_h: f32,
    ) -> f32 {
        let mut y = top + size * 0.6;
        let mut line = String::new();
        for word in text.split(' ') {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if self.measure(&candidate, size, f) > max_w && !line.is_empty() {
                self.text(&line, x, y, size, f, color, Align::Left);
                y += line_h;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            self.text(&line, x, y, size, f, color, Align::Left);
        }
        y - top + line_h * 0.4
    }

    // ------------------------------------------------------------- фигуры

    /// Обрезка рисования прямоугольником (в логических координатах).
    pub fn clip(&self, r: Option<Rect>) {
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        let s = self.k * self.dpi;
        gl.quad_gl.scissor(r.map(|r| {
            (
                (r.x * s).floor() as i32,
                (r.y * s).floor() as i32,
                (r.w * s).ceil() as i32,
                (r.h * s).ceil() as i32,
            )
        }));
    }
}

/// Скруглённый прямоугольник с вертикальным градиентом.
pub fn rrect_grad(x: f32, y: f32, w: f32, h: f32, r: f32, top: Color, bottom: Color) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let seg = ((r * 0.6) as usize).clamp(3, 12);
    let mut verts = Vec::with_capacity(4 * (seg + 1) + 1);
    let col = |py: f32| lerp_color(top, bottom, ((py - y) / h).clamp(0.0, 1.0));
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    verts.push(Vertex::new(cx, cy, 0.0, 0.0, 0.0, col(cy)));
    let corners = [
        (x + w - r, y + r, -90.0f32),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, 90.0),
        (x + r, y + r, 180.0),
    ];
    for (ccx, ccy, start) in corners {
        for i in 0..=seg {
            let a = (start + 90.0 * i as f32 / seg as f32).to_radians();
            let px = ccx + a.cos() * r;
            let py = ccy + a.sin() * r;
            verts.push(Vertex::new(px, py, 0.0, 0.0, 0.0, col(py)));
        }
    }
    let n = verts.len() as u16;
    let mut indices = Vec::with_capacity((n as usize) * 3);
    for i in 1..n {
        let next = if i + 1 < n { i + 1 } else { 1 };
        indices.extend_from_slice(&[0, i, next]);
    }
    draw_mesh(&Mesh {
        vertices: verts,
        indices,
        texture: None,
    });
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    rrect_grad(x, y, w, h, r, c, c);
}

/// Контур скруглённого прямоугольника.
pub fn rrect_line(x: f32, y: f32, w: f32, h: f32, r: f32, thick: f32, c: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    let seg = ((r * 0.6) as usize).clamp(3, 12);
    let mut pts = Vec::new();
    let corners = [
        (x + w - r, y + r, -90.0f32),
        (x + w - r, y + h - r, 0.0),
        (x + r, y + h - r, 90.0),
        (x + r, y + r, 180.0),
    ];
    for (ccx, ccy, start) in corners {
        for i in 0..=seg {
            let a = (start + 90.0 * i as f32 / seg as f32).to_radians();
            pts.push(vec2(ccx + a.cos() * r, ccy + a.sin() * r));
        }
    }
    // толстая линия как полоса треугольников
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let n = pts.len();
    for i in 0..n {
        let p = pts[i];
        let prev = pts[(i + n - 1) % n];
        let next = pts[(i + 1) % n];
        let dir = (next - prev).normalize_or_zero();
        let normal = vec2(-dir.y, dir.x) * (thick / 2.0);
        verts.push(Vertex::new(
            p.x + normal.x,
            p.y + normal.y,
            0.0,
            0.0,
            0.0,
            c,
        ));
        verts.push(Vertex::new(
            p.x - normal.x,
            p.y - normal.y,
            0.0,
            0.0,
            0.0,
            c,
        ));
    }
    for i in 0..n {
        let a = (i * 2) as u16;
        let b = (((i + 1) % n) * 2) as u16;
        indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
    }
    draw_mesh(&Mesh {
        vertices: verts,
        indices,
        texture: None,
    });
}

/// Мягкая тень под панелью.
pub fn shadow(x: f32, y: f32, w: f32, h: f32, r: f32, spread: f32, alpha: f32) {
    let layers = 6;
    for i in 0..layers {
        let t = i as f32 / layers as f32;
        let e = spread * (1.0 - t);
        rrect(
            x - e,
            y - e + spread * 0.35,
            w + e * 2.0,
            h + e * 2.0,
            r + e,
            Color::new(0.24, 0.08, 0.16, alpha / layers as f32 * (0.4 + t)),
        );
    }
}

/// Кольцевой сектор (для перезарядки навыков): доля `frac` от полного круга, по часовой от 12 часов.
pub fn pie(cx: f32, cy: f32, r: f32, frac: f32, c: Color) {
    if frac <= 0.0 {
        return;
    }
    let seg = 48;
    let n = ((seg as f32 * frac).ceil() as usize).max(1);
    let mut verts = vec![Vertex::new(cx, cy, 0.0, 0.0, 0.0, c)];
    for i in 0..=n {
        let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * frac * (i as f32 / n as f32);
        verts.push(Vertex::new(
            cx + a.cos() * r,
            cy + a.sin() * r,
            0.0,
            0.0,
            0.0,
            c,
        ));
    }
    let mut indices = Vec::new();
    for i in 1..=n as u16 {
        indices.extend_from_slice(&[0, i, i + 1]);
    }
    draw_mesh(&Mesh {
        vertices: verts,
        indices,
        texture: None,
    });
}

/// Дуга-кольцо.
pub fn arc(cx: f32, cy: f32, r: f32, thick: f32, from: f32, to: f32, c: Color) {
    let seg = 48;
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=seg {
        let a = from + (to - from) * (i as f32 / seg as f32);
        let (s, co) = a.sin_cos();
        verts.push(Vertex::new(
            cx + co * (r - thick / 2.0),
            cy + s * (r - thick / 2.0),
            0.0,
            0.0,
            0.0,
            c,
        ));
        verts.push(Vertex::new(
            cx + co * (r + thick / 2.0),
            cy + s * (r + thick / 2.0),
            0.0,
            0.0,
            0.0,
            c,
        ));
    }
    for i in 0..seg as u16 {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 1, a + 2, a + 1, a + 3, a + 2]);
    }
    draw_mesh(&Mesh {
        vertices: verts,
        indices,
        texture: None,
    });
}

/// Прямоугольник с горизонтальным градиентом (для полосок здоровья).
pub fn hgrad(x: f32, y: f32, w: f32, h: f32, left: Color, right: Color) {
    let verts = vec![
        Vertex::new(x, y, 0.0, 0.0, 0.0, left),
        Vertex::new(x + w, y, 0.0, 0.0, 0.0, right),
        Vertex::new(x + w, y + h, 0.0, 0.0, 0.0, right),
        Vertex::new(x, y + h, 0.0, 0.0, 0.0, left),
    ];
    draw_mesh(&Mesh {
        vertices: verts,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

/// Вертикальный градиент на весь прямоугольник.
pub fn vgrad(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
    let verts = vec![
        Vertex::new(x, y, 0.0, 0.0, 0.0, top),
        Vertex::new(x + w, y, 0.0, 0.0, 0.0, top),
        Vertex::new(x + w, y + h, 0.0, 0.0, 0.0, bottom),
        Vertex::new(x, y + h, 0.0, 0.0, 0.0, bottom),
    ];
    draw_mesh(&Mesh {
        vertices: verts,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

/// Полоска прогресса со скруглёнными краями.
pub fn progress_bar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    frac: f32,
    back: Color,
    left: Color,
    right: Color,
) {
    rrect(x, y, w, h, h / 2.0, back);
    let fw = (w - 4.0) * frac.clamp(0.0, 1.0);
    if fw > 1.0 {
        let r = ((h - 4.0) / 2.0).min(fw / 2.0);
        rrect_grad(
            x + 2.0,
            y + 2.0,
            fw,
            h - 4.0,
            r,
            lerp_color(left, WHITE, 0.25),
            right,
        );
        rrect(
            x + 2.0 + r * 0.5,
            y + 3.0,
            (fw - r).max(0.0),
            (h - 4.0) * 0.32,
            (h - 4.0) * 0.16,
            Color::new(1.0, 1.0, 1.0, 0.35),
        );
    }
}
