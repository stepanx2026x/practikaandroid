//! Правая панель: «Отряд», «Гардероб», «Храм», «Летопись», а также модальные окна.

use macroquad::prelude::*;

use crate::art::icons::{self, Icon};
use crate::art::{chibi, Art, Svg};
use crate::data::*;
use crate::fmt;
use crate::game::{Game, REBIRTH_MIN_STAGE};
use crate::gfx::*;
use crate::gfx::{Align, Font};
use crate::input::Input;
use crate::scene::Scene;

pub const PANEL_W: f32 = 470.0;
const TABS: [(&str, Icon); 4] = [
    ("Отряд", Icon::TabTeam),
    ("Гардероб", Icon::TabWardrobe),
    ("Храм", Icon::TabTemple),
    ("Летопись", Icon::TabChronicle),
];
const BUY_AMOUNTS: [(u32, &str); 5] = [
    (1, "×1"),
    (10, "×10"),
    (25, "×25"),
    (100, "×100"),
    (0, "Макс"),
];
const TEXT: u32 = 0x4A2C3A;
const MUTED: u32 = 0x9A7A88;
const PINK: u32 = 0xE0598B;
const PINK_DARK: u32 = 0xB83B6B;
const PURPLE: u32 = 0x8E5CC4;

pub enum Modal {
    Rebirth,
    Reset,
    Offline { gold: f64, secs: f64 },
    Welcome,
}

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Pink,
    Purple,
    Muted,
}

pub struct Panel {
    pub tab: usize,
    scroll: [f32; 4],
    target: [f32; 4],
    content_h: [f32; 4],
    drag: Option<(f32, f32)>,
    pub modal: Option<Modal>,
    time: f32,
}

impl Panel {
    pub fn new() -> Self {
        Panel {
            tab: 0,
            scroll: [0.0; 4],
            target: [0.0; 4],
            content_h: [0.0; 4],
            drag: None,
            modal: None,
            time: 0.0,
        }
    }

    pub fn draw(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        scene: &mut Scene,
        x0: f32,
        h: f32,
        dt: f32,
    ) {
        self.time += dt;
        let w = PANEL_W;

        // ---------- фон панели с узором сэйгайха
        hgrad(
            x0 - 26.0,
            0.0,
            26.0,
            h,
            Color::new(0.2, 0.05, 0.12, 0.0),
            Color::new(0.2, 0.05, 0.12, 0.22),
        );
        let key = format!("panel/bg/{}", h as i32);
        let bg = art.get(&key, w, h, || panel_bg(w, h));
        gfx.sprite_ex(bg, x0, 0.0, w, h, WHITE, 0.0, None, false);
        vgrad(x0, 0.0, 3.0, h, hexc(0xF06292), hexc(0xC2185B));

        // ---------- заголовок
        gfx.text_outline(
            "Сакура и ёкаи",
            x0 + w / 2.0,
            34.0,
            30.0,
            Font::Black,
            hexc(0xD81B60),
            WHITE,
            3.0,
            Align::Center,
        );
        let tw = gfx.measure("Сакура и ёкаи", 30.0, Font::Black);
        let blossom = art.get("icon/team/26", 26.0, 26.0, || {
            icons::icon_svg(Icon::TabTeam)
        });
        gfx.sprite(blossom, x0 + w / 2.0 - tw / 2.0 - 24.0, 34.0, 1.0, WHITE);
        gfx.sprite(blossom, x0 + w / 2.0 + tw / 2.0 + 24.0, 34.0, 1.0, WHITE);

        // ---------- вкладки
        let ty = 64.0;
        let tab_w = (w - 24.0 - 3.0 * 6.0) / 4.0;
        for (i, (name, icon)) in TABS.iter().enumerate() {
            let r = Rect::new(x0 + 12.0 + i as f32 * (tab_w + 6.0), ty, tab_w, 46.0);
            let sel = self.tab == i;
            let hover = input.hover(r);
            if sel {
                shadow(r.x, r.y, r.w, r.h, 16.0, 6.0, 0.35);
                rrect_grad(r.x, r.y, r.w, r.h, 16.0, hexc(0xF27BA5), hexc(0xC2185B));
            } else {
                rrect_grad(
                    r.x,
                    r.y,
                    r.w,
                    r.h,
                    16.0,
                    if hover {
                        hexc(0xFFFFFF)
                    } else {
                        hexa(0xFFFFFF, 0.75)
                    },
                    hexa(0xFCE4EC, 0.9),
                );
                rrect_line(r.x, r.y, r.w, r.h, 16.0, 1.5, hexc(0xF3C1D5));
            }
            let ik = format!("icon/tab{i}/20");
            let spr = art.get(&ik, 20.0, 20.0, || icons::icon_svg(*icon));
            let label_w = gfx.measure(name, 14.0, Font::Black);
            let start = r.x + r.w / 2.0 - (label_w + 24.0) / 2.0;
            gfx.sprite(spr, start + 10.0, r.y + r.h / 2.0, 1.0, WHITE);
            gfx.text(
                name,
                start + 24.0,
                r.y + r.h / 2.0,
                14.0,
                Font::Black,
                if sel { WHITE } else { hexc(PINK_DARK) },
                Align::Left,
            );
            if input.clicked(r) {
                self.tab = i;
            }
            // точка-подсказка: что-то можно купить
            if !sel && i == 2 && game.pending_souls() > 0 && game.rebirths == 0 {
                draw_circle(
                    r.x + r.w - 10.0,
                    r.y + 10.0,
                    5.0 + (self.time * 5.0).sin(),
                    hexc(0xFF5252),
                );
            }
        }

        // ---------- содержимое с прокруткой
        let view = Rect::new(x0, ty + 56.0, w, h - ty - 64.0);
        let t = self.tab;
        if input.hover(view) && input.wheel != 0.0 {
            self.target[t] -= input.wheel * 90.0;
        }
        let max_scroll = (self.content_h[t] - view.h + 16.0).max(0.0);
        // перетаскивание полосы прокрутки
        let track = Rect::new(x0 + w - 10.0, view.y + 4.0, 6.0, view.h - 8.0);
        let thumb_h = if self.content_h[t] > 0.0 {
            (view.h / self.content_h[t] * track.h).clamp(40.0, track.h)
        } else {
            track.h
        };
        let thumb_y = track.y
            + if max_scroll > 0.0 {
                self.scroll[t] / max_scroll * (track.h - thumb_h)
            } else {
                0.0
            };
        let thumb = Rect::new(track.x - 4.0, thumb_y, 14.0, thumb_h);
        if input.pressed_in(thumb) && max_scroll > 0.0 {
            self.drag = Some((input.mouse.y, self.scroll[t]));
        }
        if !input.down {
            self.drag = None;
        }
        if let Some((y0, s0)) = self.drag {
            self.target[t] = s0 + (input.mouse.y - y0) / (track.h - thumb_h).max(1.0) * max_scroll;
            self.scroll[t] = self.target[t].clamp(0.0, max_scroll);
        }
        self.target[t] = self.target[t].clamp(0.0, max_scroll);
        self.scroll[t] += (self.target[t] - self.scroll[t]) * (dt * 14.0).min(1.0);

        gfx.clip(Some(view));
        let top = view.y + 6.0 - self.scroll[t];
        let used = match t {
            0 => self.team_tab(game, gfx, art, input, x0, top, view),
            1 => self.wardrobe_tab(game, gfx, art, input, x0, top, view),
            2 => self.temple_tab(game, gfx, art, input, x0, top, view, scene),
            _ => self.chronicle_tab(game, gfx, art, input, x0, top, view),
        };
        self.content_h[t] = used;
        gfx.clip(None);
        // мягкие края прокрутки
        vgrad(
            x0 + 3.0,
            view.y,
            w - 3.0,
            10.0,
            hexa(0xFDE8F0, 0.9),
            hexa(0xFDE8F0, 0.0),
        );
        if max_scroll > 0.0 {
            rrect(
                track.x,
                track.y,
                track.w,
                track.h,
                3.0,
                hexa(0xC2185B, 0.12),
            );
            rrect(
                track.x,
                thumb_y,
                track.w,
                thumb_h,
                3.0,
                hexa(
                    0xC2185B,
                    if self.drag.is_some() || input.hover(thumb) {
                        0.6
                    } else {
                        0.35
                    },
                ),
            );
        }
    }

