package sakura;

import java.util.ArrayList;
import java.util.List;

import static sakura.Look.c;

/** Героиня отряда. Сакура (clicker) усиливает клик, остальные наносят урон в секунду. */
public final class Heroine {
    /** Уровни, на которых открывается «Свиток силы» (×2 к урону). */
    public static final int[] MILESTONES = {10, 25, 50, 75, 100, 150, 200, 300};
    private static final double[] MILESTONE_COST = {5, 25, 100, 500, 2_500, 15_000, 100_000, 1_000_000};

    public final String id, name, title;
    public final boolean clicker;
    public final double baseCost, baseDps;
    public final Look look;

    public int level;
    public int upgrades;

    public Heroine(String id, String name, String title, boolean clicker, double baseCost, double baseDps, Look look) {
        this.id = id;
        this.name = name;
        this.title = title;
        this.clicker = clicker;
        this.baseCost = baseCost;
        this.baseDps = baseDps;
        this.look = look;
        this.level = clicker ? 1 : 0;
    }

    /** Цена покупки [n] уровней подряд (каждый следующий на 7% дороже). */
    public double costFor(int n) {
        double sum = 0;
        double price = baseCost * Math.pow(1.07, clicker ? level - 1 : level);
        for (int i = 0; i < n; i++) {
            sum += price;
            price *= 1.07;
        }
        return Math.floor(sum);
    }

    /** Сколько уровней можно купить на [gold]. */
    public int maxAffordable(double gold) {
        double price = baseCost * Math.pow(1.07, clicker ? level - 1 : level);
        double sum = 0;
        int n = 0;
        while (n < 10_000 && sum + price <= gold) {
            sum += price;
            price *= 1.07;
            n++;
        }
        return n;
    }

    public double multiplier() {
        return Math.pow(2, upgrades);
    }

    /** Урон в секунду без глобальных бонусов. */
    public double rawDps() {
        return baseDps * level * multiplier();
    }

    public boolean hasMoreUpgrades() {
        return upgrades < MILESTONES.length;
    }

    public int nextMilestone() {
        return hasMoreUpgrades() ? MILESTONES[upgrades] : -1;
    }

    public boolean canUpgrade() {
        return hasMoreUpgrades() && level >= MILESTONES[upgrades];
    }

    public double upgradeCost() {
        return Math.floor(baseCost * MILESTONE_COST[upgrades] * (clicker ? 4 : 1));
    }

    public static List<Heroine> createAll() {
        List<Heroine> list = new ArrayList<>();
        Look sakura = new Look(c(0xF06292), c(0xF8BBD0), c(0xF48FB1), c(0xC2185B), Look.Weapon.SWORD);
        sakura.flower = true;
        list.add(new Heroine("sakura", "Сакура", "главная героиня • ваш клик", true, 5, 0, sakura));

        list.add(h("hana", "Хана", "юная мечница", 50, 5,
            new Look(c(0x6D4C41), c(0xA5D6A7), c(0x66BB6A), c(0xFFF176), Look.Weapon.SWORD)));
        list.add(h("rin", "Рин", "мико с талисманами", 250, 22,
            new Look(c(0x212121), c(0xFFFFFF), c(0xE53935), c(0xE53935), Look.Weapon.TALISMAN)));
        list.add(h("yuki", "Юки", "лучница горных рек", 1_000, 74,
            new Look(c(0x81D4FA), c(0xE1F5FE), c(0x4FC3F7), c(0x0277BD), Look.Weapon.BOW)));
        Look kage = new Look(c(0x37474F), c(0x455A64), c(0x263238), c(0xE91E63), Look.Weapon.KUNAI);
        kage.scarf = true;
        list.add(h("kage", "Кагэ", "тень-ниндзя", 4_000, 245, kage));
        list.add(h("ayame", "Аямэ", "мастер нагинаты", 20_000, 976,
            new Look(c(0x5E35B1), c(0xD1C4E9), c(0x9575CD), c(0xFFD54F), Look.Weapon.SPEAR)));
        Look tomoe = new Look(c(0x3E2723), c(0xC62828), c(0x424242), c(0xFFD54F), Look.Weapon.SWORD);
        tomoe.helmet = true;
        list.add(h("tomoe", "Томоэ", "странствующий самурай", 100_000, 3_725, tomoe));
        Look mia = new Look(c(0xFFA726), c(0xFFF3E0), c(0xFFCC80), c(0xEF6C00), Look.Weapon.FAN);
        mia.foxEars = true;
        list.add(h("mia", "Мия", "лисья волшебница", 400_000, 10_860, mia));
        list.add(h("akari", "Акари", "хранительница фонарей", 2.5e6, 47_150,
            new Look(c(0xFF7043), c(0xFFE0B2), c(0xFF8A65), c(0xBF360C), Look.Weapon.LANTERN)));
        list.add(h("shizuka", "Сидзука", "мастер чайной церемонии", 1.5e7, 186_000,
            new Look(c(0x2E7D32), c(0xC8E6C9), c(0x81C784), c(0x1B5E20), Look.Weapon.FAN)));
        list.add(h("ryuko", "Рюко", "укротительница драконов", 1e8, 782_000,
            new Look(c(0xB71C1C), c(0x37474F), c(0x212121), c(0xFF5252), Look.Weapon.SPEAR)));
        Look tsukiko = new Look(c(0xE0E0E0), c(0x3949AB), c(0x283593), c(0xB3E5FC), Look.Weapon.STAFF);
        tsukiko.hood = true;
        list.add(h("tsukiko", "Цукико", "лунная жрица", 8e8, 3.72e6, tsukiko));
        Look amatera = new Look(c(0xFFCA28), c(0xFFF9C4), c(0xFFE082), c(0xFF6F00), Look.Weapon.STAFF);
        amatera.halo = true;
        list.add(h("amatera", "Аматэра", "солнечная императрица", 6.5e9, 1.44e7, amatera));
        list.add(h("hotaru", "Хотару", "повелительница светлячков", 6e10, 6.5e7,
            new Look(c(0x9CCC65), c(0x1A237E), c(0x0D47A1), c(0xC6FF00), Look.Weapon.LANTERN)));
        return list;
    }

    private static Heroine h(String id, String name, String title, double cost, double dps, Look look) {
        return new Heroine(id, name, title, false, cost, dps, look);
    }
}
