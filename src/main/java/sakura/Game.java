package sakura;

import java.util.List;
import java.util.Random;

/** Вся логика игры: бой, золото, героини, наряды, навыки, перерождение. */
public final class Game {

    /** События для отрисовки эффектов и сообщений. */
    public interface Listener {
        void onHit(double damage, boolean crit, boolean fromClick);

        void onKill(double gold, boolean boss);

        void onMessage(String text);
    }

    public static final int MONSTERS_PER_STAGE = 10;
    public static final int BOSS_EVERY = 5;
    public static final int REBIRTH_MIN_STAGE = 30;
    public static final double CRIT_MULT = 5;
    public static final double OFFLINE_LIMIT_SEC = 8 * 3600;

    public final List<Heroine> heroines = Heroine.createAll();
    public final Heroine sakura = heroines.get(0);
    public final List<Outfit> outfits = Outfit.createAll();
    public final List<Skill> skills = Skill.createAll();
    public final List<Talisman> talismans = Talisman.createAll();

    public double gold;
    public double totalGold;
    public int stage = 1;
    /** Самый дальний открытый этап в текущем перерождении. */
    public int maxStage = 1;
    /** Рекорд за всё время. */
    public int bestStage = 1;
    /** Побеждено ёкаев на текущем этапе. */
    public int kills;
    public boolean autoAdvance = true;
    public Monster monster;
    public double bossTimeLeft;

    public long souls, soulsSpent;
    public int rebirths;
    public long totalClicks, totalKills, bossesKilled;
    public double playTime;
    public String outfitId = "casual";
    /** Сколько уровней покупать за раз (0 — максимум). */
    public int buyAmount = 1;

    public Listener listener;

    private final Random rnd = new Random();
    private double stormAcc;

    public Game() {
        spawn();
    }

    // ---------- Множители ----------

    public Outfit outfit() {
        for (Outfit o : outfits) if (o.id.equals(outfitId)) return o;
        return outfits.get(0);
    }

    public Skill skill(String id) {
        for (Skill s : skills) if (s.id.equals(id)) return s;
        throw new IllegalArgumentException(id);
    }

    public int talismanLevel(String id) {
        for (Talisman t : talismans) if (t.id.equals(id)) return t.level;
        return 0;
    }

    public long availableSouls() {
        return souls - soulsSpent;
    }

    /** Каждая душа сакуры: +10% ко всему урону. */
    public double soulMult() {
        return 1 + 0.10 * souls;
    }

    /** Каждый собранный наряд: +5% ко всему урону. */
    public double collectionMult() {
        int n = 0;
        for (Outfit o : outfits) if (o.owned && o.unlockStage > 0) n++;
        return 1 + 0.05 * n;
    }

    public double dps() {
        double sum = 0;
        for (Heroine h : heroines) if (!h.clicker) sum += h.rawDps();
        double m = soulMult() * collectionMult() * outfit().dpsMult * (1 + 0.15 * talismanLevel("spirit"));
        if (skill("blessing").active()) m *= 2;
        return sum * m;
    }

    /** Урон клика. Каждый «Свиток силы» Сакуры также добавляет 1% от урона героинь. */
    public double clickDamage() {
        double base = sakura.level * sakura.multiplier();
        double m = soulMult() * collectionMult() * outfit().clickMult * (1 + 0.25 * talismanLevel("blade"));
        return base * m + dps() * 0.01 * sakura.upgrades;
    }

    public double goldMult(boolean withSkills) {
        double m = outfit().goldMult * (1 + 0.10 * talismanLevel("wealth"));
        if (withSkills && skill("fan").active()) m *= 2;
        return m;
    }

    public double critChance() {
        return Math.min(0.5, 0.03 + outfit().critAdd + 0.02 * talismanLevel("luck"));
    }

    public double bossTime() {
        return 30 + outfit().timerAdd + 2 * talismanLevel("hourglass");
    }

    // ---------- Ёкаи ----------

    public static boolean isBossStage(int s) {
        return s % BOSS_EVERY == 0;
    }

    public static double monsterHp(int s) {
        return Math.ceil(10 * Math.pow(1.39, s - 1) + 5 * (s - 1));
    }

    public static double monsterGold(int s) {
        return Math.max(1, Math.ceil(monsterHp(s) / 12));
    }

    private void spawn() {
        boolean boss = isBossStage(stage);
        Zone z = Zone.forStage(stage);
        MonsterType t = boss ? z.boss : z.pool[rnd.nextInt(z.pool.length)];
        double hp = monsterHp(stage) * (boss ? 8 : 1);
        double g = monsterGold(stage) * (boss ? 5 : 1);
        monster = new Monster(t, hp, g, boss);
        bossTimeLeft = boss ? bossTime() : 0;
    }

    public boolean stageCleared(int s) {
        return s < maxStage;
    }

    public void setStage(int s) {
        if (s < 1 || s > maxStage) return;
        stage = s;
        kills = 0;
        spawn();
    }

    // ---------- Бой ----------

    public void click() {
        totalClicks++;
        boolean crit = rnd.nextDouble() < critChance();
        double dmg = clickDamage() * (crit ? CRIT_MULT : 1);
        if (listener != null) listener.onHit(dmg, crit, true);
        damage(dmg);
    }

    private void damage(double d) {
        if (monster.hp <= 0) return;
        monster.hp -= d;
        if (monster.hp <= 0) killMonster();
    }

