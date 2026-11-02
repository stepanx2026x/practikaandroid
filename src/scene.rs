//! Поле боя: пейзаж, Сакура и отряд, ёкай, эффекты и игровой интерфейс поверх сцены.

use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::art::icons::{self, Icon, Particle};
use crate::art::scenery::{self, Light};
use crate::art::{chibi, yokai, Art};
use crate::data::*;
use crate::fmt;
use crate::game::{Event, Game, MONSTERS_PER_STAGE};
use crate::gfx::*;
use crate::gfx::{Align, Font};
use crate::input::Input;

const SAKURA_SCALE: f32 = 0.9;
const MONSTER_SIZE: f32 = 270.0;
const BOSS_SIZE: f32 = 340.0;

struct Part {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max: f32,
    size: f32,
    rot: f32,
    spin: f32,
    kind: PartKind,
    color: Color,
}

#[derive(Clone, Copy, PartialEq)]
enum PartKind {
    Petal,
    Spark,
    Smoke,
    Coin,
    Sparkle,
}

struct FloatText {
    gold: bool,
    pos: Vec2,
    life: f32,
    max: f32,
    text: String,
    color: Color,
    size: f32,
    crit: bool,
}

struct Coin {
    from: Vec2,
    ctrl: Vec2,
    t: f32,
    delay: f32,
}

struct Drift {
    pos: Vec2,
    vel: Vec2,
    z: f32,
    phase: f32,
    rot: f32,
    spin: f32,
}

struct Dying {
    def: usize,
    boss: bool,
    t: f32,
}

struct Banner {
    title: String,
    sub: String,
    t: f32,
    dur: f32,
    color: u32,
}

pub struct Scene {
    time: f32,
    attack: f32,
    hit_flash: f32,
    shake: f32,
    spawn_t: f32,
    last_serial: u64,
    dying: Option<Dying>,
    blink: f32,
    next_blink: f32,
    parts: Vec<Part>,
    texts: Vec<FloatText>,
    coins: Vec<Coin>,
    weather: Vec<Drift>,
    last_def: (usize, bool),
    controls: Vec<Rect>,
    clouds: Vec<(f32, f32, f32, f32)>,
    toasts: VecDeque<String>,
    toast: Option<(String, f32)>,
    banner: Option<Banner>,
    gold_shown: f64,
    gold_bump: f32,
    lights: Vec<Light>,
    lights_key: String,
    zone: usize,
    prev_zone: Option<(usize, f32)>,
    strike: f32,
    red_flash: f32,
    last_zone_stage: u32,
    /// Куда летят монеты (центр иконки золота).
    coin_target: Vec2,
    pub hint_shown: bool,
}

pub struct Layout {
    pub w: f32,
    pub h: f32,
    pub ground: f32,
    pub sakura_x: f32,
    pub monster_x: f32,
}

impl Layout {
    pub fn new(w: f32, h: f32) -> Self {
        Layout {
            w,
            h,
            ground: h * 0.865,
            sakura_x: w * 0.42,
            monster_x: w * 0.74,
        }
    }

    fn monster_center(&self, boss: bool) -> Vec2 {
        let size = if boss { BOSS_SIZE } else { MONSTER_SIZE };
        vec2(self.monster_x, self.ground - size * 0.47)
    }
}

impl Scene {
    pub fn new(game: &Game) -> Self {
        let mut s = Scene {
            time: 0.0,
            attack: 0.0,
            hit_flash: 0.0,
            shake: 0.0,
            spawn_t: 1.0,
            last_serial: game.monster.serial,
            dying: None,
            blink: 0.0,
            next_blink: 2.5,
            parts: Vec::new(),
            texts: Vec::new(),
            coins: Vec::new(),
            weather: Vec::new(),
            last_def: (game.monster.def, game.monster.boss),
            controls: Vec::new(),
            clouds: Vec::new(),
            toasts: VecDeque::new(),
            toast: None,
            banner: None,
            gold_shown: game.gold,
            gold_bump: 0.0,
            lights: Vec::new(),
            lights_key: String::new(),
            zone: zone_index(game.stage),
            prev_zone: None,
            strike: 0.0,
            red_flash: 0.0,
            last_zone_stage: game.stage,
            coin_target: vec2(40.0, 44.0),
            hint_shown: false,
        };
        for i in 0..4 {
            s.clouds.push((
                rand::gen_range(0.0, 1.0),
                0.06 + i as f32 * 0.07,
                rand::gen_range(0.6, 1.1),
                rand::gen_range(6.0, 14.0),
            ));
        }
        s
    }

    pub fn message(&mut self, text: String) {
        if self.toasts.len() < 5
            && self.toast.as_ref().map(|t| t.0 != text).unwrap_or(true)
            && !self.toasts.contains(&text)
        {
            self.toasts.push_back(text);
        }
    }

    // ------------------------------------------------------------------ события

    pub fn handle(&mut self, events: Vec<Event>, game: &Game, lay: &Layout) {
        for e in events {
            match e {
                Event::Hit {
                    damage,
                    crit,
                    click,
                } => self.on_hit(damage, crit, click, game, lay),
                Event::Kill { gold, boss } => self.on_kill(gold, boss, game, lay),
                Event::Message(m) => self.message(m),
                Event::StageChanged => {}
                Event::BossFailed => {
                    self.red_flash = 1.0;
                    self.shake = 0.8;
                }
                Event::OutfitUnlocked(i) => {
                    self.banner = Some(Banner {
                        title: "Новый наряд!".into(),
                        sub: OUTFITS[i].name.into(),
                        t: 0.0,
                        dur: 2.6,
                        color: 0xE0598B,
                    });
                    for _ in 0..40 {
                        self.burst(
                            vec2(lay.sakura_x, lay.ground - 150.0),
                            PartKind::Sparkle,
                            1.0,
                        );
                    }
                }
                Event::SkillUsed(i) => {
                    if i == SKILL_STRIKE {
                        self.strike = 1.0;
                        self.shake = 1.2;
                    }
                    let c = lay.monster_center(game.monster.boss);
                    for _ in 0..30 {
                        self.burst(
                            if i == SKILL_STRIKE {
                                c
                            } else {
                                vec2(lay.sakura_x, lay.ground - 140.0)
                            },
                            if i == SKILL_FAN {
                                PartKind::Coin
                            } else {
                                PartKind::Sparkle
                            },
                            1.2,
                        );
                    }
                }
            }
        }
    }

    fn burst(&mut self, at: Vec2, kind: PartKind, power: f32) {
        let a = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(120.0, 380.0) * power;
        let life = rand::gen_range(0.6, 1.1);
        self.parts.push(Part {
            pos: at,
            vel: vec2(a.cos() * sp, a.sin() * sp - 160.0),
            life,
            max: life,
            size: match kind {
                PartKind::Coin => rand::gen_range(14.0, 20.0),
                PartKind::Sparkle => rand::gen_range(14.0, 26.0),
                PartKind::Smoke => rand::gen_range(36.0, 60.0),
                _ => rand::gen_range(8.0, 14.0),
            },
            rot: rand::gen_range(0.0, 6.0),
            spin: rand::gen_range(-8.0, 8.0),
            kind,
            color: WHITE,
        });
    }