    // ---------------------------------------------------------------- кнопки

    #[allow(clippy::too_many_arguments)]
    fn button(
        &self,
        gfx: &Gfx,
        input: &Input,
        r: Rect,
        enabled: bool,
        style: Style,
        view: Rect,
        label: &str,
        size: f32,
    ) -> bool {
        let hover = enabled && input.hover(r) && view.contains(input.mouse);
        let pressed = hover && input.down;
        let (top, bottom) = match (enabled, style) {
            (false, _) => (hexc(0xEEDDE5), hexc(0xE3CCD7)),
            (true, Style::Pink) => (hexc(0xF58BB0), hexc(0xD63F78)),
            (true, Style::Purple) => (hexc(0xB38BEB), hexc(0x7E4DBA)),
            (true, Style::Muted) => (hexc(0xC9B6C0), hexc(0xA48E9A)),
        };
        let lift = if pressed {
            1.0
        } else if hover {
            -1.0
        } else {
            0.0
        };
        if enabled {
            shadow(
                r.x,
                r.y + lift,
                r.w,
                r.h,
                r.h.min(30.0) / 2.0,
                if hover { 7.0 } else { 5.0 },
                0.35,
            );
        }
        let (top, bottom) = if hover {
            (lerp_color(top, WHITE, 0.15), lerp_color(bottom, WHITE, 0.1))
        } else {
            (top, bottom)
        };
        rrect_grad(r.x, r.y + lift, r.w, r.h, r.h.min(30.0) / 2.0, top, bottom);
        if enabled {
            rrect(
                r.x + 8.0,
                r.y + lift + 3.0,
                r.w - 16.0,
                r.h * 0.28,
                r.h * 0.14,
                Color::new(1.0, 1.0, 1.0, 0.25),
            );
        }
        if !label.is_empty() {
            gfx.text(
                label,
                r.x + r.w / 2.0,
                r.y + r.h / 2.0 + lift,
                size,
                Font::Black,
                if enabled { WHITE } else { hexc(0xB59AA7) },
                Align::Center,
            );
        }
        enabled && input.clicked(r) && view.contains(input.mouse)
    }

