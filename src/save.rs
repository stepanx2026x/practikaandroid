//! Сохранение в простой текстовый файл «ключ=значение».
//!
//! Windows: %APPDATA%\SakuraYokai\save.txt
//! macOS:   ~/Library/Application Support/SakuraYokai/save.txt
//! Linux:   ~/.local/share/sakura-yokai/save.txt

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::*;
use crate::game::Game;

pub fn path() -> PathBuf {
    if let Ok(dir) = std::env::var("SAKURA_SAVE_DIR") {
        return PathBuf::from(dir).join("save.txt");
    }
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("SakuraYokai"))
            .ok()
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/SakuraYokai"))
            .ok()
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .map(|p| p.join("sakura-yokai"))
            .ok()
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("save.txt")
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn save(g: &Game) {
    let mut out = String::from("# Сакура и ёкаи — сохранение\n");
    let mut put = |k: &str, v: String| {
        out.push_str(k);
        out.push('=');
        out.push_str(&v);
        out.push('\n');
    };
    put("version", "2".into());
    put("saved_at", now_secs().to_string());
    put("gold", g.gold.to_string());
    put("total_gold", g.total_gold.to_string());
    put("stage", g.stage.to_string());
    put("max_stage", g.max_stage.to_string());
    put("best_stage", g.best_stage.to_string());
    put("auto_advance", g.auto_advance.to_string());
    put("souls", g.souls.to_string());
    put("souls_spent", g.souls_spent.to_string());
    put("rebirths", g.rebirths.to_string());
    put("total_clicks", g.total_clicks.to_string());
    put("total_kills", g.total_kills.to_string());
    put("bosses_killed", g.bosses_killed.to_string());
    put("play_time", g.play_time.to_string());
    put("buy_amount", g.buy_amount.to_string());
    put("outfit", OUTFITS[g.outfit].id.into());
    for (i, h) in HEROINES.iter().enumerate() {
        put(
            &format!("hero.{}.level", h.id),
            g.heroes[i].level.to_string(),
        );
        put(
            &format!("hero.{}.upgrades", h.id),
            g.heroes[i].upgrades.to_string(),
        );
    }
    for (i, o) in OUTFITS.iter().enumerate() {
        put(&format!("outfit.{}", o.id), g.outfits_owned[i].to_string());
    }
    for (i, t) in TALISMANS.iter().enumerate() {
        put(&format!("talisman.{}", t.id), g.talismans[i].to_string());
    }
    for (i, s) in SKILLS.iter().enumerate() {
        put(
            &format!("skill.{}.cooldown", s.id),
            g.skills[i].cooldown_left.to_string(),
        );
    }

    let file = path();
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let tmp = file.with_extension("tmp");
    if fs::write(&tmp, out).is_ok() {
        let _ = fs::rename(&tmp, &file);
    }
}

/// Загружает сохранение. Возвращает, сколько секунд прошло с момента сохранения
/// (`None`, если сохранения нет).
pub fn load(g: &mut Game) -> Option<f64> {
    let text = fs::read_to_string(path()).ok()?;
    let map: HashMap<&str, &str> = text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect();
    let f = |k: &str, d: f64| {
        map.get(k)
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(d)
    };
    let u = |k: &str, d: u64| map.get(k).and_then(|v| v.parse::<u64>().ok()).unwrap_or(d);
    let b = |k: &str, d: bool| map.get(k).and_then(|v| v.parse::<bool>().ok()).unwrap_or(d);

    g.gold = f("gold", 0.0).max(0.0);
    g.total_gold = f("total_gold", 0.0);
    g.max_stage = u("max_stage", 1).max(1) as u32;
    g.best_stage = (u("best_stage", 1) as u32).max(g.max_stage);
    g.auto_advance = b("auto_advance", true);
    g.souls = u("souls", 0);
    g.souls_spent = u("souls_spent", 0).min(g.souls);
    g.rebirths = u("rebirths", 0) as u32;
    g.total_clicks = u("total_clicks", 0);
    g.total_kills = u("total_kills", 0);
    g.bosses_killed = u("bosses_killed", 0);
    g.play_time = f("play_time", 0.0);
    g.buy_amount = match u("buy_amount", 1) as u32 {
        a @ (0 | 1 | 10 | 25 | 100) => a,
        _ => 1,
    };
    for (i, h) in HEROINES.iter().enumerate() {
        let min = if h.clicker { 1 } else { 0 };
        g.heroes[i].level = (u(&format!("hero.{}.level", h.id), min as u64) as u32).max(min);
        g.heroes[i].upgrades = (u(&format!("hero.{}.upgrades", h.id), 0) as u32)
            .min(crate::game::MILESTONES.len() as u32);
    }
    for (i, o) in OUTFITS.iter().enumerate() {
        g.outfits_owned[i] = o.unlock_stage == 0 || b(&format!("outfit.{}", o.id), false);
    }
    for (i, t) in TALISMANS.iter().enumerate() {
        g.talismans[i] = (u(&format!("talisman.{}", t.id), 0) as u32).min(t.max_level);
    }
    for (i, s) in SKILLS.iter().enumerate() {
        g.skills[i].cooldown_left =
            f(&format!("skill.{}.cooldown", s.id), 0.0).clamp(0.0, s.cooldown);
    }
    if let Some(id) = map.get("outfit") {
        if let Some(i) = OUTFITS.iter().position(|o| o.id == *id) {
            if g.outfits_owned[i] {
                g.outfit = i;
            }
        }
    }
    let stage = (u("stage", 1) as u32).clamp(1, g.max_stage);
    g.set_stage(stage);
    g.take_events();

    let saved_at = u("saved_at", now_secs());
    Some(now_secs().saturating_sub(saved_at) as f64)
}
