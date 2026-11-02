package sakura;

import java.io.IOException;
import java.io.Reader;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Properties;

/** Сохранение в ~/.sakura-yokai/save.properties. */
public final class Save {
    private Save() {}

    public static Path file() {
        return Path.of(System.getProperty("user.home"), ".sakura-yokai", "save.properties");
    }

    public static void save(Game g) {
        Properties p = new Properties();
        p.setProperty("gold", String.valueOf(g.gold));
        p.setProperty("totalGold", String.valueOf(g.totalGold));
        p.setProperty("stage", String.valueOf(g.stage));
        p.setProperty("maxStage", String.valueOf(g.maxStage));
        p.setProperty("bestStage", String.valueOf(g.bestStage));
        p.setProperty("autoAdvance", String.valueOf(g.autoAdvance));
        p.setProperty("souls", String.valueOf(g.souls));
        p.setProperty("soulsSpent", String.valueOf(g.soulsSpent));
        p.setProperty("rebirths", String.valueOf(g.rebirths));
        p.setProperty("totalClicks", String.valueOf(g.totalClicks));
        p.setProperty("totalKills", String.valueOf(g.totalKills));
        p.setProperty("bossesKilled", String.valueOf(g.bossesKilled));
        p.setProperty("playTime", String.valueOf(g.playTime));
        p.setProperty("outfit", g.outfitId);
        p.setProperty("buyAmount", String.valueOf(g.buyAmount));
        p.setProperty("savedAt", String.valueOf(System.currentTimeMillis()));
        for (Heroine h : g.heroines) {
            p.setProperty("hero." + h.id + ".level", String.valueOf(h.level));
            p.setProperty("hero." + h.id + ".upgrades", String.valueOf(h.upgrades));
        }
        for (Outfit o : g.outfits) p.setProperty("outfit." + o.id, String.valueOf(o.owned));
        for (Talisman t : g.talismans) p.setProperty("talisman." + t.id, String.valueOf(t.level));
        for (Skill s : g.skills) p.setProperty("skill." + s.id + ".cooldown", String.valueOf(s.cooldownLeft));
        try {
            Path f = file();
            Files.createDirectories(f.getParent());
            Path tmp = f.resolveSibling("save.tmp");
            try (Writer w = Files.newBufferedWriter(tmp, StandardCharsets.UTF_8)) {
                p.store(w, "Sakura & Yokai save");
            }
            Files.move(tmp, f, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        } catch (IOException e) {
            System.err.println("Не удалось сохранить игру: " + e.getMessage());
        }
    }

    /** Загружает сохранение. Возвращает время (мс) с момента сохранения или -1, если сохранения нет. */
    public static long load(Game g) {
        Path f = file();
        if (!Files.exists(f)) return -1;
        Properties p = new Properties();
        try (Reader r = Files.newBufferedReader(f, StandardCharsets.UTF_8)) {
            p.load(r);
        } catch (IOException e) {
            System.err.println("Не удалось загрузить игру: " + e.getMessage());
            return -1;
        }
        g.gold = d(p, "gold", 0);
        g.totalGold = d(p, "totalGold", 0);
        g.maxStage = Math.max(1, i(p, "maxStage", 1));
        g.bestStage = Math.max(g.maxStage, i(p, "bestStage", 1));
        g.autoAdvance = Boolean.parseBoolean(p.getProperty("autoAdvance", "true"));
        g.souls = l(p, "souls", 0);
        g.soulsSpent = l(p, "soulsSpent", 0);
        g.rebirths = i(p, "rebirths", 0);
        g.totalClicks = l(p, "totalClicks", 0);
        g.totalKills = l(p, "totalKills", 0);
        g.bossesKilled = l(p, "bossesKilled", 0);
        g.playTime = d(p, "playTime", 0);
        g.buyAmount = i(p, "buyAmount", 1);
        for (Heroine h : g.heroines) {
            h.level = Math.max(h.clicker ? 1 : 0, i(p, "hero." + h.id + ".level", h.level));
            h.upgrades = Math.min(Heroine.MILESTONES.length, i(p, "hero." + h.id + ".upgrades", 0));
        }
        for (Outfit o : g.outfits) o.owned = o.unlockStage == 0 || Boolean.parseBoolean(p.getProperty("outfit." + o.id, "false"));
        for (Talisman t : g.talismans) t.level = Math.min(t.maxLevel, i(p, "talisman." + t.id, 0));
        for (Skill s : g.skills) s.cooldownLeft = d(p, "skill." + s.id + ".cooldown", 0);
        String outfit = p.getProperty("outfit", "casual");
        for (Outfit o : g.outfits) if (o.id.equals(outfit) && o.owned) g.outfitId = outfit;
        g.setStage(Math.min(g.maxStage, Math.max(1, i(p, "stage", 1))));
        return Math.max(0, System.currentTimeMillis() - l(p, "savedAt", System.currentTimeMillis()));
    }

    private static double d(Properties p, String k, double def) {
        try {
            return Double.parseDouble(p.getProperty(k, String.valueOf(def)));
        } catch (NumberFormatException e) {
            return def;
        }
    }

    private static int i(Properties p, String k, int def) {
        try {
            return Integer.parseInt(p.getProperty(k, String.valueOf(def)));
        } catch (NumberFormatException e) {
            return def;
        }
    }

    private static long l(Properties p, String k, long def) {
        try {
            return Long.parseLong(p.getProperty(k, String.valueOf(def)));
        } catch (NumberFormatException e) {
            return def;
        }
    }
}
