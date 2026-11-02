package sakura;

import java.util.List;

/** Активный навык с длительностью и перезарядкой. */
public final class Skill {
    public final String id, name, description;
    public final int key, unlockStage;
    public final double duration, cooldown;
    public double activeLeft, cooldownLeft;

    private Skill(String id, String name, String description, int key, int unlockStage, double duration, double cooldown) {
        this.id = id;
        this.name = name;
        this.description = description;
        this.key = key;
        this.unlockStage = unlockStage;
        this.duration = duration;
        this.cooldown = cooldown;
    }

    public boolean active() {
        return activeLeft > 0;
    }

    public boolean ready() {
        return cooldownLeft <= 0;
    }

    void activate() {
        activeLeft = duration;
        cooldownLeft = cooldown;
    }

    void update(double dt) {
        if (activeLeft > 0) activeLeft = Math.max(0, activeLeft - dt);
        if (cooldownLeft > 0) cooldownLeft = Math.max(0, cooldownLeft - dt);
    }

    void reset() {
        activeLeft = 0;
        cooldownLeft = 0;
    }

    public static List<Skill> createAll() {
        return List.of(
            new Skill("storm", "Буря лепестков", "10 ударов в секунду на 30 с", 1, 3, 30, 300),
            new Skill("blessing", "Благословение", "Урон героинь ×2 на 30 с", 2, 10, 30, 600),
            new Skill("fan", "Золотой веер", "Золото ×2 на 30 с", 3, 20, 30, 600),
            new Skill("strike", "Небесный удар", "Мгновенно: 60 с урона героинь", 4, 35, 0, 900)
        );
    }
}