    /// Кнопка «действие + цена»: две строки с иконкой.
    #[allow(clippy::too_many_arguments)]
    fn price_button(
        &self,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        r: Rect,
        enabled: bool,
        style: Style,
        view: Rect,
        label: &str,
        price: &str,
        icon: Icon,
    ) -> bool {
        let clicked = self.button(gfx, input, r, enabled, style, view, "", 0.0);
        let hover = enabled && input.hover(r) && view.contains(input.mouse);
        let lift = if hover && input.down {
            1.0
        } else if hover {
            -1.0
        } else {
            0.0
        };
        let col = if enabled { WHITE } else { hexc(0xB59AA7) };
        gfx.text(
            label,
            r.x + r.w / 2.0,
            r.y + r.h * 0.32 + lift,
            14.0,
            Font::Black,
            col,
            Align::Center,
        );
        let pw = gfx.measure(price, 14.0, Font::Black);
        let key = format!("icon/{icon:?}/16");
        let spr = art.get(&key, 16.0, 16.0, || icons::icon_svg(icon));
        let start = r.x + r.w / 2.0 - (pw + 20.0) / 2.0;
        gfx.sprite(
            spr,
            start + 8.0,
            r.y + r.h * 0.7 + lift,
            1.0,
            if enabled {
                WHITE
            } else {
                Color::new(1.0, 1.0, 1.0, 0.6)
            },
        );
        gfx.text(
            price,
            start + 20.0,
            r.y + r.h * 0.7 + lift,
            14.0,
            Font::Black,
            if enabled {
                hexc(0xFFF3C4)
            } else {
                hexc(0xB59AA7)
            },
            Align::Left,
        );
        clicked
    }

    fn card(&self, x: f32, y: f32, w: f32, h: f32, accent: Option<Color>) {
        shadow(x, y, w, h, 18.0, 8.0, 0.22);
        rrect_grad(x, y, w, h, 18.0, WHITE, hexc(0xFFF6F9));
        rrect_line(
            x,
            y,
            w,
            h,
            18.0,
            if accent.is_some() { 2.5 } else { 1.5 },
            accent.unwrap_or(hexc(0xF6D3E1)),
        );
    }

    // ---------------------------------------------------------------- Отряд