    public void update(double dt) {
        if (dt <= 0) return;
        playTime += dt;
        for (Skill s : skills) s.update(dt);

        if (skill("storm").active()) {
            stormAcc += dt * 10;
            while (stormAcc >= 1) {
                stormAcc -= 1;
                click();
            }
        } else {
            stormAcc = 0;
        }

        double d = dps() * dt;
        if (d > 0) damage(d);

        if (monster.boss) {
            bossTimeLeft -= dt;
            if (bossTimeLeft <= 0 && monster.hp > 0) bossFailed();
        }
    }

    private void killMonster() {
        double g = monster.gold * goldMult(true);
        gold += g;
        totalGold += g;
        totalKills++;
        boolean boss = monster.boss;
        if (listener != null) listener.onKill(g, boss);

        if (boss) {
            bossesKilled++;
            unlockOutfit(stage);
            if (stage == maxStage) advanceMax();
        } else {
            kills++;
            if (kills >= MONSTERS_PER_STAGE && stage == maxStage) advanceMax();
        }

        boolean done = boss || kills >= MONSTERS_PER_STAGE;
        if (autoAdvance && done && stage < maxStage) {
            setStage(stage + 1);
        } else {
            spawn();
        }
    }

    private void advanceMax() {
        maxStage++;
        if (maxStage > bestStage) bestStage = maxStage;
        for (Skill s : skills) {
            if (s.unlockStage == maxStage && bestStage == maxStage) {
                message("Новый навык: «" + s.name + "» — клавиша " + s.key);
            }
        }
        if (maxStage == REBIRTH_MIN_STAGE && rebirths == 0) {
            message("Открыто перерождение! Загляните во вкладку «Перерождение»");
        }
    }

    private void unlockOutfit(int s) {
        for (Outfit o : outfits) {
            if (o.unlockStage == s && !o.owned) {
                o.owned = true;
                message("Новый наряд: «" + o.name + "»! Наденьте его во вкладке «Наряды»");
            }
        }
    }

    private void bossFailed() {
        message(monster.type.name + " оказался слишком силён! Прокачайте героинь и попробуйте снова");
        autoAdvance = false;
        setStage(Math.max(1, stage - 1));
    }

    private void message(String text) {
        if (listener != null) listener.onMessage(text);
    }

    // ---------- Навыки ----------

    public boolean isUnlocked(Skill s) {
        return bestStage >= s.unlockStage;
    }

    public boolean useSkill(Skill s) {
        if (!isUnlocked(s) || !s.ready()) return false;
        s.activate();
        if (s.id.equals("strike")) {
            double dmg = dps() * 60 + clickDamage() * 10;
            if (listener != null) listener.onHit(dmg, true, false);
            damage(dmg);
        }
        return true;
    }

    // ---------- Покупки ----------

    public int amountFor(Heroine h) {
        if (buyAmount > 0) return buyAmount;
        return Math.max(1, h.maxAffordable(gold));
    }

    public double costFor(Heroine h) {
        return h.costFor(amountFor(h));
    }

    public boolean buy(Heroine h) {
        int n = amountFor(h);
        double c = h.costFor(n);
        if (gold < c) return false;
        gold -= c;
        h.level += n;
        return true;
    }

    public boolean buyUpgrade(Heroine h) {
        if (!h.canUpgrade()) return false;
        double c = h.upgradeCost();
        if (gold < c) return false;
        gold -= c;
        h.upgrades++;
        return true;
    }

    public boolean wear(Outfit o) {
        if (!o.owned) return false;
        outfitId = o.id;
        if (monster.boss) bossTimeLeft = Math.min(bossTimeLeft, bossTime());
        return true;
    }

    /** Героиня видна в списке, если нанята она или предыдущая. */
    public boolean isVisible(Heroine h) {
        int i = heroines.indexOf(h);
        return i <= 1 || h.level > 0 || heroines.get(i - 1).level > 0;
    }

    // ---------- Перерождение ----------

    public long pendingSouls() {
        if (maxStage < REBIRTH_MIN_STAGE) return 0;
        return (long) Math.floor(Math.pow(maxStage / 10.0, 2));
    }

    public long rebirth() {
        long gained = pendingSouls();
        if (gained <= 0) return 0;
        souls += gained;
        rebirths++;
        resetRun();
        return gained;
    }

    private void resetRun() {
        gold = 0;
        maxStage = 1;
        autoAdvance = true;
        for (Heroine h : heroines) {
            h.level = h.clicker ? 1 : 0;
            h.upgrades = 0;
        }
        for (Skill s : skills) s.reset();
        setStage(1);
    }

    public boolean buyTalisman(Talisman t) {
        if (t.maxed() || availableSouls() < t.cost()) return false;
        soulsSpent += t.cost();
        t.level++;
        return true;
    }

    /** Золото, заработанное героинями, пока игра была закрыта. */
    public double offlineGold(double seconds) {
        double dps = dps();
        if (dps <= 0 || seconds <= 0) return 0;
        int s = isBossStage(stage) ? Math.max(1, stage - 1) : stage;
        double killsPerSec = Math.min(dps / monsterHp(s), 5);
        return killsPerSec * Math.min(seconds, OFFLINE_LIMIT_SEC) * monsterGold(s) * goldMult(false);
    }

    public void earn(double g) {
        gold += g;
        totalGold += g;
    }

    /** Полный сброс игры. */
    public void resetAll() {
        souls = soulsSpent = 0;
        rebirths = 0;
        totalClicks = totalKills = bossesKilled = 0;
        playTime = 0;
        totalGold = 0;
        bestStage = 1;
        outfitId = "casual";
        for (Outfit o : outfits) o.owned = o.unlockStage == 0;
        for (Talisman t : talismans) t.level = 0;
        resetRun();
    }
}
