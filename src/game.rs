//! Вся логика игры: бой, золото, героини, наряды, навыки, перерождение.
//! Ничего не знает о графике — общается со сценой через события.

use crate::data::*;

pub const MONSTERS_PER_STAGE: u32 = 10;
pub const BOSS_EVERY: u32 = 5;
pub const REBIRTH_MIN_STAGE: u32 = 30;
pub const CRIT_MULT: f64 = 5.0;
pub const OFFLINE_LIMIT_SEC: f64 = 8.0 * 3600.0;
/// Уровни, на которых открывается «Свиток силы» (×2 к урону героини).
pub const MILESTONES: [u32; 8] = [10, 25, 50, 75, 100, 150, 200, 300];
const MILESTONE_COST: [f64; 8] = [
    5.0,
    25.0,
    100.0,
    500.0,
    2_500.0,
    15_000.0,
    100_000.0,
    1_000_000.0,
];

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Hit {
        damage: f64,
        crit: bool,
        click: bool,
    },
    Kill {
        gold: f64,
        boss: bool,
    },
    Message(String),
    StageChanged,
    BossFailed,
    OutfitUnlocked(usize),
    SkillUsed(usize),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HeroState {
    pub level: u32,
    pub upgrades: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SkillState {
    pub active_left: f64,
    pub cooldown_left: f64,
}

impl SkillState {
    pub fn active(&self) -> bool {
        self.active_left > 0.0
    }
    pub fn ready(&self) -> bool {
        self.cooldown_left <= 0.0
    }
}

#[derive(Clone, Debug)]
pub struct Monster {
    pub def: usize,
    pub hp: f64,
    pub max_hp: f64,
    pub gold: f64,
    pub boss: bool,
    /// Уникальный номер — сцена по нему понимает, что появился новый ёкай.
    pub serial: u64,
}

pub struct Game {
    pub gold: f64,
    pub total_gold: f64,
    pub stage: u32,
    /// Самый дальний открытый этап в текущем перерождении.
    pub max_stage: u32,
    /// Рекорд за всё время.
    pub best_stage: u32,
    /// Побеждено ёкаев на текущем этапе.
    pub kills: u32,
    pub auto_advance: bool,
    pub monster: Monster,
    pub boss_time_left: f64,

    pub heroes: Vec<HeroState>,
    pub outfit: usize,
    pub outfits_owned: Vec<bool>,
    pub skills: [SkillState; 4],
    pub talismans: [u32; 5],

    pub souls: u64,
    pub souls_spent: u64,
    pub rebirths: u32,
    pub total_clicks: u64,
    pub total_kills: u64,
    pub bosses_killed: u64,
    pub play_time: f64,
    /// Сколько уровней покупать за раз (0 — максимум).
    pub buy_amount: u32,

    pub events: Vec<Event>,
    /// Счётчик важных изменений (покупки и т.п.) — по нему игра понимает, что пора сохраниться.
    pub changes: u64,
    rng: u64,
    storm_acc: f64,
    serial: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let mut heroes = vec![HeroState::default(); HEROINES.len()];
        heroes[0].level = 1;
        let mut g = Game {
            gold: 0.0,
            total_gold: 0.0,
            stage: 1,
            max_stage: 1,
            best_stage: 1,
            kills: 0,
            auto_advance: true,
            monster: Monster {
                def: 0,
                hp: 1.0,
                max_hp: 1.0,
                gold: 0.0,
                boss: false,
                serial: 0,
            },
            boss_time_left: 0.0,
            heroes,
            outfit: 0,
            outfits_owned: OUTFITS.iter().map(|o| o.unlock_stage == 0).collect(),
            skills: [SkillState::default(); 4],
            talismans: [0; 5],
            souls: 0,
            souls_spent: 0,
            rebirths: 0,
            total_clicks: 0,
            total_kills: 0,
            bosses_killed: 0,
            play_time: 0.0,
            buy_amount: 1,
            events: Vec::new(),
            changes: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
            storm_acc: 0.0,
            serial: 0,
        };
        g.spawn();
        g
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }

    fn rand(&mut self) -> f64 {
        // xorshift64*
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    // ---------------------------------------------------------------- множители

    pub fn outfit_def(&self) -> &'static OutfitDef {
        &OUTFITS[self.outfit]
    }

    pub fn available_souls(&self) -> u64 {
        self.souls.saturating_sub(self.souls_spent)
    }

    /// Каждая душа сакуры: +10% ко всему урону.
    pub fn soul_mult(&self) -> f64 {
        1.0 + 0.10 * self.souls as f64
    }

    /// Каждый собранный наряд: +5% ко всему урону.
    pub fn collection_mult(&self) -> f64 {
        let n = OUTFITS
            .iter()
            .zip(&self.outfits_owned)
            .filter(|(o, owned)| **owned && o.unlock_stage > 0)
            .count();
        1.0 + 0.05 * n as f64
    }

    pub fn hero_mult(&self, i: usize) -> f64 {
        2f64.powi(self.heroes[i].upgrades as i32)
    }

    /// Урон героини в секунду без глобальных бонусов.
    pub fn hero_raw_dps(&self, i: usize) -> f64 {
        HEROINES[i].base_dps * self.heroes[i].level as f64 * self.hero_mult(i)
    }

    /// Общий множитель урона отряда.
    pub fn dps_mult(&self) -> f64 {
        let mut m = self.soul_mult()
            * self.collection_mult()
            * self.outfit_def().dps_mult
            * (1.0 + 0.15 * self.talismans[TAL_SPIRIT] as f64);
        if self.skills[SKILL_BLESSING].active() {
            m *= 2.0;
        }
        m
    }

    pub fn hero_dps(&self, i: usize) -> f64 {
        self.hero_raw_dps(i) * self.dps_mult()
    }

    pub fn dps(&self) -> f64 {
        let sum: f64 = (1..HEROINES.len()).map(|i| self.hero_raw_dps(i)).sum();
        sum * self.dps_mult()
    }

    /// Урон удара Сакуры. Каждый её «Свиток силы» добавляет ещё 1% урона отряда.
    pub fn click_damage(&self) -> f64 {
        let base = self.heroes[0].level as f64 * self.hero_mult(0);
        let m = self.soul_mult()
            * self.collection_mult()
            * self.outfit_def().click_mult
            * (1.0 + 0.25 * self.talismans[TAL_BLADE] as f64);
        base * m + self.dps() * 0.01 * self.heroes[0].upgrades as f64
    }

    pub fn gold_mult(&self, with_skills: bool) -> f64 {
        let mut m = self.outfit_def().gold_mult * (1.0 + 0.10 * self.talismans[TAL_WEALTH] as f64);
        if with_skills && self.skills[SKILL_FAN].active() {
            m *= 2.0;
        }
        m
    }

    pub fn crit_chance(&self) -> f64 {
        (0.03 + self.outfit_def().crit_add + 0.02 * self.talismans[TAL_LUCK] as f64).min(0.5)
    }

    pub fn boss_time(&self) -> f64 {
        30.0 + self.outfit_def().timer_add + 2.0 * self.talismans[TAL_HOURGLASS] as f64
    }

    // ---------------------------------------------------------------- ёкаи

    pub fn is_boss_stage(s: u32) -> bool {
        s.is_multiple_of(BOSS_EVERY)
    }

    pub fn monster_hp(s: u32) -> f64 {
        (10.0 * 1.39f64.powi(s as i32 - 1) + 5.0 * (s as f64 - 1.0)).ceil()
    }

    pub fn monster_gold(s: u32) -> f64 {
        (Self::monster_hp(s) / 12.0).ceil().max(1.0)
    }

    fn spawn(&mut self) {
        let boss = Self::is_boss_stage(self.stage);
        let zone = &ZONES[zone_index(self.stage)];
        let def = if boss {
            zone.boss
        } else {
            zone.monsters[(self.rand() * 3.0) as usize % 3]
        };
        let hp = Self::monster_hp(self.stage) * if boss { 8.0 } else { 1.0 };
        let gold = Self::monster_gold(self.stage) * if boss { 5.0 } else { 1.0 };
        self.serial += 1;
        self.monster = Monster {
            def,
            hp,
            max_hp: hp,
            gold,
            boss,
            serial: self.serial,
        };
        self.boss_time_left = if boss { self.boss_time() } else { 0.0 };
    }

    pub fn stage_cleared(&self, s: u32) -> bool {
        s < self.max_stage
    }

    pub fn set_stage(&mut self, s: u32) {
        if s < 1 || s > self.max_stage {
            return;
        }
        let old_zone = zone_index(self.stage);
        self.stage = s;
        self.kills = 0;
        self.spawn();
        self.events.push(Event::StageChanged);
        if zone_index(s) != old_zone {
            self.events
                .push(Event::Message(format!("Новая локация: {}", zone_name(s))));
        }
    }

    // ---------------------------------------------------------------- бой

    pub fn click(&mut self) {
        self.total_clicks += 1;
        let crit = self.rand() < self.crit_chance();
        let damage = self.click_damage() * if crit { CRIT_MULT } else { 1.0 };
        self.events.push(Event::Hit {
            damage,
            crit,
            click: true,
        });
        self.damage(damage);
    }

    fn damage(&mut self, d: f64) {
        if self.monster.hp <= 0.0 {
            return;
        }
        self.monster.hp -= d;
        if self.monster.hp <= 0.0 {
            self.kill_monster();
        }
    }

    pub fn update(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        self.play_time += dt;
        for s in self.skills.iter_mut() {
            s.active_left = (s.active_left - dt).max(0.0);
            s.cooldown_left = (s.cooldown_left - dt).max(0.0);
        }
        if self.skills[SKILL_STORM].active() {
            self.storm_acc += dt * 10.0;
            while self.storm_acc >= 1.0 {
                self.storm_acc -= 1.0;
                self.click();
            }
        } else {
            self.storm_acc = 0.0;
        }

        let d = self.dps() * dt;
        if d > 0.0 {
            self.damage(d);
        }

        if self.monster.boss {
            self.boss_time_left -= dt;
            if self.boss_time_left <= 0.0 && self.monster.hp > 0.0 {
                self.boss_failed();
            }
        }
    }

    fn kill_monster(&mut self) {
        let gold = self.monster.gold * self.gold_mult(true);
        self.earn(gold);
        self.total_kills += 1;
        let boss = self.monster.boss;
        self.events.push(Event::Kill { gold, boss });

        if boss {
            self.bosses_killed += 1;
            self.unlock_outfit(self.stage);
            if self.stage == self.max_stage {
                self.advance_max();
            }
        } else {
            self.kills += 1;
            if self.kills >= MONSTERS_PER_STAGE && self.stage == self.max_stage {
                self.advance_max();
            }
        }

        let done = boss || self.kills >= MONSTERS_PER_STAGE;
        if self.auto_advance && done && self.stage < self.max_stage {
            self.set_stage(self.stage + 1);
        } else {
            self.spawn();
        }
    }

    fn advance_max(&mut self) {
        self.max_stage += 1;
        if self.max_stage > self.best_stage {
            self.best_stage = self.max_stage;
            for s in SKILLS.iter() {
                if s.unlock_stage == self.max_stage {
                    self.events
                        .push(Event::Message(format!("Новый навык: «{}»", s.name)));
                }
            }
        }
        if self.max_stage == REBIRTH_MIN_STAGE && self.rebirths == 0 {
            self.events
                .push(Event::Message("Открыт Храм перерождения!".into()));
        }
    }

    fn unlock_outfit(&mut self, s: u32) {
        for (i, o) in OUTFITS.iter().enumerate() {
            if o.unlock_stage == s && !self.outfits_owned[i] {
                self.outfits_owned[i] = true;
                self.events.push(Event::OutfitUnlocked(i));
                self.events
                    .push(Event::Message(format!("Новый наряд: «{}»!", o.name)));
            }
        }
    }

    fn boss_failed(&mut self) {
        let name = MONSTERS[self.monster.def].name;
        self.events.push(Event::BossFailed);
        self.events.push(Event::Message(format!(
            "{} слишком силён! Усильте отряд",
            name
        )));
        self.auto_advance = false;
        let back = self.stage.saturating_sub(1).max(1);
        self.set_stage(back);
    }

    // ---------------------------------------------------------------- навыки

    pub fn skill_unlocked(&self, i: usize) -> bool {
        self.best_stage >= SKILLS[i].unlock_stage
    }

    pub fn use_skill(&mut self, i: usize) -> bool {
        if !self.skill_unlocked(i) || !self.skills[i].ready() {
            return false;
        }
        self.skills[i].active_left = SKILLS[i].duration;
        self.skills[i].cooldown_left = SKILLS[i].cooldown;
        self.events.push(Event::SkillUsed(i));
        if i == SKILL_STRIKE {
            let damage = self.dps() * 60.0 + self.click_damage() * 10.0;
            self.events.push(Event::Hit {
                damage,
                crit: true,
                click: false,
            });
            self.damage(damage);
        }
        true
    }

    // ---------------------------------------------------------------- героини

    fn price(&self, i: usize) -> f64 {
        let lvl = self.heroes[i].level as i32 - if HEROINES[i].clicker { 1 } else { 0 };
        HEROINES[i].base_cost * 1.07f64.powi(lvl)
    }

    /// Цена покупки `n` уровней подряд (каждый следующий на 7% дороже).
    pub fn hero_cost(&self, i: usize, n: u32) -> f64 {
        let mut price = self.price(i);
        let mut sum = 0.0;
        for _ in 0..n {
            sum += price;
            price *= 1.07;
        }
        sum.floor()
    }

    pub fn max_affordable(&self, i: usize) -> u32 {
        let mut price = self.price(i);
        let mut sum = 0.0;
        let mut n = 0;
        while n < 10_000 && sum + price <= self.gold {
            sum += price;
            price *= 1.07;
            n += 1;
        }
        n
    }

    pub fn amount_for(&self, i: usize) -> u32 {
        if self.buy_amount > 0 {
            self.buy_amount
        } else {
            self.max_affordable(i).max(1)
        }
    }

    pub fn buy(&mut self, i: usize) -> bool {
        let n = self.amount_for(i);
        let cost = self.hero_cost(i, n);
        if self.gold < cost {
            return false;
        }
        self.gold -= cost;
        self.heroes[i].level += n;
        self.changes += 1;
        true
    }

    pub fn next_milestone(&self, i: usize) -> Option<u32> {
        MILESTONES.get(self.heroes[i].upgrades as usize).copied()
    }

    pub fn can_upgrade(&self, i: usize) -> bool {
        matches!(self.next_milestone(i), Some(m) if self.heroes[i].level >= m)
    }

    pub fn upgrade_cost(&self, i: usize) -> f64 {
        let k = (self.heroes[i].upgrades as usize).min(MILESTONE_COST.len() - 1);
        (HEROINES[i].base_cost * MILESTONE_COST[k] * if HEROINES[i].clicker { 4.0 } else { 1.0 })
            .floor()
    }

    pub fn buy_upgrade(&mut self, i: usize) -> bool {
        if !self.can_upgrade(i) {
            return false;
        }
        let cost = self.upgrade_cost(i);
        if self.gold < cost {
            return false;
        }
        self.gold -= cost;
        self.heroes[i].upgrades += 1;
        self.changes += 1;
        true
    }

    /// Героиня видна в списке, если нанята она или предыдущая.
    pub fn hero_visible(&self, i: usize) -> bool {
        i <= 1 || self.heroes[i].level > 0 || self.heroes[i - 1].level > 0
    }

    pub fn party_size(&self) -> usize {
        (1..HEROINES.len())
            .filter(|&i| self.heroes[i].level > 0)
            .count()
    }

    // ---------------------------------------------------------------- наряды

    pub fn wear(&mut self, i: usize) -> bool {
        if !self.outfits_owned[i] {
            return false;
        }
        self.outfit = i;
        self.changes += 1;
        if self.monster.boss {
            self.boss_time_left = self.boss_time_left.min(self.boss_time());
        }
        true
    }

    // ---------------------------------------------------------------- перерождение

    pub fn pending_souls(&self) -> u64 {
        if self.max_stage < REBIRTH_MIN_STAGE {
            0
        } else {
            (self.max_stage as f64 / 10.0).powi(2).floor() as u64
        }
    }

    pub fn rebirth(&mut self) -> u64 {
        let gained = self.pending_souls();
        if gained == 0 {
            return 0;
        }
        self.souls += gained;
        self.rebirths += 1;
        self.changes += 1;
        self.reset_run();
        gained
    }

    fn reset_run(&mut self) {
        self.gold = 0.0;
        self.max_stage = 1;
        self.auto_advance = true;
        for (i, h) in self.heroes.iter_mut().enumerate() {
            h.level = if HEROINES[i].clicker { 1 } else { 0 };
            h.upgrades = 0;
        }
        self.skills = [SkillState::default(); 4];
        self.set_stage(1);
    }

    pub fn talisman_cost(&self, i: usize) -> u64 {
        TALISMANS[i].base_cost * (self.talismans[i] as u64 + 1)
    }

    pub fn buy_talisman(&mut self, i: usize) -> bool {
        if self.talismans[i] >= TALISMANS[i].max_level
            || self.available_souls() < self.talisman_cost(i)
        {
            return false;
        }
        self.souls_spent += self.talisman_cost(i);
        self.talismans[i] += 1;
        self.changes += 1;
        true
    }

    /// Золото, добытое отрядом, пока игра была закрыта.
    pub fn offline_gold(&self, seconds: f64) -> f64 {
        let dps = self.dps();
        if dps <= 0.0 || seconds <= 0.0 {
            return 0.0;
        }
        let s = if Self::is_boss_stage(self.stage) {
            self.stage.saturating_sub(1).max(1)
        } else {
            self.stage
        };
        let kills_per_sec = (dps / Self::monster_hp(s)).min(5.0);
        kills_per_sec
            * seconds.min(OFFLINE_LIMIT_SEC)
            * Self::monster_gold(s)
            * self.gold_mult(false)
    }

    pub fn earn(&mut self, g: f64) {
        self.gold += g;
        self.total_gold += g;
    }

    /// Полный сброс игры.
    pub fn reset_all(&mut self) {
        let buy_amount = self.buy_amount;
        let changes = self.changes;
        *self = Game::new();
        self.buy_amount = buy_amount;
        self.changes = changes + 1;
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices() {
        let g = Game::new();
        assert_eq!(g.hero_cost(0, 1), 5.0);
        assert_eq!(g.hero_cost(1, 1), 50.0);
        // 50 + 53.5 = 103.5
        assert_eq!(g.hero_cost(1, 2), 103.0);
    }

    #[test]
    fn kill_and_advance() {
        let mut g = Game::new();
        for _ in 0..200 {
            g.click();
            if g.stage > 1 {
                break;
            }
        }
        assert_eq!(g.stage, 2);
        assert_eq!(g.max_stage, 2);
        assert!(g.gold > 0.0);
    }

    #[test]
    fn boss_fail_goes_back() {
        let mut g = Game::new();
        g.max_stage = 5;
        g.set_stage(5);
        assert!(g.monster.boss);
        g.update(g.boss_time() + 1.0);
        assert_eq!(g.stage, 4);
        assert!(!g.auto_advance);
    }

    #[test]
    fn boss_drops_outfit() {
        let mut g = Game::new();
        g.max_stage = 5;
        g.set_stage(5);
        g.heroes[0].level = 1_000_000;
        g.click();
        assert!(g.outfits_owned[1]);
        assert_eq!(g.max_stage, 6);
        assert!(g.wear(1));
    }

    #[test]
    fn rebirth_gives_souls() {
        let mut g = Game::new();
        assert_eq!(g.rebirth(), 0);
        g.max_stage = 40;
        g.heroes[3].level = 10;
        assert_eq!(g.pending_souls(), 16);
        assert_eq!(g.rebirth(), 16);
        assert_eq!(g.stage, 1);
        assert_eq!(g.heroes[3].level, 0);
        assert_eq!(g.heroes[0].level, 1);
        assert!(g.buy_talisman(TAL_BLADE));
        assert_eq!(g.available_souls(), 15);
        assert!(g.soul_mult() > 2.5);
    }

    #[test]
    fn bulk_and_max_buy() {
        let mut g = Game::new();
        g.gold = 1000.0;
        g.buy_amount = 0;
        let n = g.max_affordable(1);
        assert!(n >= 10);
        assert!(g.buy(1));
        assert_eq!(g.heroes[1].level, n);
        assert!(g.gold < g.hero_cost(1, 1));
        assert!(g.can_upgrade(1));
    }
}
