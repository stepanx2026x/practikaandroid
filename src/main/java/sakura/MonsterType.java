package sakura;

import java.awt.Color;

/** Вид ёкая и параметры его рисования. */
public final class MonsterType {
    public enum Shape { BLOB, GHOST, LANTERN, UMBRELLA, CAT, SPIDER, WALL, SERPENT, FIREBALL, WHEEL }

    public final String name;
    public final Shape shape;
    public final Color body, accent;
    public final int eyes, horns;
    public final boolean longNose;

    public MonsterType(String name, Shape shape, Color body, Color accent, int eyes, int horns, boolean longNose) {
        this.name = name;
        this.shape = shape;
        this.body = body;
        this.accent = accent;
        this.eyes = eyes;
        this.horns = horns;
        this.longNose = longNose;
    }
}
