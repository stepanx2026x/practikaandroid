package sakura;

import java.util.List;

/** Талисман — вечное улучшение, покупается за души сакуры и не сбрасывается при перерождении. */
public final class Talisman {
    public final String id, name, description;
    public final long baseCost;
    public final int maxLevel;
    public int level;

    private Talisman(String id, String name, String description, long baseCost, int maxLevel) {
        this.id = id;
        this.name = name;
        this.description = description;
        this.baseCost = baseCost;
        this.maxLevel = maxLevel;
    }

    public long cost() {
        return baseCost * (level + 1);
    }

    public boolean maxed() {
        return level >= maxLevel;
    }

    public static List<Talisman> createAll() {
        return List.of(
            new Talisman("blade", "Талисман клинка", "+25% урона клика за уровень", 1, 50),
            new Talisman("spirit", "Талисман духа", "+15% урона героинь за уровень", 1, 50),
            new Talisman("wealth", "Манэки-нэко", "+10% золота за уровень", 1, 50),
            new Talisman("luck", "Омамори удачи", "+2% шанс крита за уровень", 2, 10),
            new Talisman("hourglass", "Песочные часы", "+2 с на битву с боссом за уровень", 2, 10)
        );
    }
}