    fn on_hit(&mut self, damage: f64, crit: bool, click: bool, game: &Game, lay: &Layout) {
        if click {
            self.attack = 1.0;
        }
        self.hit_flash = 1.0;
        self.shake = self.shake.max(if crit { 0.55 } else { 0.18 });
        let c = lay.monster_center(game.monster.boss);
        let jitter = vec2(rand::gen_range(-80.0, 80.0), rand::gen_range(-50.0, 30.0));
        self.texts.push(FloatText {
            gold: false,
            pos: c + jitter + vec2(0.0, -60.0),
            life: if crit { 1.3 } else { 0.9 },
            max: if crit { 1.3 } else { 0.9 },
            text: fmt::num(damage),
            color: if crit { hexc(0xFFD54F) } else { WHITE },
            size: if crit { 40.0 } else { 28.0 },
            crit,
        });
        if self.texts.len() > 22 {
            self.texts.remove(0);
        }
        let hit = c + jitter * 0.5;
        for _ in 0..(if crit { 10 } else { 4 }) {
            self.burst(hit, PartKind::Spark, 0.8);
        }
        for _ in 0..(if crit { 6 } else { 2 }) {
            self.burst(hit, PartKind::Petal, 0.6);
        }
        if crit {
            for _ in 0..3 {
                self.burst(hit, PartKind::Sparkle, 0.7);
            }
        }
        // след удара (не больше нескольких одновременно)
        let slashes = self
            .parts
            .iter()
            .filter(|p| p.kind == PartKind::Smoke && p.color.a < 0.0)
            .count();
        if slashes >= 4 {
            return;
        }
        self.parts.push(Part {
            pos: hit,
            vel: Vec2::ZERO,
            life: 0.28,
            max: 0.28,
            size: if crit { 1.4 } else { 1.0 },
            rot: rand::gen_range(-0.9, 0.5),
            spin: 0.0,
            kind: PartKind::Smoke,
            color: Color::new(1.0, 1.0, 1.0, -1.0), // маркер: это «разрез»
        });
    }

    fn on_kill(&mut self, gold: f64, boss: bool, game: &Game, lay: &Layout) {
        let c = lay.monster_center(boss);
        for _ in 0..(if boss { 16 } else { 7 }) {
            let from = c + vec2(rand::gen_range(-40.0, 40.0), rand::gen_range(-40.0, 40.0));
            self.coins.push(Coin {
                from,
                ctrl: from
                    + vec2(
                        rand::gen_range(-160.0, 60.0),
                        rand::gen_range(-260.0, -120.0),
                    ),
                t: 0.0,
                delay: rand::gen_range(0.0, 0.25),
            });
        }
        for _ in 0..12 {
            self.burst(c, PartKind::Petal, 1.0);
        }
        for _ in 0..8 {
            let mut p = Part {
                pos: c + vec2(rand::gen_range(-70.0, 70.0), rand::gen_range(-60.0, 60.0)),
                vel: vec2(rand::gen_range(-30.0, 30.0), rand::gen_range(-60.0, -20.0)),
                life: 0.7,
                max: 0.7,
                size: rand::gen_range(40.0, 70.0),
                rot: 0.0,
                spin: 0.0,
                kind: PartKind::Smoke,
                color: WHITE,
            };
            p.vel.y -= 20.0;
            self.parts.push(p);
        }
        let gold_texts = self.texts.iter().filter(|t| t.gold).count();
        if gold_texts < 3 {
            self.texts.push(FloatText {
                gold: true,
                pos: c + vec2(
                    rand::gen_range(-30.0, 30.0),
                    -150.0 - gold_texts as f32 * 34.0,
                ),
                life: 1.4,
                max: 1.4,
                text: format!("+{} рё", fmt::num(gold)),
                color: hexc(0xFFC93C),
                size: if boss { 38.0 } else { 28.0 },
                crit: false,
            });
        }
        if boss {
            self.banner = Some(Banner {
                title: "Босс повержен!".into(),
                sub: format!("+{} рё", fmt::num(gold)),
                t: 0.0,
                dur: 2.2,
                color: 0xE0A520,
            });
            self.shake = 0.9;
        }
        let _ = game;
    }

    // ------------------------------------------------------------------ обновление

    pub fn update(&mut self, dt: f32, game: &Game, lay: &Layout) {
        self.time += dt;
        self.attack = (self.attack - dt * 5.0).max(0.0);
        self.hit_flash = (self.hit_flash - dt * 7.0).max(0.0);
        self.shake = (self.shake - dt * 3.5).max(0.0);
        self.strike = (self.strike - dt * 1.6).max(0.0);
        self.red_flash = (self.red_flash - dt * 1.5).max(0.0);
        self.gold_bump = (self.gold_bump - dt * 4.0).max(0.0);

        // смена ёкая
        if game.monster.serial != self.last_serial {
            if self.spawn_t > 0.3 {
                let prev = self.last_def;
                self.dying = Some(Dying {
                    def: prev.0,
                    boss: prev.1,
                    t: 0.0,
                });
            }
            self.last_serial = game.monster.serial;
            self.spawn_t = -0.18;
            if game.monster.boss {
                self.banner = Some(Banner {
                    title: "Босс!".into(),
                    sub: MONSTERS[game.monster.def].name.into(),
                    t: 0.0,
                    dur: 1.8,
                    color: 0xD32F2F,
                });
            }
        }
        self.last_def = (game.monster.def, game.monster.boss);
        self.spawn_t = (self.spawn_t + dt * 3.2).min(1.0);
        if let Some(d) = &mut self.dying {
            d.t += dt * 3.0;
            if d.t >= 1.0 {
                self.dying = None;
            }
        }

        // смена локации — плавный переход
        let z = zone_index(game.stage);
        if z != self.zone {
            self.prev_zone = Some((self.zone, 1.0));
            self.zone = z;
            if game.stage != self.last_zone_stage {
                self.banner = Some(Banner {
                    title: zone_name(game.stage),
                    sub: format!(
                        "Этапы {}–{}",
                        (game.stage - 1) / 10 * 10 + 1,
                        (game.stage - 1) / 10 * 10 + 10
                    ),
                    t: 0.0,
                    dur: 2.6,
                    color: 0x7E57C2,
                });
            }
        }
        self.last_zone_stage = game.stage;
        if let Some((_, a)) = &mut self.prev_zone {
            *a -= dt * 1.2;
            if *a <= 0.0 {
                self.prev_zone = None;
            }
        }

        // моргание
        self.next_blink -= dt;
        if self.next_blink <= 0.0 {
            self.blink = 0.14;
            self.next_blink = rand::gen_range(2.5, 5.5);
        }
        self.blink = (self.blink - dt).max(0.0);

        for p in self.parts.iter_mut() {
            p.life -= dt;
            match p.kind {
                PartKind::Smoke => {
                    p.pos += p.vel * dt;
                    p.size += 40.0 * dt;
                }
                _ => {
                    p.vel.y += 700.0 * dt;
                    p.vel *= 1.0 - dt * 0.8;
                    p.pos += p.vel * dt;
                    p.rot += p.spin * dt;
                }
            }
        }
        self.parts.retain(|p| p.life > 0.0);
        if self.parts.len() > 500 {
            let n = self.parts.len() - 500;
            self.parts.drain(0..n);
        }
        for t in self.texts.iter_mut() {
            t.life -= dt;
            t.pos.y -= (if t.crit { 70.0 } else { 90.0 }) * dt;
        }
        self.texts.retain(|t| t.life > 0.0);

        let target = self.coin_target;
        let mut arrived = 0;
        for c in self.coins.iter_mut() {
            if c.delay > 0.0 {
                c.delay -= dt;
                continue;
            }
            c.t += dt * 1.25;
            if c.t >= 1.0 {
                arrived += 1;
            }
        }
        let _ = target;
        self.coins.retain(|c| c.t < 1.0);
        if arrived > 0 {
            self.gold_bump = 1.0;
        }
        // счётчик золота плавно догоняет настоящее значение
        let diff = game.gold - self.gold_shown;
        if diff.abs() < 1.0 || self.coins.is_empty() && diff.abs() < game.gold * 0.0005 {
            self.gold_shown = game.gold;
        } else {
            self.gold_shown += diff * (1.0 - (-dt as f64 * 8.0).exp());
        }

        // погода
        let weather = ZONES[self.zone].weather;
        let target_n = match weather {
            Weather::Snow => 90,
            Weather::Fireflies => 40,
            Weather::Embers => 60,
            Weather::Sparkles => 40,
            _ => 55,
        } + if game.skills[SKILL_STORM].active() {
            80
        } else {
            0
        };
        while self.weather.len() < target_n {
            let z = rand::gen_range(0.35, 1.35);
            let rising = matches!(
                weather,
                Weather::Embers | Weather::Fireflies | Weather::Sparkles
            );
            let y = if self.time < 0.5 {
                rand::gen_range(0.0, lay.h)
            } else if rising {
                lay.h + 20.0
            } else {
                -30.0
            };
            self.weather.push(Drift {
                pos: vec2(rand::gen_range(-40.0, lay.w + 120.0), y),
                vel: if rising {
                    vec2(
                        rand::gen_range(-10.0, 10.0),
                        -rand::gen_range(20.0, 50.0) * z,
                    )
                } else {
                    vec2(
                        -rand::gen_range(15.0, 45.0) * z,
                        rand::gen_range(30.0, 70.0) * z,
                    )
                },
                z,
                phase: rand::gen_range(0.0, std::f32::consts::TAU),
                rot: rand::gen_range(0.0, std::f32::consts::TAU),
                spin: rand::gen_range(-2.0, 2.0),
            });
        }
        let storm = game.skills[SKILL_STORM].active();
        for w in self.weather.iter_mut() {
            let sway = (self.time * 1.3 + w.phase).sin() * 28.0 * w.z;
            w.pos += (w.vel + vec2(sway, 0.0)) * dt;
            if storm {
                // вихрь вокруг ёкая
                let c = vec2(lay.monster_x, lay.ground - 140.0);
                let d = w.pos - c;
                w.pos += vec2(-d.y, d.x).normalize_or_zero() * 160.0 * dt * w.z;
            }
            w.rot += w.spin * dt;
        }
        let (ww, wh) = (lay.w, lay.h);
        self.weather.retain(|w| {
            w.pos.y < wh + 40.0 && w.pos.y > -60.0 && w.pos.x > -80.0 && w.pos.x < ww + 160.0
        });
        if self.weather.len() > target_n + 10 {
            self.weather.truncate(target_n + 10);
        }

        for c in self.clouds.iter_mut() {
            c.0 += dt * c.3 / 1000.0;
            if c.0 > 1.25 {
                c.0 = -0.3;
            }
        }

        if self.toast.is_none() {
            if let Some(t) = self.toasts.pop_front() {
                self.toast = Some((t, 0.0));
            }
        }
        if let Some((_, t)) = &mut self.toast {
            *t += dt;
            if *t > 3.4 {
                self.toast = None;
            }
        }
        if let Some(b) = &mut self.banner {
            b.t += dt;
            if b.t > b.dur {
                self.banner = None;
            }
        }
    }