    #[allow(clippy::too_many_arguments)]
    fn team_tab(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        x0: f32,
        top: f32,
        view: Rect,
    ) -> f32 {
        let (cx, cw) = (x0 + 14.0, PANEL_W - 30.0);
        let mut y = top;
        // выбор количества
        self.card(cx, y, cw, 50.0, None);
        gfx.text(
            "Покупать:",
            cx + 16.0,
            y + 25.0,
            15.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        let bw = 56.0;
        for (k, (amount, label)) in BUY_AMOUNTS.iter().enumerate() {
            let r = Rect::new(
                cx + cw - 12.0 - (5 - k) as f32 * (bw + 4.0),
                y + 9.0,
                bw,
                32.0,
            );
            let sel = game.buy_amount == *amount;
            if sel {
                rrect_grad(r.x, r.y, r.w, r.h, 16.0, hexc(0xF27BA5), hexc(0xC2185B));
                gfx.text(
                    label,
                    r.x + r.w / 2.0,
                    r.y + 16.0,
                    14.0,
                    Font::Black,
                    WHITE,
                    Align::Center,
                );
            } else if self.button(gfx, input, r, true, Style::Muted, view, label, 14.0) {
                game.buy_amount = *amount;
            }
        }
        y += 62.0;

        let mut shown_locked = false;
        for i in 0..HEROINES.len() {
            if game.hero_visible(i) {
                y += self.hero_card(game, gfx, art, input, i, cx, y, cw, view) + 10.0;
            } else if !shown_locked {
                shown_locked = true;
                self.card(cx, y, cw, 92.0, None);
                let key = format!("portrait/{}", HEROINES[i].id);
                let look = HEROINES[i].look;
                let spr = art.get(&key, 76.0, 76.0, || portrait_svg(&look));
                gfx.silhouette_ex(
                    spr,
                    cx + 14.0,
                    y + 8.0,
                    76.0,
                    76.0,
                    hexa(0x6E4A5E, 0.85),
                    false,
                );
                gfx.text(
                    "???",
                    cx + 52.0,
                    y + 46.0,
                    26.0,
                    Font::Black,
                    WHITE,
                    Align::Center,
                );
                gfx.text(
                    "Неизвестная героиня",
                    cx + 104.0,
                    y + 32.0,
                    17.0,
                    Font::Black,
                    hexc(MUTED),
                    Align::Left,
                );
                gfx.text_wrap(
                    &format!(
                        "Придёт, когда к отряду присоединится {}",
                        HEROINES[i - 1].name
                    ),
                    cx + 104.0,
                    y + 46.0,
                    cw - 120.0,
                    13.0,
                    Font::Bold,
                    hexc(MUTED),
                    17.0,
                );
                y += 102.0;
            }
        }
        y - top
    }

    #[allow(clippy::too_many_arguments)]
    fn hero_card(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        i: usize,
        x: f32,
        y: f32,
        w: f32,
        view: Rect,
    ) -> f32 {
        let h = 118.0;
        if y > view.y + view.h + 10.0 || y + h < view.y - 10.0 {
            return h;
        }
        let def = &HEROINES[i];
        let st = game.heroes[i];
        let can_up = game.can_upgrade(i) && game.gold >= game.upgrade_cost(i);
        self.card(x, y, w, h, if can_up { Some(hexc(PURPLE)) } else { None });
        let look = if def.clicker {
            sakura_look(&OUTFITS[game.outfit])
        } else {
            def.look
        };
        // портрет
        let pc = vec2(x + 56.0, y + 54.0);
        draw_circle(pc.x, pc.y, 43.0, hexc(look.obi));
        let key = if def.clicker {
            format!("portrait/sakura/{}", game.outfit)
        } else {
            format!("portrait/{}", def.id)
        };
        let spr = art.get(&key, 80.0, 80.0, || portrait_svg(&look));
        if st.level == 0 && !def.clicker {
            gfx.sprite_ex(
                spr,
                pc.x - 40.0,
                pc.y - 40.0,
                80.0,
                80.0,
                Color::new(1.0, 1.0, 1.0, 0.55),
                0.0,
                None,
                false,
            );
        } else {
            gfx.sprite_ex(
                spr,
                pc.x - 40.0,
                pc.y - 40.0,
                80.0,
                80.0,
                WHITE,
                0.0,
                None,
                false,
            );
        }
        // плашка уровня
        let lv = format!("ур. {}", st.level);
        let lw = gfx.measure(&lv, 13.0, Font::Black) + 16.0;
        rrect_grad(
            pc.x - lw / 2.0,
            y + 94.0,
            lw,
            20.0,
            10.0,
            hexc(0xF27BA5),
            hexc(0xC2185B),
        );
        gfx.text(
            &lv,
            pc.x,
            y + 104.0,
            13.0,
            Font::Black,
            WHITE,
            Align::Center,
        );

        let tx = x + 110.0;
        gfx.text(
            def.name,
            tx,
            y + 22.0,
            19.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        gfx.text(
            def.title,
            tx,
            y + 42.0,
            12.5,
            Font::Bold,
            hexc(MUTED),
            Align::Left,
        );
        let (icon, stat) = if def.clicker {
            (
                Icon::Sword,
                format!("Удар: {}", fmt::num(game.click_damage())),
            )
        } else if st.level == 0 {
            (
                Icon::Party,
                format!(
                    "{} / с за уровень",
                    fmt::num(def.base_dps * game.hero_mult(i) * game.dps_mult())
                ),
            )
        } else {
            (Icon::Party, format!("{} / с", fmt::num(game.hero_dps(i))))
        };
        let ik = format!("icon/{icon:?}/16");
        let ispr = art.get(&ik, 16.0, 16.0, || icons::icon_svg(icon));
        gfx.sprite(ispr, tx + 8.0, y + 64.0, 1.0, WHITE);
        let sw = gfx.text(
            &stat,
            tx + 20.0,
            y + 64.0,
            14.0,
            Font::Black,
            hexc(PINK_DARK),
            Align::Left,
        );
        if st.upgrades > 0 {
            gfx.text(
                &fmt::mult(game.hero_mult(i)),
                tx + 26.0 + sw,
                y + 64.0,
                12.0,
                Font::Black,
                hexc(PURPLE),
                Align::Left,
            );
        }
        // прогресс до свитка силы
        let bar_w = w - 110.0 - 150.0;
        if let Some(ms) = game.next_milestone(i) {
            let prev = if st.upgrades == 0 {
                0
            } else {
                crate::game::MILESTONES[st.upgrades as usize - 1]
            };
            let frac = (st.level.saturating_sub(prev)) as f32 / (ms - prev) as f32;
            progress_bar(
                tx,
                y + 84.0,
                bar_w,
                10.0,
                frac,
                hexa(0xC2185B, 0.12),
                hexc(0xB38BEB),
                hexc(0x7E4DBA),
            );
            let label = if game.can_upgrade(i) {
                "Свиток силы готов!".to_string()
            } else {
                format!("свиток ×2 на ур. {ms}")
            };
            gfx.text(
                &label,
                tx,
                y + 104.0,
                11.5,
                Font::Bold,
                if game.can_upgrade(i) {
                    hexc(PURPLE)
                } else {
                    hexc(MUTED)
                },
                Align::Left,
            );
        } else {
            gfx.text(
                "Все свитки изучены",
                tx,
                y + 92.0,
                12.0,
                Font::Bold,
                hexc(PURPLE),
                Align::Left,
            );
        }

        // кнопки
        let n = game.amount_for(i);
        let cost = game.hero_cost(i, n);
        let verb = if def.clicker {
            "Тренировать"
        } else if st.level == 0 {
            "Нанять"
        } else {
            "Улучшить"
        };
        let label = if n > 1 {
            format!("{verb} +{n}")
        } else {
            verb.to_string()
        };
        let bx = x + w - 146.0;
        let has_scroll = game.can_upgrade(i);
        let buy_r = Rect::new(bx, y + 12.0, 134.0, if has_scroll { 48.0 } else { 94.0 });
        if self.price_button(
            gfx,
            art,
            input,
            buy_r,
            game.gold >= cost,
            Style::Pink,
            view,
            &label,
            &fmt::num(cost),
            Icon::Coin,
        ) {
            game.buy(i);
        }
        if has_scroll {
            let up_r = Rect::new(bx, y + 66.0, 134.0, 40.0);
            let ucost = game.upgrade_cost(i);
            let glow = 0.5 + 0.5 * (self.time * 4.0).sin();
            if game.gold >= ucost {
                gfx.glow(
                    up_r.x + up_r.w / 2.0,
                    up_r.y + up_r.h / 2.0,
                    80.0,
                    Color::new(0.6, 0.35, 1.0, 0.18 * glow),
                );
            }
            if self.price_button(
                gfx,
                art,
                input,
                up_r,
                game.gold >= ucost,
                Style::Purple,
                view,
                "Свиток силы ×2",
                &fmt::num(ucost),
                Icon::Coin,
            ) {
                game.buy_upgrade(i);
            }
        }
        h
    }

    // ---------------------------------------------------------------- Гардероб

    #[allow(clippy::too_many_arguments)]
    fn wardrobe_tab(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        x0: f32,
        top: f32,
        view: Rect,
    ) -> f32 {
        let (cx, cw) = (x0 + 14.0, PANEL_W - 30.0);
        let mut y = top;
        self.card(cx, y, cw, 76.0, None);
        let owned = game.outfits_owned.iter().filter(|o| **o).count();
        gfx.text(
            &format!("Гардероб: {} из {}", owned, OUTFITS.len()),
            cx + 16.0,
            y + 22.0,
            17.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        gfx.text(
            &format!("бонус коллекции {}", fmt::mult(game.collection_mult())),
            cx + cw - 16.0,
            y + 22.0,
            13.0,
            Font::Black,
            hexc(PINK_DARK),
            Align::Right,
        );
        gfx.text_wrap("Наряды выпадают с боссов. Надетый наряд даёт свой бонус, а каждый собранный — ещё +5% ко всему урону.", cx + 16.0, y + 36.0, cw - 32.0, 12.5, Font::Bold, hexc(MUTED), 16.0);
        y += 88.0;
        let col_w = (cw - 10.0) / 2.0;
        let card_h = 300.0;
        for (i, o) in OUTFITS.iter().enumerate() {
            let x = cx + (i % 2) as f32 * (col_w + 10.0);
            let yy = y + (i / 2) as f32 * (card_h + 10.0);
            if yy > view.y + view.h + 10.0 || yy + card_h < view.y - 10.0 {
                continue;
            }
            let worn = game.outfit == i;
            let has = game.outfits_owned[i];
            self.card(
                x,
                yy,
                col_w,
                card_h,
                if worn { Some(hexc(PINK)) } else { None },
            );
            // подиум
            gfx.glow_ellipse(x + col_w / 2.0, yy + 196.0, 70.0, 16.0, hexa(0xF8BBD0, 0.5));
            let look = sakura_look(o);
            let (sw, sh) = (chibi::W * 0.6, chibi::H * 0.6);
            let key = format!("outfit/{}", o.id);
            let spr = art.get(&key, sw, sh, || chibi::heroine_svg(&look, false, false));
            let bob = if worn {
                (self.time * 2.0).sin() * 2.0
            } else {
                0.0
            };
            if has {
                gfx.sprite_ex(
                    spr,
                    x + col_w / 2.0 - sw / 2.0,
                    yy + 4.0 + bob,
                    sw,
                    sh,
                    WHITE,
                    0.0,
                    None,
                    false,
                );
            } else {
                gfx.silhouette_ex(
                    spr,
                    x + col_w / 2.0 - sw / 2.0,
                    yy + 4.0,
                    sw,
                    sh,
                    hexa(0x6E4A5E, 0.75),
                    false,
                );
                let lock = art.get("icon/lock/34", 34.0, 34.0, || icons::icon_svg(Icon::Lock));
                gfx.sprite(lock, x + col_w / 2.0, yy + 110.0, 1.0, WHITE);
            }
            gfx.text_wrap(
                o.name,
                x + 12.0,
                yy + 202.0,
                col_w - 24.0,
                14.5,
                Font::Black,
                hexc(TEXT),
                17.0,
            );
            gfx.text(
                o.bonus,
                x + 12.0,
                yy + 248.0,
                12.0,
                Font::Black,
                hexc(PINK_DARK),
                Align::Left,
            );
            let r = Rect::new(x + 12.0, yy + card_h - 40.0, col_w - 24.0, 30.0);
            if !has {
                rrect(r.x, r.y, r.w, r.h, 15.0, hexc(0xF1E3EA));
                gfx.text(
                    &format!("Босс этапа {}", o.unlock_stage),
                    r.x + r.w / 2.0,
                    r.y + 15.0,
                    13.0,
                    Font::Black,
                    hexc(MUTED),
                    Align::Center,
                );
            } else if worn {
                rrect(r.x, r.y, r.w, r.h, 15.0, hexa(0xE0598B, 0.15));
                gfx.text(
                    "Надето",
                    r.x + r.w / 2.0,
                    r.y + 15.0,
                    14.0,
                    Font::Black,
                    hexc(PINK_DARK),
                    Align::Center,
                );
            } else if self.button(gfx, input, r, true, Style::Pink, view, "Надеть", 14.0) {
                game.wear(i);
            }
        }
        y += OUTFITS.len().div_ceil(2) as f32 * (card_h + 10.0);
        y - top
    }

    // ---------------------------------------------------------------- Храм

    #[allow(clippy::too_many_arguments)]
    fn temple_tab(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        x0: f32,
        top: f32,
        view: Rect,
        scene: &mut Scene,
    ) -> f32 {
        let (cx, cw) = (x0 + 14.0, PANEL_W - 30.0);
        let mut y = top;
        let h = 236.0;
        self.card(cx, y, cw, h, Some(hexc(0xD1B3F0)));
        vgrad(
            cx + 2.0,
            y + 2.0,
            cw - 4.0,
            70.0,
            hexa(0xEDE1FA, 0.9),
            hexa(0xEDE1FA, 0.0),
        );
        let torii = art.get("icon/temple/64", 64.0, 64.0, || {
            icons::icon_svg(Icon::TabTemple)
        });
        gfx.sprite(torii, cx + cw - 50.0, y + 44.0, 1.0, WHITE);
        gfx.text(
            "Перерождение",
            cx + 16.0,
            y + 26.0,
            21.0,
            Font::Black,
            hexc(0x5E35B1),
            Align::Left,
        );
        gfx.text_wrap(
            &format!("Начните путь заново с этапа 1 и получите души сакуры: каждая навсегда даёт +10% ко всему урону. Наряды, талисманы и рекорды сохранятся. Доступно с этапа {}.", REBIRTH_MIN_STAGE),
            cx + 16.0, y + 42.0, cw - 110.0, 12.5, Font::Bold, hexc(MUTED), 16.5,
        );
        let soul = art.get("icon/soul/26", 26.0, 26.0, || icons::icon_svg(Icon::Soul));
        gfx.sprite(soul, cx + 30.0, y + 132.0, 1.0, WHITE);
        gfx.text(
            &format!("Души: {}", game.souls),
            cx + 48.0,
            y + 132.0,
            18.0,
            Font::Black,
            hexc(0x5E35B1),
            Align::Left,
        );
        gfx.text(
            &format!(
                "свободно {} • урон {}",
                game.available_souls(),
                fmt::mult(game.soul_mult())
            ),
            cx + cw - 16.0,
            y + 132.0,
            13.0,
            Font::Black,
            hexc(PURPLE),
            Align::Right,
        );
        let pending = game.pending_souls();
        let info = if pending > 0 {
            format!(
                "Сейчас вы получите +{} душ (этап {})",
                pending, game.max_stage
            )
        } else {
            format!(
                "Дойдите до этапа {} (сейчас {})",
                REBIRTH_MIN_STAGE, game.max_stage
            )
        };
        gfx.text(
            &info,
            cx + 16.0,
            y + 162.0,
            13.5,
            Font::Bold,
            hexc(TEXT),
            Align::Left,
        );
        let r = Rect::new(cx + 16.0, y + 180.0, cw - 32.0, 42.0);
        if pending > 0 {
            let g = 0.5 + 0.5 * (self.time * 3.0).sin();
            gfx.glow(
                r.x + r.w / 2.0,
                r.y + r.h / 2.0,
                160.0,
                Color::new(0.6, 0.35, 1.0, 0.15 * g),
            );
        }
        let label = if pending > 0 {
            format!("Переродиться (+{} душ)", pending)
        } else {
            "Пока недоступно".to_string()
        };
        if self.button(
            gfx,
            input,
            r,
            pending > 0,
            Style::Purple,
            view,
            &label,
            16.0,
        ) {
            self.modal = Some(Modal::Rebirth);
        }
        y += h + 16.0;

        gfx.text(
            "Талисманы",
            cx + 6.0,
            y + 12.0,
            18.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        gfx.text(
            "покупаются за свободные души",
            cx + cw - 6.0,
            y + 12.0,
            12.0,
            Font::Bold,
            hexc(MUTED),
            Align::Right,
        );
        y += 30.0;
        let tal_icons = [
            Icon::TalBlade,
            Icon::TalSpirit,
            Icon::TalWealth,
            Icon::TalLuck,
            Icon::TalHourglass,
        ];
        for (i, t) in TALISMANS.iter().enumerate() {
            let ch = 84.0;
            self.card(cx, y, cw, ch, None);
            draw_circle(cx + 42.0, y + ch / 2.0, 30.0, hexc(0xF3E8FD));
            let key = format!("icon/tal{i}/44");
            let spr = art.get(&key, 44.0, 44.0, || icons::icon_svg(tal_icons[i]));
            gfx.sprite(spr, cx + 42.0, y + ch / 2.0, 1.0, WHITE);
            gfx.text(
                t.name,
                cx + 84.0,
                y + 24.0,
                16.0,
                Font::Black,
                hexc(TEXT),
                Align::Left,
            );
            gfx.text(
                t.desc,
                cx + 84.0,
                y + 46.0,
                12.0,
                Font::Bold,
                hexc(MUTED),
                Align::Left,
            );
            let lvl = game.talismans[i];
            // пипсы уровня
            let pips = t.max_level.min(10);
            let filled = if t.max_level > 10 {
                (lvl * 10).div_ceil(t.max_level)
            } else {
                lvl
            };
            for k in 0..pips {
                draw_circle(
                    cx + 88.0 + k as f32 * 11.0,
                    y + 66.0,
                    4.0,
                    if k < filled {
                        hexc(PURPLE)
                    } else {
                        hexc(0xE8DDF2)
                    },
                );
            }
            gfx.text(
                &format!("{}/{}", lvl, t.max_level),
                cx + 88.0 + pips as f32 * 11.0 + 4.0,
                y + 66.0,
                11.5,
                Font::Black,
                hexc(PURPLE),
                Align::Left,
            );
            let r = Rect::new(cx + cw - 116.0, y + 18.0, 104.0, 48.0);
            if lvl >= t.max_level {
                rrect(r.x, r.y, r.w, r.h, 16.0, hexc(0xF3E8FD));
                gfx.text(
                    "Максимум",
                    r.x + r.w / 2.0,
                    r.y + r.h / 2.0,
                    14.0,
                    Font::Black,
                    hexc(PURPLE),
                    Align::Center,
                );
            } else {
                let cost = game.talisman_cost(i);
                if self.price_button(
                    gfx,
                    art,
                    input,
                    r,
                    game.available_souls() >= cost,
                    Style::Purple,
                    view,
                    "Улучшить",
                    &cost.to_string(),
                    Icon::Soul,
                ) {
                    game.buy_talisman(i);
                    scene.message(format!("{}: уровень {}", t.name, game.talismans[i]));
                }
            }
            y += ch + 10.0;
        }
        y - top
    }

    // ---------------------------------------------------------------- Летопись

    #[allow(clippy::too_many_arguments)]
    fn chronicle_tab(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        _art: &mut Art,
        input: &Input,
        x0: f32,
        top: f32,
        view: Rect,
    ) -> f32 {
        let (cx, cw) = (x0 + 14.0, PANEL_W - 30.0);
        let mut y = top;
        let rows: Vec<(&str, String)> = vec![
            ("Золото", format!("{} рё", fmt::num(game.gold))),
            (
                "Добыто за всё время",
                format!("{} рё", fmt::num(game.total_gold)),
            ),
            ("Удар Сакуры", fmt::num(game.click_damage())),
            ("Урон отряда", format!("{} / с", fmt::num(game.dps()))),
            (
                "Шанс крита",
                format!("{}% (×5)", (game.crit_chance() * 100.0).round()),
            ),
            ("Время на босса", format!("{} с", game.boss_time().round())),
            ("Текущий этап", game.stage.to_string()),
            ("Рекорд этапа", game.best_stage.to_string()),
            ("Ударов", fmt::int(game.total_clicks)),
            ("Ёкаев повержено", fmt::int(game.total_kills)),
            ("Боссов повержено", fmt::int(game.bosses_killed)),
            (
                "Героинь в отряде",
                format!("{} из {}", game.party_size(), HEROINES.len() - 1),
            ),
            ("Перерождений", game.rebirths.to_string()),
            ("Души сакуры", game.souls.to_string()),
            ("Время в игре", fmt::time(game.play_time)),
        ];
        let h = 30.0 + rows.len() as f32 * 26.0;
        self.card(cx, y, cw, h, None);
        gfx.text(
            "Летопись",
            cx + 16.0,
            y + 22.0,
            19.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        for (k, (label, value)) in rows.iter().enumerate() {
            let ry = y + 50.0 + k as f32 * 26.0;
            if k % 2 == 0 {
                rrect(
                    cx + 8.0,
                    ry - 12.0,
                    cw - 16.0,
                    24.0,
                    8.0,
                    hexa(0xFCE4EC, 0.6),
                );
            }
            gfx.text(
                label,
                cx + 18.0,
                ry,
                13.5,
                Font::Bold,
                hexc(MUTED),
                Align::Left,
            );
            gfx.text(
                value,
                cx + cw - 18.0,
                ry,
                13.5,
                Font::Black,
                hexc(TEXT),
                Align::Right,
            );
        }
        y += h + 18.0;

        let help = [
            ("Клик по полю боя", "удар Сакуры"),
            ("1 – 4", "навыки"),
            ("Стрелки влево / вправо", "смена этапа"),
            ("A", "вперёд: вкл / выкл"),
            ("Колесо мыши", "прокрутка панели"),
        ];
        let hh = 40.0 + help.len() as f32 * 24.0 + 40.0;
        self.card(cx, y, cw, hh, None);
        gfx.text(
            "Управление",
            cx + 16.0,
            y + 22.0,
            17.0,
            Font::Black,
            hexc(TEXT),
            Align::Left,
        );
        for (k, (key, what)) in help.iter().enumerate() {
            let ry = y + 50.0 + k as f32 * 24.0;
            let kw = gfx.measure(key, 12.5, Font::Black) + 16.0;
            rrect(cx + 16.0, ry - 10.0, kw, 20.0, 6.0, hexc(0xF3E3EA));
            gfx.text(
                key,
                cx + 24.0,
                ry,
                12.5,
                Font::Black,
                hexc(PINK_DARK),
                Align::Left,
            );
            gfx.text(
                what,
                cx + 24.0 + kw,
                ry,
                13.0,
                Font::Bold,
                hexc(TEXT),
                Align::Left,
            );
        }
        gfx.text_wrap(
            "Игра сохраняется сама, а отряд сражается, даже когда игра закрыта (до 8 часов).",
            cx + 16.0,
            y + 50.0 + help.len() as f32 * 24.0 - 4.0,
            cw - 32.0,
            12.5,
            Font::Bold,
            hexc(MUTED),
            16.0,
        );
        y += hh + 18.0;

        let r = Rect::new(cx + cw / 2.0 - 110.0, y, 220.0, 38.0);
        if self.button(
            gfx,
            input,
            r,
            true,
            Style::Muted,
            view,
            "Начать игру с нуля",
            14.0,
        ) {
            self.modal = Some(Modal::Reset);
        }
        y += 54.0;
        y - top
    }

    // ---------------------------------------------------------------- модальные окна

    /// Рисует модальное окно поверх всего. Возвращает true, если оно открыто.
    pub fn draw_modal(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &mut Input,
        scene: &mut Scene,
    ) -> bool {
        let Some(modal) = &self.modal else {
            return false;
        };
        let (cw, ch) = (gfx.canvas_w, gfx.canvas_h);
        draw_rectangle(0.0, 0.0, cw, ch, Color::new(0.18, 0.06, 0.13, 0.55));
        let (w, h) = (540.0, 270.0);
        let (x, y) = ((cw - w) / 2.0, (ch - h) / 2.0);
        shadow(x, y, w, h, 26.0, 18.0, 0.5);
        rrect_grad(x, y, w, h, 26.0, WHITE, hexc(0xFFF0F6));
        rrect_line(
            x + 6.0,
            y + 6.0,
            w - 12.0,
            h - 12.0,
            22.0,
            2.0,
            hexc(0xF6C9DA),
        );
        let all = Rect::new(0.0, 0.0, cw, ch);
        let (title, text, icon, ok_label, cancel): (String, String, Icon, &str, bool) = match modal {
            Modal::Rebirth => (
                "Переродиться?".into(),
                format!("Вы получите {} душ сакуры. Золото, этапы и уровни героинь сбросятся. Наряды, талисманы и рекорды сохранятся.", game.pending_souls()),
                Icon::Soul,
                "Переродиться",
                true,
            ),
            Modal::Reset => ("Начать с нуля?".into(), "Весь прогресс будет удалён: золото, героини, наряды, души и талисманы. Это нельзя отменить.".into(), Icon::Lock, "Удалить всё", true),
            Modal::Offline { gold, secs } => (
                "С возвращением!".into(),
                format!("Пока вас не было ({}), отряд добыл {} рё.", fmt::time(*secs), fmt::num(*gold)),
                Icon::Coin,
                "Забрать",
                false,
            ),
            Modal::Welcome => (
                "Сакура и ёкаи".into(),
                "Кликайте по ёкаям, чтобы Сакура била катаной. Нанимайте героинь справа — они сражаются сами. Каждый пятый этап — босс!".into(),
                Icon::Gift,
                "В бой!",
                false,
            ),
        };
        let key = format!("icon/{icon:?}/56");
        let spr = art.get(&key, 56.0, 56.0, || icons::icon_svg(icon));
        gfx.sprite(spr, x + 56.0, y + 62.0, 1.0, WHITE);
        gfx.text(
            &title,
            x + 100.0,
            y + 60.0,
            28.0,
            Font::Black,
            hexc(0xC2185B),
            Align::Left,
        );
        gfx.text_wrap(
            &text,
            x + 40.0,
            y + 100.0,
            w - 80.0,
            16.0,
            Font::Bold,
            hexc(TEXT),
            22.0,
        );
        let ok = Rect::new(
            if cancel {
                x + w - 40.0 - 200.0
            } else {
                x + w / 2.0 - 100.0
            },
            y + h - 70.0,
            200.0,
            46.0,
        );
        let style = if matches!(modal, Modal::Rebirth) {
            Style::Purple
        } else {
            Style::Pink
        };
        input.blocked = false;
        let ok_clicked = self.button(gfx, input, ok, true, style, all, ok_label, 17.0);
        let mut close = false;
        if cancel {
            let c = Rect::new(x + 40.0, y + h - 70.0, 160.0, 46.0);
            if self.button(gfx, input, c, true, Style::Muted, all, "Отмена", 17.0) {
                close = true;
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            close = true;
        }
        if ok_clicked || is_key_pressed(KeyCode::Enter) {
            match modal {
                Modal::Rebirth => {
                    let got = game.rebirth();
                    if got > 0 {
                        scene.message(format!("Перерождение! +{} душ сакуры", got));
                    }
                }
                Modal::Reset => {
                    game.reset_all();
                    scene.message("Новая история начинается".into());
                }
                Modal::Offline { .. } | Modal::Welcome => {}
            }
            close = true;
        }
        if close {
            self.modal = None;
        }
        input.blocked = true;
        true
    }
}

/// Портрет в круге (фон + героиня).
pub fn portrait_svg(look: &Look) -> String {
    let (px, py, size) = chibi::PORTRAIT;
    let inner = chibi::heroine(look, false, false).finish_view(px, py, size, size);
    let c = size / 2.0;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><defs><clipPath id="pc"><circle cx="{c}" cy="{c}" r="{r}"/></clipPath><radialGradient id="pg" cx="0.5" cy="0.35" r="0.7"><stop offset="0" stop-color="#ffffff"/><stop offset="1" stop-color="{bg}"/></radialGradient></defs><circle cx="{c}" cy="{c}" r="{r}" fill="url(#pg)"/><g clip-path="url(#pc)">{inner}</g></svg>"##,
        r = c - 2.0,
        bg = crate::art::hex(crate::art::tint(look.obi, 0.55)),
    )
}

/// Фон панели: мягкий градиент и узор сэйгайха (волны).
fn panel_bg(w: f32, h: f32) -> String {
    let mut s = Svg::new(w, h);
    let g = s.vgrad(0.0, h, 0xFFF7FA, 0xFCE4EE);
    s.raw(&format!(r#"<rect width="{w}" height="{h}" fill="{g}"/>"#));
    let mut waves = String::new();
    let r = 22.0;
    let mut row = 0;
    let mut y = 0.0;
    while y < h + r {
        let offset = if row % 2 == 0 { 0.0 } else { r };
        let mut x = -r + offset;
        while x < w + r {
            for k in [1.0, 0.72, 0.44] {
                use std::fmt::Write;
                let _ = write!(
                    waves,
                    r##"<circle cx="{x:.1}" cy="{y:.1}" r="{:.1}" fill="{}" stroke="#f3c6d8" stroke-width="1.4"/>"##,
                    r * k,
                    if k == 1.0 { "#fdeef4" } else { "none" }
                );
            }
            x += r * 2.0;
        }
        y += r * 0.55;
        row += 1;
    }
    s.raw(&format!(r#"<g opacity="0.55">{waves}</g>"#));
    s.finish()
}
