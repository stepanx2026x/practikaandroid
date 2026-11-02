package sakura;

/** Конкретный ёкай на поле боя. */
public final class Monster {
    public final MonsterType type;
    public final double maxHp, gold;
    public final boolean boss;
    public double hp;

    public Monster(MonsterType type, double maxHp, double gold, boolean boss) {
        this.type = type;
        this.maxHp = maxHp;
        this.hp = maxHp;
        this.gold = gold;
        this.boss = boss;
    }
}