    // ------------------------------------------------------------------ отрисовка

    /// Рисует сцену и обрабатывает клики по ней. `lay.w` — ширина поля боя.
    pub fn draw(&mut self, game: &mut Game, gfx: &Gfx, art: &mut Art, input: &Input, lay: &Layout) {
        let (w, h) = (lay.w, lay.h);
        let shake = if self.shake > 0.0 {
            vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake * 9.0
        } else {
            Vec2::ZERO
        };

        // ---------- фон
        self.draw_backdrop(gfx, art, self.zone, w, h, 1.0, shake);
        if let Some((pz, a)) = self.prev_zone {
            self.draw_backdrop(gfx, art, pz, w, h, a, shake);
        }
        self.draw_ambient(gfx, art, lay, false);

        // ---------- герои и ёкай
        self.draw_party(game, gfx, art, lay, shake);
        self.draw_monster(game, gfx, art, lay, shake);
        self.draw_sakura(game, gfx, art, lay, shake);

        // ---------- передний план
        let key = format!("fg/{}/{}x{}", self.zone, w as i32, h as i32);
        let zone = self.zone;
        let fg = art.get(&key, w, h, || scenery::foreground(zone, w, h));
        let sway = (self.time * 0.6).sin() * 3.0;
        gfx.sprite_ex(
            fg,
            sway + shake.x * 0.5,
            shake.y * 0.5,
            w,
            h,
            WHITE,
            0.0,
            None,
            false,
        );
        self.draw_ambient(gfx, art, lay, true);
        self.draw_effects(gfx, art, lay);

        // ---------- виньетка и вспышки
        let dark = Color::new(0.16, 0.05, 0.12, 0.28);
        vgrad(0.0, 0.0, w, 90.0, dark, Color::new(0.16, 0.05, 0.12, 0.0));
        vgrad(
            0.0,
            h - 120.0,
            w,
            120.0,
            Color::new(0.16, 0.05, 0.12, 0.0),
            dark,
        );
        hgrad(
            0.0,
            0.0,
            60.0,
            h,
            with_alpha(dark, 0.7),
            Color::new(0.16, 0.05, 0.12, 0.0),
        );
        if game.monster.boss && game.boss_time_left < game.boss_time() * 0.3 {
            let pulse = (self.time * 8.0).sin() * 0.5 + 0.5;
            vgrad(
                0.0,
                0.0,
                w,
                h * 0.5,
                Color::new(0.9, 0.1, 0.1, 0.12 * pulse),
                Color::new(0.9, 0.1, 0.1, 0.0),
            );
        }
        if self.red_flash > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                w,
                h,
                Color::new(0.85, 0.1, 0.15, self.red_flash * 0.35),
            );
        }
        if self.strike > 0.0 {
            self.draw_lightning(gfx, game, lay);
        }

        // ---------- интерфейс поверх сцены
        self.draw_hud(game, gfx, art, input, lay);
        self.draw_monster_plate(game, gfx, art, lay);
        self.draw_skills(game, gfx, art, input, lay);
        self.draw_banner(gfx, art, lay);
        self.draw_toast(gfx, lay);

        // ---------- атака по клику
        let battle = Rect::new(0.0, 104.0, w, h - 104.0 - 96.0);
        if input.pressed_in(battle) && !self.over_controls(input.mouse) {
            game.click();
            self.hint_shown = true;
        }
        if !self.hint_shown && game.total_clicks == 0 {
            let c = lay.monster_center(game.monster.boss);
            let pulse = (self.time * 4.0).sin() * 0.5 + 0.5;
            gfx.text_outline(
                "Кликайте по ёкаю!",
                c.x,
                c.y - 200.0 - pulse * 6.0,
                26.0,
                Font::Black,
                WHITE,
                hexc(0xC2185B),
                2.5,
                Align::Center,
            );
        }
    }

    fn over_controls(&self, m: Vec2) -> bool {
        // кнопки навыков и переключатели обрабатываются отдельно
        self.controls.iter().any(|r| r.contains(m))
    }

    fn draw_backdrop(
        &mut self,
        gfx: &Gfx,
        art: &mut Art,
        zone: usize,
        w: f32,
        h: f32,
        alpha: f32,
        shake: Vec2,
    ) {
        let key = format!("bg/{}/{}x{}", zone, w as i32, h as i32);
        let need_lights = self.lights_key != key && zone == self.zone;
        let mut lights = None;
        let bg = art.get(&key, w + 24.0, h + 24.0, || {
            let (svg, l) = scenery::backdrop(zone, w + 24.0, h + 24.0);
            lights = Some(l);
            svg
        });
        // лёгкий параллакс за мышью
        let m = gfx.mouse();
        let px = ((m.x / w).clamp(0.0, 1.0) - 0.5) * -10.0;
        let py = ((m.y / h).clamp(0.0, 1.0) - 0.5) * -6.0;
        gfx.sprite_ex(
            bg,
            -12.0 + px + shake.x * 0.3,
            -12.0 + py + shake.y * 0.3,
            w + 24.0,
            h + 24.0,
            Color::new(1.0, 1.0, 1.0, alpha),
            0.0,
            None,
            false,
        );
        if need_lights {
            match lights {
                Some(l) => {
                    self.lights = l
                        .into_iter()
                        .map(|mut li| {
                            li.x -= 12.0;
                            li.y -= 12.0;
                            li
                        })
                        .collect();
                    self.lights_key = key;
                }
                None => {
                    let (_, l) = scenery::backdrop(zone, w + 24.0, h + 24.0);
                    self.lights = l
                        .into_iter()
                        .map(|mut li| {
                            li.x -= 12.0;
                            li.y -= 12.0;
                            li
                        })
                        .collect();
                    self.lights_key = key;
                }
            }
        }
    }

    /// Погода, облака, лучи и огоньки. `front` — слой перед героями.
    fn draw_ambient(&self, gfx: &Gfx, art: &mut Art, lay: &Layout, front: bool) {
        let zone = &ZONES[self.zone];
        if !front {
            // облака
            if !zone.night {
                let cloud = art.get(&format!("cloud/{}", self.zone), 220.0, 110.0, || {
                    scenery::cloud_svg(if self.zone == 5 { 0x8D6E63 } else { 0xF3E5F5 })
                });
                for (x, y, sc, _) in self.clouds.iter() {
                    let cw = 220.0 * sc;
                    gfx.sprite_ex(
                        cloud,
                        x * (lay.w + cw) - cw,
                        y * lay.h,
                        cw,
                        110.0 * sc,
                        Color::new(1.0, 1.0, 1.0, 0.75),
                        0.0,
                        None,
                        false,
                    );
                }
            }
            // лучи света
            if matches!(self.zone, 0 | 1 | 6) {
                let origin = match self.zone {
                    0 => vec2(lay.w * 0.8, lay.h * 0.18),
                    1 => vec2(lay.w * 0.5, -40.0),
                    _ => vec2(lay.w * 0.5, -60.0),
                };
                for i in 0..5 {
                    let a = 1.75 + i as f32 * 0.22 + (self.time * 0.15 + i as f32).sin() * 0.05;
                    let len = lay.h * 1.2;
                    let spread = 0.05 + (i % 2) as f32 * 0.03;
                    let p1 = origin + vec2((a - spread).cos(), (a - spread).sin()) * len;
                    let p2 = origin + vec2((a + spread).cos(), (a + spread).sin()) * len;
                    let alpha = 0.05 + 0.03 * (self.time * 0.7 + i as f32 * 1.7).sin();
                    let mesh = Mesh {
                        vertices: vec![
                            Vertex::new(
                                origin.x,
                                origin.y,
                                0.0,
                                0.5,
                                0.5,
                                Color::new(1.0, 0.98, 0.9, alpha),
                            ),
                            Vertex::new(
                                p1.x,
                                p1.y,
                                0.0,
                                0.5,
                                0.5,
                                Color::new(1.0, 0.95, 0.85, 0.0),
                            ),
                            Vertex::new(
                                p2.x,
                                p2.y,
                                0.0,
                                0.5,
                                0.5,
                                Color::new(1.0, 0.95, 0.85, 0.0),
                            ),
                        ],
                        indices: vec![0, 1, 2],
                        texture: None,
                    };
                    gfx.additive_mesh(&mesh);
                }
            }
            // огоньки фонарей, окон, лавы
            for (i, l) in self.lights.iter().enumerate() {
                let flicker = 0.75
                    + 0.25
                        * (self.time * 3.0 + i as f32 * 1.7).sin()
                        * (self.time * 1.3 + i as f32).cos();
                gfx.glow(
                    l.x,
                    l.y,
                    l.r * (0.9 + flicker * 0.2),
                    hexa(l.color, 0.35 * flicker),
                );
            }
        }
        // частицы погоды: дальние позади, ближние спереди
        for w in self.weather.iter() {
            if (w.z > 1.0) != front {
                continue;
            }
            let a = (0.35 + w.z * 0.45).min(1.0);
            match zone.weather {
                Weather::Petals | Weather::Leaves => {
                    let (p, key) = if zone.weather == Weather::Petals {
                        (Particle::Petal, "part/petal")
                    } else {
                        (Particle::Leaf, "part/leaf")
                    };
                    let size = 14.0 * w.z;
                    let spr = art.get(key, 32.0, 32.0, || icons::particle_svg(p));
                    let flip = (self.time * 2.0 + w.phase).cos().abs().max(0.2);
                    gfx.sprite_ex(
                        spr,
                        w.pos.x - size * flip / 2.0,
                        w.pos.y - size / 2.0,
                        size * flip,
                        size,
                        Color::new(1.0, 1.0, 1.0, a),
                        w.rot,
                        None,
                        false,
                    );
                }
                Weather::Snow => gfx.glow(
                    w.pos.x,
                    w.pos.y,
                    5.0 * w.z,
                    Color::new(1.0, 1.0, 1.0, a * 0.9),
                ),
                Weather::Embers => {
                    gfx.glow(w.pos.x, w.pos.y, 6.0 * w.z, Color::new(1.0, 0.55, 0.2, a))
                }
                Weather::Fireflies => {
                    let blink = 0.5 + 0.5 * (self.time * 2.5 + w.phase * 3.0).sin();
                    gfx.glow(
                        w.pos.x,
                        w.pos.y,
                        10.0 * w.z,
                        Color::new(1.0, 0.95, 0.5, a * blink),
                    );
                }
                Weather::Sparkles => {
                    let spr = art.get("part/sparkle", 24.0, 24.0, || {
                        icons::particle_svg(Particle::Sparkle)
                    });
                    let s = 12.0 * w.z * (0.6 + 0.4 * (self.time * 3.0 + w.phase).sin().abs());
                    gfx.sprite_ex(
                        spr,
                        w.pos.x - s / 2.0,
                        w.pos.y - s / 2.0,
                        s,
                        s,
                        Color::new(1.0, 1.0, 1.0, a),
                        w.rot,
                        None,
                        false,
                    );
                }
            }
        }
    }

    fn party_slots(game: &Game, lay: &Layout) -> Vec<(usize, Vec2, f32)> {
        let mut out = Vec::new();
        let hired: Vec<usize> = (1..HEROINES.len())
            .filter(|&i| game.heroes[i].level > 0)
            .collect();
        let n = hired.len();
        for (k, &i) in hired.iter().enumerate() {
            let row = if k < 7 { 0 } else { 1 };
            let col = if row == 0 { k } else { k - 7 };
            let per_row = if row == 0 { n.min(7) } else { n - 7 };
            let span = lay.sakura_x - 110.0 - lay.w * 0.03;
            let step = if per_row > 1 {
                (span / (per_row - 1) as f32).min(62.0)
            } else {
                0.0
            };
            let x = lay.w * 0.04 + col as f32 * step + if row == 1 { step * 0.5 } else { 0.0 };
            let (y, scale) = if row == 0 {
                (lay.ground - 34.0, 0.4)
            } else {
                (lay.ground - 66.0, 0.34)
            };
            out.push((i, vec2(x + 30.0, y), scale));
        }
        out.sort_by(|a, b| a.1.y.partial_cmp(&b.1.y).unwrap());
        out
    }

    fn draw_party(&self, game: &Game, gfx: &Gfx, art: &mut Art, lay: &Layout, shake: Vec2) {
        let blessed = game.skills[SKILL_BLESSING].active();
        for (n, (i, pos, scale)) in Scene::party_slots(game, lay).into_iter().enumerate() {
            let h = &HEROINES[i];
            let (w, hh) = (chibi::W * scale, chibi::H * scale);
            let phase = self.time * 3.2 + n as f32 * 1.37;
            let hop = phase.sin().max(0.0).powf(3.0) * 10.0;
            let pos = pos + shake * 0.5;
            gfx.soft_shadow(pos.x, pos.y - 2.0, w * 0.36, 7.0, 0.3);
            if blessed {
                gfx.glow(
                    pos.x,
                    pos.y - hh * 0.45,
                    hh * 0.6,
                    Color::new(1.0, 0.85, 0.35, 0.35),
                );
            }
            let blink = (self.time * 0.7 + n as f32 * 2.1).sin() > 0.985;
            let key = format!("party/{}/{}/{}", h.id, blink as u8, (scale * 100.0) as i32);
            let look = h.look;
            let spr = art.get(&key, w, hh, || chibi::heroine_svg(&look, blink, true));
            gfx.sprite_ex(
                spr,
                pos.x - w / 2.0,
                pos.y - hh + 4.0 - hop,
                w,
                hh,
                WHITE,
                0.0,
                None,
                false,
            );
            if phase.sin() > 0.97 {
                gfx.glow(
                    pos.x + w * 0.3,
                    pos.y - hh * 0.6 - hop,
                    14.0,
                    Color::new(1.0, 0.9, 0.7, 0.6),
                );
            }
        }
    }

    fn draw_sakura(&self, game: &Game, gfx: &Gfx, art: &mut Art, lay: &Layout, shake: Vec2) {
        let (w, h) = (chibi::W * SAKURA_SCALE, chibi::H * SAKURA_SCALE);
        let lunge = (self.attack * std::f32::consts::PI).sin() * 26.0;
        let breathe = (self.time * 2.2).sin();
        let x = lay.sakura_x + lunge + shake.x * 0.7;
        let feet = lay.ground + 6.0 + shake.y * 0.7;
        gfx.soft_shadow(x, feet - 6.0, w * 0.34, 10.0, 0.35);
        let outfit = game.outfit;
        let look = sakura_look(&OUTFITS[outfit]);
        let blink = self.blink > 0.0;
        let key = format!("sakura/{}/{}", outfit, blink as u8);
        let spr = art.get(&key, w, h, || chibi::heroine_svg(&look, blink, false));
        let stretch = 1.0 + breathe * 0.008;
        let (dw, dh) = (w / stretch, h * stretch);
        let top = feet - dh * (326.0 / chibi::H);
        gfx.sprite_ex(spr, x - dw / 2.0, top, dw, dh, WHITE, 0.0, None, false);

        // катана в руке
        let hand = vec2(
            x - dw / 2.0 + chibi::HAND.0 / chibi::W * dw,
            top + chibi::HAND.1 / chibi::H * dh,
        );
        let angle = if self.attack > 0.0 {
            let t = 1.0 - self.attack;
            -0.6 + t.powf(0.6) * 2.7
        } else {
            0.45 + (self.time * 1.6).sin() * 0.04
        };
        let ks = SAKURA_SCALE;
        let kw = chibi::KATANA_W * ks;
        let kh = chibi::KATANA_H * ks;
        let pivot = vec2(chibi::KATANA_PIVOT.0 * ks, chibi::KATANA_PIVOT.1 * ks);
        if self.attack > 0.0 {
            // светящийся след взмаха
            for i in 1..5 {
                let ta = angle - i as f32 * 0.18;
                let tip = hand + vec2(ta.sin(), -ta.cos()) * kh * 0.75;
                gfx.glow(
                    tip.x,
                    tip.y,
                    26.0 - i as f32 * 4.0,
                    Color::new(1.0, 0.75, 0.85, 0.35 * self.attack),
                );
            }
        }
        let katana = art.get("katana", kw, kh, chibi::katana_svg);
        gfx.sprite_ex(
            katana,
            hand.x - pivot.x,
            hand.y - pivot.y,
            kw,
            kh,
            WHITE,
            angle,
            Some(hand),
            false,
        );
    }

    fn draw_monster(&mut self, game: &Game, gfx: &Gfx, art: &mut Art, lay: &Layout, shake: Vec2) {
        let zone_glow = game.monster.boss;
        // умирающий ёкай
        if let Some(d) = &self.dying {
            let size = if d.boss { BOSS_SIZE } else { MONSTER_SIZE };
            let c = lay.monster_center(d.boss);
            let def = &MONSTERS[d.def];
            let spr = art.get(
                &format!("mon/{}/{}", d.def, size as i32),
                size,
                size,
                || yokai::monster_svg(def),
            );
            let sc = 1.0 + d.t * 0.25;
            let a = (1.0 - d.t).max(0.0);
            gfx.sprite_ex(
                spr,
                c.x - size * sc / 2.0,
                c.y - size * sc / 2.0,
                size * sc,
                size * sc,
                Color::new(1.0, 1.0, 1.0, a),
                0.0,
                None,
                false,
            );
            gfx.silhouette_ex(
                spr,
                c.x - size * sc / 2.0,
                c.y - size * sc / 2.0,
                size * sc,
                size * sc,
                Color::new(1.0, 1.0, 1.0, a * d.t.min(0.6) * 1.6),
                false,
            );
        }
        if self.spawn_t <= 0.0 {
            return;
        }
        let m = &game.monster;
        let size = if m.boss { BOSS_SIZE } else { MONSTER_SIZE };
        let c = lay.monster_center(m.boss) + shake;
        let def = &MONSTERS[m.def];
        let t = self.spawn_t.clamp(0.0, 1.0);
        let pop = ease_out_back(t);
        let bob = (self.time * 2.4).sin() * 5.0;
        let squash = 1.0 - self.hit_flash * 0.09;
        let (sw, sh) = (size * pop / squash, size * pop * squash);
        gfx.soft_shadow(c.x, lay.ground - 4.0, size * 0.36 * pop, 14.0, 0.4 * t);
        if zone_glow {
            let pulse = 0.5 + 0.5 * (self.time * 2.0).sin();
            gfx.glow(
                c.x,
                c.y,
                size * 0.75,
                Color::new(0.9, 0.2, 0.4, 0.18 + 0.1 * pulse),
            );
        }
        let spr = art.get(
            &format!("mon/{}/{}", m.def, size as i32),
            size,
            size,
            || yokai::monster_svg(def),
        );
        let x = c.x - sw / 2.0;
        let y = c.y + size / 2.0 - sh - 4.0 + bob * (1.0 - self.hit_flash);
        let tilt = (self.time * 1.7).sin() * 0.03;
        gfx.sprite_ex(
            spr,
            x,
            y,
            sw,
            sh,
            Color::new(1.0, 1.0, 1.0, t.min(1.0)),
            tilt,
            None,
            false,
        );
        if self.hit_flash > 0.0 {
            gfx.silhouette_ex(
                spr,
                x,
                y,
                sw,
                sh,
                Color::new(1.0, 1.0, 1.0, self.hit_flash * 0.35),
                false,
            );
        }
    }

    fn draw_effects(&self, gfx: &Gfx, art: &mut Art, lay: &Layout) {
        for p in self.parts.iter() {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            match p.kind {
                PartKind::Smoke if p.color.a < 0.0 => {
                    // разрез
                    let spr = art.get("part/slash", 230.0, 115.0, || {
                        icons::particle_svg(Particle::Slash)
                    });
                    let (w, h) = (230.0 * p.size * (1.2 - a * 0.2), 115.0 * p.size);
                    gfx.sprite_ex(
                        spr,
                        p.pos.x - w / 2.0,
                        p.pos.y - h / 2.0,
                        w,
                        h,
                        Color::new(1.0, 1.0, 1.0, a),
                        p.rot,
                        None,
                        false,
                    );
                    gfx.glow(
                        p.pos.x,
                        p.pos.y,
                        60.0 * p.size,
                        Color::new(1.0, 0.6, 0.8, 0.16 * a),
                    );
                }
                PartKind::Smoke => gfx.glow(
                    p.pos.x,
                    p.pos.y,
                    p.size,
                    Color::new(1.0, 0.97, 1.0, a * 0.35),
                ),
                PartKind::Spark => gfx.glow(
                    p.pos.x,
                    p.pos.y,
                    p.size * 1.4,
                    Color::new(1.0, 0.85, 0.6, a),
                ),
                PartKind::Petal => {
                    let spr = art.get("part/petal", 32.0, 32.0, || {
                        icons::particle_svg(Particle::Petal)
                    });
                    gfx.sprite_ex(
                        spr,
                        p.pos.x - p.size,
                        p.pos.y - p.size,
                        p.size * 2.0,
                        p.size * 2.0,
                        Color::new(1.0, 1.0, 1.0, a),
                        p.rot,
                        None,
                        false,
                    );
                }
                PartKind::Coin => {
                    let spr = art.get("icon/coin/24", 24.0, 24.0, || icons::icon_svg(Icon::Coin));
                    let flip = (p.rot * 2.0).cos().abs().max(0.25);
                    gfx.sprite_ex(
                        spr,
                        p.pos.x - p.size * flip / 2.0,
                        p.pos.y - p.size / 2.0,
                        p.size * flip,
                        p.size,
                        Color::new(1.0, 1.0, 1.0, a),
                        0.0,
                        None,
                        false,
                    );
                }
                PartKind::Sparkle => {
                    let spr = art.get("part/sparkle", 24.0, 24.0, || {
                        icons::particle_svg(Particle::Sparkle)
                    });
                    gfx.sprite_ex(
                        spr,
                        p.pos.x - p.size / 2.0,
                        p.pos.y - p.size / 2.0,
                        p.size,
                        p.size,
                        Color::new(1.0, 1.0, 1.0, a),
                        p.rot,
                        None,
                        false,
                    );
                }
            }
        }
        // летящие к счётчику монеты
        for c in self.coins.iter() {
            if c.delay > 0.0 {
                continue;
            }
            let t = c.t;
            let e = t * t * (3.0 - 2.0 * t);
            let p = c.from * (1.0 - e) * (1.0 - e)
                + c.ctrl * 2.0 * e * (1.0 - e)
                + self.coin_target * e * e;
            let spr = art.get("icon/coin/24", 24.0, 24.0, || icons::icon_svg(Icon::Coin));
            let s = 22.0 * (1.0 - t * 0.3);
            let flip = (self.time * 12.0 + c.from.x).cos().abs().max(0.25);
            gfx.glow(p.x, p.y, 14.0, Color::new(1.0, 0.8, 0.3, 0.4));
            gfx.sprite_ex(
                spr,
                p.x - s * flip / 2.0,
                p.y - s / 2.0,
                s * flip,
                s,
                WHITE,
                0.0,
                None,
                false,
            );
        }
        // цифры урона
        for t in self.texts.iter() {
            let age = t.max - t.life;
            let pop = if age < 0.12 {
                0.6 + age / 0.12 * 0.7
            } else {
                1.3 - ((age - 0.12) * 3.0).min(0.3)
            };
            let a = (t.life / t.max * 2.0).clamp(0.0, 1.0);
            let size = t.size * pop;
            if t.crit {
                gfx.text_outline(
                    "КРИТ!",
                    t.pos.x,
                    t.pos.y - size * 0.75,
                    size * 0.5,
                    Font::Black,
                    with_alpha(hexc(0xFFF59D), a),
                    Color::new(0.55, 0.1, 0.25, a),
                    2.0,
                    Align::Center,
                );
            }
            gfx.text_outline(
                &t.text,
                t.pos.x,
                t.pos.y,
                size,
                Font::Black,
                with_alpha(t.color, a),
                Color::new(0.29, 0.1, 0.2, a * 0.9),
                size * 0.08,
                Align::Center,
            );
        }
        let _ = lay;
    }

    fn draw_lightning(&self, gfx: &Gfx, game: &Game, lay: &Layout) {
        let c = lay.monster_center(game.monster.boss);
        let a = self.strike;
        draw_rectangle(
            0.0,
            0.0,
            lay.w,
            lay.h,
            Color::new(1.0, 1.0, 0.95, a * a * 0.5),
        );
        let mut p = vec2(c.x + 40.0, -20.0);
        let mut seed = 7.0f32;
        while p.y < c.y {
            seed = (seed * 9.13 + 3.7) % 17.0;
            let next = vec2(c.x + (seed - 8.5) * 9.0, p.y + 50.0);
            draw_line(
                p.x,
                p.y,
                next.x,
                next.y,
                10.0 * a,
                Color::new(0.7, 0.9, 1.0, a),
            );
            draw_line(
                p.x,
                p.y,
                next.x,
                next.y,
                4.0 * a,
                Color::new(1.0, 1.0, 1.0, a),
            );
            p = next;
        }
        gfx.glow(c.x, c.y, 220.0 * a, Color::new(0.6, 0.85, 1.0, a));
    }

    fn draw_hud(&mut self, game: &mut Game, gfx: &Gfx, art: &mut Art, input: &Input, lay: &Layout) {
        self.controls.clear();
        // ---------- золото
        let (x, y, w, h) = (14.0, 12.0, 300.0, 84.0);
        shadow(x, y, w, h, 20.0, 10.0, 0.35);
        rrect_grad(
            x,
            y,
            w,
            h,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.92),
            Color::new(1.0, 0.93, 0.96, 0.92),
        );
        rrect_line(x, y, w, h, 20.0, 2.0, hexa(0xF3C1D5, 1.0));
        let bump = 1.0 + self.gold_bump * 0.18;
        self.coin_target = vec2(x + 36.0, y + 32.0);
        if game.skills[SKILL_FAN].active() {
            gfx.glow(
                x + 36.0,
                y + 32.0,
                40.0 + (self.time * 6.0).sin() * 6.0,
                Color::new(1.0, 0.8, 0.3, 0.6),
            );
        }
        let coin = art.get("icon/coin/40", 40.0, 40.0, || icons::icon_svg(Icon::Coin));
        gfx.sprite(coin, x + 36.0, y + 32.0, bump, WHITE);
        let gold = format!("{} рё", fmt::num(self.gold_shown.max(0.0)));
        gfx.text(
            &gold,
            x + 64.0,
            y + 31.0,
            30.0 * (1.0 + self.gold_bump * 0.06),
            Font::Black,
            hexc(0x8A5A06),
            Align::Left,
        );
        let sword = art.get("icon/sword/18", 18.0, 18.0, || icons::icon_svg(Icon::Sword));
        gfx.sprite(sword, x + 24.0, y + 64.0, 1.0, WHITE);
        let tw = gfx.text(
            &fmt::num(game.click_damage()),
            x + 36.0,
            y + 64.0,
            15.0,
            Font::Bold,
            hexc(0x6B3A50),
            Align::Left,
        );
        let party = art.get("icon/party/18", 18.0, 18.0, || icons::icon_svg(Icon::Party));
        gfx.sprite(party, x + 56.0 + tw, y + 64.0, 1.0, WHITE);
        gfx.text(
            &format!("{} / с", fmt::num(game.dps())),
            x + 68.0 + tw,
            y + 64.0,
            15.0,
            Font::Bold,
            hexc(0x6B3A50),
            Align::Left,
        );

        // ---------- этап
        let cx = (lay.w * 0.5).max(x + w + 150.0).min(lay.w - 290.0);
        let (nw, nh) = (250.0, 84.0);
        let (nx, ny) = (cx - nw / 2.0, 12.0);
        shadow(nx, ny, nw, nh, 20.0, 10.0, 0.35);
        rrect_grad(
            nx,
            ny,
            nw,
            nh,
            20.0,
            Color::new(1.0, 1.0, 1.0, 0.92),
            Color::new(1.0, 0.93, 0.96, 0.92),
        );
        rrect_line(nx, ny, nw, nh, 20.0, 2.0, hexa(0xF3C1D5, 1.0));
        let prev = Rect::new(nx + 8.0, ny + 14.0, 36.0, 36.0);
        let next = Rect::new(nx + nw - 44.0, ny + 14.0, 36.0, 36.0);
        self.controls.push(prev);
        self.controls.push(next);
        arrow_button(gfx, prev, true, game.stage > 1, input.hover(prev));
        arrow_button(
            gfx,
            next,
            false,
            game.stage < game.max_stage,
            input.hover(next),
        );
        if input.clicked(prev) && game.stage > 1 {
            game.auto_advance = false;
            game.set_stage(game.stage - 1);
        }
        if input.clicked(next) && game.stage < game.max_stage {
            game.set_stage(game.stage + 1);
        }
        gfx.text(
            &format!("Этап {}", game.stage),
            cx,
            ny + 30.0,
            26.0,
            Font::Black,
            hexc(0xB83B6B),
            Align::Center,
        );
        gfx.text(
            &zone_name(game.stage),
            cx,
            ny + 54.0,
            13.0,
            Font::Bold,
            hexc(0x8C6A7A),
            Align::Center,
        );
        // прогресс этапа
        if game.monster.boss {
            let crown = art.get("icon/crown/18", 18.0, 18.0, || icons::icon_svg(Icon::Crown));
            gfx.sprite(crown, nx + 22.0, ny + 70.0, 1.0, WHITE);
            let frac = (game.boss_time_left / game.boss_time()) as f32;
            let urgent = frac < 0.3;
            progress_bar(
                nx + 36.0,
                ny + 64.0,
                nw - 98.0,
                12.0,
                frac,
                Color::new(0.3, 0.1, 0.2, 0.15),
                if urgent {
                    hexc(0xFF5252)
                } else {
                    hexc(0xFFB300)
                },
                if urgent {
                    hexc(0xD50000)
                } else {
                    hexc(0xFF6F00)
                },
            );
            gfx.text(
                &format!("{:.1} с", game.boss_time_left.max(0.0)),
                nx + nw - 14.0,
                ny + 70.0,
                13.0,
                Font::Black,
                if urgent {
                    hexc(0xD50000)
                } else {
                    hexc(0x8A5A06)
                },
                Align::Right,
            );
        } else if game.stage_cleared(game.stage) {
            gfx.text(
                "этап пройден",
                cx,
                ny + 72.0,
                12.0,
                Font::Bold,
                hexc(0x43A047),
                Align::Center,
            );
        } else {
            for i in 0..MONSTERS_PER_STAGE {
                let dx = cx - 63.0 + i as f32 * 14.0;
                let done = i < game.kills;
                draw_circle(
                    dx,
                    ny + 72.0,
                    4.5,
                    if done {
                        hexc(0xE0598B)
                    } else {
                        Color::new(0.85, 0.75, 0.8, 0.8)
                    },
                );
            }
        }

        // ---------- переключатель «вперёд»
        let ax = lay.w - 176.0;
        let auto = Rect::new(ax, 20.0, 160.0, 34.0);
        self.controls.push(auto);
        let hover = input.hover(auto);
        shadow(auto.x, auto.y, auto.w, auto.h, 17.0, 6.0, 0.3);
        rrect_grad(
            auto.x,
            auto.y,
            auto.w,
            auto.h,
            17.0,
            Color::new(1.0, 1.0, 1.0, 0.94),
            Color::new(1.0, 0.93, 0.96, 0.94),
        );
        let sw = Rect::new(auto.x + auto.w - 56.0, auto.y + 7.0, 46.0, 20.0);
        rrect(
            sw.x,
            sw.y,
            sw.w,
            sw.h,
            10.0,
            if game.auto_advance {
                hexc(0xE0598B)
            } else {
                hexc(0xCDB8C2)
            },
        );
        let knob = if game.auto_advance {
            sw.x + sw.w - 10.0
        } else {
            sw.x + 10.0
        };
        draw_circle(
            knob,
            sw.y + 10.0,
            8.0 + if hover { 1.0 } else { 0.0 },
            WHITE,
        );
        gfx.text(
            "Вперёд",
            auto.x + 14.0,
            auto.y + 17.0,
            15.0,
            Font::Black,
            hexc(0x8C4A68),
            Align::Left,
        );
        gfx.text_outline(
            &format!("рекорд: {}", game.best_stage),
            auto.x + auto.w / 2.0,
            auto.y + 50.0,
            13.0,
            Font::Black,
            WHITE,
            hexa(0x8C3A60, 0.85),
            1.6,
            Align::Center,
        );
        if input.clicked(auto) {
            game.auto_advance = !game.auto_advance;
            if game.auto_advance && game.stage_cleared(game.stage) && !game.monster.boss {
                game.set_stage(game.stage + 1);
            }
        }
    }

    fn draw_monster_plate(&self, game: &Game, gfx: &Gfx, art: &mut Art, lay: &Layout) {
        let m = &game.monster;
        let size = if m.boss { BOSS_SIZE } else { MONSTER_SIZE };
        let c = lay.monster_center(m.boss);
        let top = (c.y - size * 0.5 - 40.0).max(118.0);
        let bw = 280.0;
        let x = (c.x - bw / 2.0).min(lay.w - bw - 12.0);
        let name = MONSTERS[m.def].name;
        let cx = x + bw / 2.0;
        if m.boss {
            let crown = art.get("icon/crown/26", 26.0, 26.0, || icons::icon_svg(Icon::Crown));
            let nw = gfx.measure(name, 22.0, Font::Black);
            gfx.sprite(crown, cx - nw / 2.0 - 18.0, top - 2.0, 1.0, WHITE);
        }
        gfx.text_outline(
            name,
            cx,
            top,
            22.0,
            Font::Black,
            if m.boss { hexc(0xFFE082) } else { WHITE },
            hexc(0x4A2C3A),
            2.4,
            Align::Center,
        );
        let frac = (m.hp / m.max_hp).clamp(0.0, 1.0) as f32;
        rrect(
            x - 2.0,
            top + 14.0,
            bw + 4.0,
            22.0,
            11.0,
            Color::new(0.29, 0.17, 0.23, 0.75),
        );
        progress_bar(
            x,
            top + 16.0,
            bw,
            18.0,
            frac,
            Color::new(0.0, 0.0, 0.0, 0.0),
            hexc(0xFF7FA3),
            hexc(0xE03257),
        );
        gfx.text_outline(
            &format!("{} / {}", fmt::num(m.hp.max(0.0)), fmt::num(m.max_hp)),
            cx,
            top + 25.0,
            12.0,
            Font::Black,
            WHITE,
            Color::new(0.29, 0.1, 0.2, 0.8),
            1.2,
            Align::Center,
        );
    }

    fn draw_skills(
        &mut self,
        game: &mut Game,
        gfx: &Gfx,
        art: &mut Art,
        input: &Input,
        lay: &Layout,
    ) {
        let r = 30.0;
        let gap = 84.0;
        let cx0 = lay.w * 0.5 - gap * 1.5;
        let cy = lay.h - 52.0;
        // подложка
        let (bx, bw) = (cx0 - r - 18.0, gap * 3.0 + r * 2.0 + 36.0);
        shadow(bx, cy - r - 12.0, bw, r * 2.0 + 24.0, 36.0, 10.0, 0.35);
        rrect_grad(
            bx,
            cy - r - 12.0,
            bw,
            r * 2.0 + 24.0,
            36.0,
            Color::new(1.0, 1.0, 1.0, 0.85),
            Color::new(1.0, 0.9, 0.95, 0.85),
        );
        let icons_list = [Icon::Storm, Icon::Blessing, Icon::Fan, Icon::Strike];
        let mut tooltip = None;
        for i in 0..4 {
            let def = &SKILLS[i];
            let st = game.skills[i];
            let c = vec2(cx0 + i as f32 * gap, cy);
            let rect = Rect::new(c.x - r, c.y - r, r * 2.0, r * 2.0);
            self.controls.push(rect);
            let unlocked = game.skill_unlocked(i);
            let hover = input.hover(rect);
            let ready = unlocked && st.ready();
            if st.active() {
                gfx.glow(
                    c.x,
                    c.y,
                    r * 1.9 + (self.time * 6.0).sin() * 4.0,
                    Color::new(1.0, 0.75, 0.3, 0.55),
                );
            } else if ready {
                gfx.glow(
                    c.x,
                    c.y,
                    r * 1.5 + (self.time * 3.0).sin() * 3.0,
                    Color::new(1.0, 0.5, 0.7, 0.35),
                );
            }
            let base = if !unlocked {
                hexc(0xC9B6C0)
            } else if st.active() {
                hexc(0xFFB74D)
            } else {
                hexc(0xF06292)
            };
            draw_circle(c.x, c.y + 2.0, r + 1.0, Color::new(0.3, 0.1, 0.2, 0.25));
            draw_circle(
                c.x,
                c.y,
                r,
                lerp_color(base, WHITE, if hover && unlocked { 0.15 } else { 0.0 }),
            );
            draw_circle(c.x, c.y - r * 0.2, r * 0.8, Color::new(1.0, 1.0, 1.0, 0.18));
            let key = format!("icon/skill{i}/40");
            let spr = art.get(&key, 40.0, 40.0, || icons::icon_svg(icons_list[i]));
            if unlocked {
                gfx.sprite(spr, c.x, c.y, 1.0, WHITE);
            } else {
                gfx.silhouette_ex(
                    spr,
                    c.x - 20.0,
                    c.y - 20.0,
                    40.0,
                    40.0,
                    Color::new(0.45, 0.35, 0.4, 0.6),
                    false,
                );
                let lock = art.get("icon/lock/20", 20.0, 20.0, || icons::icon_svg(Icon::Lock));
                gfx.sprite(lock, c.x + r * 0.6, c.y + r * 0.6, 1.0, WHITE);
            }
            if unlocked && !st.ready() && !st.active() {
                pie(
                    c.x,
                    c.y,
                    r,
                    (st.cooldown_left / def.cooldown) as f32,
                    Color::new(0.2, 0.05, 0.12, 0.55),
                );
                gfx.text_outline(
                    &short_time(st.cooldown_left),
                    c.x,
                    c.y,
                    14.0,
                    Font::Black,
                    WHITE,
                    Color::new(0.2, 0.05, 0.12, 0.9),
                    1.5,
                    Align::Center,
                );
            }
            if st.active() {
                arc(
                    c.x,
                    c.y,
                    r + 4.0,
                    4.0,
                    -std::f32::consts::FRAC_PI_2,
                    -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::TAU * (st.active_left / def.duration.max(0.01)) as f32,
                    hexc(0xFFE082),
                );
            }
            // номер клавиши
            draw_circle(c.x - r * 0.72, c.y - r * 0.72, 10.0, hexc(0x4A2C3A));
            gfx.text(
                &format!("{}", i + 1),
                c.x - r * 0.72,
                c.y - r * 0.72,
                12.0,
                Font::Black,
                WHITE,
                Align::Center,
            );
            if hover {
                tooltip = Some((i, c));
            }
            if input.clicked(rect) {
                self.try_skill(game, i);
            }
        }
        if let Some((i, c)) = tooltip {
            let def = &SKILLS[i];
            let st = game.skills[i];
            let line2 = if !game.skill_unlocked(i) {
                format!("Откроется на этапе {}", def.unlock_stage)
            } else if st.active() {
                format!("Действует ещё {}", fmt::time(st.active_left + 0.99))
            } else if !st.ready() {
                format!("Перезарядка: {}", fmt::time(st.cooldown_left + 0.99))
            } else {
                format!("Готово • перезарядка {}", fmt::time(def.cooldown))
            };
            let w = gfx
                .measure(def.desc, 14.0, Font::Bold)
                .max(gfx.measure(&line2, 13.0, Font::Bold))
                .max(gfx.measure(def.name, 17.0, Font::Black))
                + 28.0;
            let x = (c.x - w / 2.0).clamp(8.0, lay.w - w - 8.0);
            let y = c.y - r - 96.0;
            shadow(x, y, w, 78.0, 14.0, 8.0, 0.4);
            rrect(x, y, w, 78.0, 14.0, Color::new(0.24, 0.12, 0.19, 0.95));
            gfx.text(
                def.name,
                x + 14.0,
                y + 18.0,
                17.0,
                Font::Black,
                hexc(0xFFD1E3),
                Align::Left,
            );
            gfx.text(
                def.desc,
                x + 14.0,
                y + 41.0,
                14.0,
                Font::Bold,
                WHITE,
                Align::Left,
            );
            gfx.text(
                &line2,
                x + 14.0,
                y + 61.0,
                13.0,
                Font::Bold,
                hexc(0xD8B4C6),
                Align::Left,
            );
        }
    }

    pub fn try_skill(&mut self, game: &mut Game, i: usize) {
        let def = &SKILLS[i];
        if !game.skill_unlocked(i) {
            self.message(format!(
                "«{}» откроется на этапе {}",
                def.name, def.unlock_stage
            ));
        } else if !game.skills[i].ready() {
            self.message(format!(
                "«{}» перезаряжается: {}",
                def.name,
                fmt::time(game.skills[i].cooldown_left)
            ));
        } else {
            game.use_skill(i);
        }
    }

    fn draw_banner(&self, gfx: &Gfx, art: &mut Art, lay: &Layout) {
        let Some(b) = &self.banner else { return };
        let t = b.t;
        let appear = ease_out_back((t / 0.35).min(1.0));
        let fade = ((b.dur - t) / 0.4).clamp(0.0, 1.0);
        let cy = lay.h * 0.36;
        let w =
            (gfx.measure(&b.title, 40.0, Font::Black)
                .max(gfx.measure(&b.sub, 18.0, Font::Bold))
                + 120.0)
                * appear;
        let h = 96.0;
        let x = lay.w * 0.5 - w / 2.0;
        let col = hexc(b.color);
        // «мазок кисти»
        gfx.glow_ellipse(
            lay.w * 0.5,
            cy,
            w * 0.75,
            90.0,
            with_alpha(col, 0.25 * fade),
        );
        rrect_grad(
            x,
            cy - h / 2.0,
            w,
            h,
            30.0,
            with_alpha(lerp_color(col, WHITE, 0.25), 0.92 * fade),
            with_alpha(lerp_color(col, BLACK, 0.15), 0.92 * fade),
        );
        rrect_line(
            x + 6.0,
            cy - h / 2.0 + 6.0,
            w - 12.0,
            h - 12.0,
            24.0,
            2.0,
            Color::new(1.0, 1.0, 1.0, 0.5 * fade),
        );
        if appear > 0.8 {
            let blossom = art.get("icon/gift/40", 40.0, 40.0, || icons::icon_svg(Icon::Gift));
            gfx.sprite(blossom, x + 34.0, cy, 1.0, Color::new(1.0, 1.0, 1.0, fade));
            gfx.sprite(
                blossom,
                x + w - 34.0,
                cy,
                1.0,
                Color::new(1.0, 1.0, 1.0, fade),
            );
            gfx.text_outline(
                &b.title,
                lay.w * 0.5,
                cy - 12.0,
                38.0,
                Font::Black,
                Color::new(1.0, 1.0, 1.0, fade),
                Color::new(0.25, 0.05, 0.15, 0.5 * fade),
                2.0,
                Align::Center,
            );
            gfx.text(
                &b.sub,
                lay.w * 0.5,
                cy + 26.0,
                18.0,
                Font::Bold,
                Color::new(1.0, 0.95, 0.97, fade),
                Align::Center,
            );
        }
    }

    fn draw_toast(&self, gfx: &Gfx, lay: &Layout) {
        let Some((text, t)) = &self.toast else { return };
        let slide = ease_out_back((*t / 0.3).min(1.0));
        let fade = ((3.4 - t) / 0.4).clamp(0.0, 1.0);
        let w = (gfx.measure(text, 16.0, Font::Bold) + 56.0).min(lay.w - 24.0);
        let x = lay.w * 0.5 - w / 2.0;
        let y = 104.0 + (1.0 - slide) * -30.0;
        shadow(x, y, w, 40.0, 20.0, 8.0, 0.35 * fade);
        rrect(
            x,
            y,
            w,
            40.0,
            20.0,
            Color::new(0.27, 0.12, 0.2, 0.92 * fade),
        );
        draw_circle(x + 22.0, y + 20.0, 7.0, with_alpha(hexc(0xF48FB1), fade));
        draw_circle(x + 22.0, y + 20.0, 3.0, with_alpha(WHITE, fade));
        gfx.text(
            text,
            x + 38.0,
            y + 20.0,
            16.0,
            Font::Bold,
            Color::new(1.0, 1.0, 1.0, fade),
            Align::Left,
        );
    }
}

fn arrow_button(gfx: &Gfx, r: Rect, left: bool, enabled: bool, hover: bool) {
    let c = vec2(r.x + r.w / 2.0, r.y + r.h / 2.0);
    let base = if !enabled {
        hexc(0xE8D3DC)
    } else if hover {
        hexc(0xC2185B)
    } else {
        hexc(0xE0598B)
    };
    draw_circle(c.x, c.y + 2.0, r.w / 2.0, Color::new(0.3, 0.1, 0.2, 0.2));
    draw_circle(c.x, c.y, r.w / 2.0, base);
    let d = if left { -1.0 } else { 1.0 };
    draw_triangle(
        vec2(c.x + 7.0 * d, c.y),
        vec2(c.x - 5.0 * d, c.y - 9.0),
        vec2(c.x - 5.0 * d, c.y + 9.0),
        WHITE,
    );
    let _ = gfx;
}

fn short_time(s: f64) -> String {
    let s = s.ceil() as u64;
    if s >= 60 {
        format!("{}:{:02}", s / 60, s % 60)
    } else {
        format!("{s}")
    }
}

pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}
