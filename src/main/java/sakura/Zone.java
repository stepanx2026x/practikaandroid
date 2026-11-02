package sakura;

import java.awt.Color;
import java.util.List;

import static sakura.Look.c;

/** Локация: 10 этапов со своим небом, ёкаями и боссом. */
public final class Zone {
    public enum Particles { PETALS, SNOW, EMBERS, FIREFLIES }

    public final String name;
    public final Color skyTop, skyBottom, ground, hills;
    public final Particles particles;
    public final boolean night, torii;
    public final MonsterType[] pool;
    public final MonsterType boss;

    private Zone(String name, int skyTop, int skyBottom, int ground, int hills, Particles particles,
                 boolean night, boolean torii, MonsterType boss, MonsterType... pool) {
        this.name = name;
        this.skyTop = c(skyTop);
        this.skyBottom = c(skyBottom);
        this.ground = c(ground);
        this.hills = c(hills);
        this.particles = particles;
        this.night = night;
        this.torii = torii;
        this.boss = boss;
        this.pool = pool;
    }

    private static MonsterType m(String name, MonsterType.Shape shape, int body, int accent, int eyes, int horns) {
        return new MonsterType(name, shape, c(body), c(accent), eyes, horns, false);
    }

    private static MonsterType nose(String name, int body, int accent) {
        return new MonsterType(name, MonsterType.Shape.BLOB, c(body), c(accent), 2, 0, true);
    }

    public static final List<Zone> ZONES = List.of(
        new Zone("Сакуровая роща", 0xFFC9DE, 0xFFF4F8, 0xB9DFA0, 0xF3B6CF, Particles.PETALS, false, true,
            m("Красный они", MonsterType.Shape.BLOB, 0xE53935, 0xFFCDD2, 2, 2),
            m("Каппа", MonsterType.Shape.BLOB, 0x66BB6A, 0xFFF59D, 2, 0),
            m("Тануки", MonsterType.Shape.CAT, 0xA1887F, 0x5D4037, 2, 0),
            m("Каса-обакэ", MonsterType.Shape.UMBRELLA, 0x9575CD, 0xD1C4E9, 1, 0)),
        new Zone("Бамбуковый лес", 0xC8E6C9, 0xF1F8E9, 0x8BC34A, 0x81C784, Particles.PETALS, false, false,
            nose("Великий тэнгу", 0xD32F2F, 0x212121),
            m("Бакэнэко", MonsterType.Shape.CAT, 0x424242, 0xFFEB3B, 2, 0),
            m("Кама-итати", MonsterType.Shape.CAT, 0xD7CCC8, 0x8D6E63, 2, 0),
            nose("Карасу-тэнгу", 0x37474F, 0xFFB300)),
        new Zone("Горный храм", 0xD1C4E9, 0xF3E5F5, 0xB0BEC5, 0x9FA8DA, Particles.PETALS, false, true,
            m("Цутигумо", MonsterType.Shape.SPIDER, 0x4E342E, 0xFF5252, 4, 0),
            m("Нурикабэ", MonsterType.Shape.WALL, 0x9E9E9E, 0x757575, 2, 0),
            m("Ёкай-монах", MonsterType.Shape.GHOST, 0x8D6E63, 0xD7CCC8, 1, 0),
            m("Каменный они", MonsterType.Shape.BLOB, 0x78909C, 0xCFD8DC, 2, 1)),
        new Zone("Город фонарей", 0x1A237E, 0x5C6BC0, 0x3E2723, 0x283593, Particles.FIREFLIES, true, false,
            m("Нуэ", MonsterType.Shape.CAT, 0x6D4C41, 0xFFEB3B, 2, 1),
            m("Чётин-обакэ", MonsterType.Shape.LANTERN, 0xFF8A65, 0xBF360C, 1, 0),
            m("Ноппэрапон", MonsterType.Shape.GHOST, 0xFFE0B2, 0xFFCC80, 0, 0),
            m("Хитоцумэ-кодзо", MonsterType.Shape.BLOB, 0xFFCC80, 0x8D6E63, 1, 0)),
        new Zone("Снежный перевал", 0xBBDEFB, 0xFFFFFF, 0xECEFF1, 0xCFD8DC, Particles.SNOW, false, false,
            m("Ледяной они", MonsterType.Shape.BLOB, 0x4FC3F7, 0xE1F5FE, 2, 2),
            m("Юки-онна", MonsterType.Shape.GHOST, 0xE1F5FE, 0x81D4FA, 2, 0),
            m("Снежный дух", MonsterType.Shape.BLOB, 0xFFFFFF, 0x90CAF9, 2, 0),
            m("Цурара-кодзо", MonsterType.Shape.WALL, 0xB3E5FC, 0x4FC3F7, 2, 0)),
        new Zone("Огненный вулкан", 0xFF8A65, 0xFFE0B2, 0x5D4037, 0x8D6E63, Particles.EMBERS, false, false,
            m("Ванъюдо", MonsterType.Shape.WHEEL, 0xFF6F00, 0x4E342E, 2, 0),
            m("Хи-но-тама", MonsterType.Shape.FIREBALL, 0xFF7043, 0xFFEB3B, 2, 0),
            m("Огненный они", MonsterType.Shape.BLOB, 0xFF5722, 0xFFCCBC, 2, 1),
            m("Каэн-обакэ", MonsterType.Shape.GHOST, 0xFF8A65, 0xFFAB91, 2, 0)),
        new Zone("Небесный дворец", 0xFFF3C4, 0xFFFDE7, 0xFFE082, 0xFFCC80, Particles.PETALS, false, true,
            m("Ямата-но Ороти", MonsterType.Shape.SERPENT, 0x7CB342, 0xFFF59D, 2, 2),
            m("Райдзю", MonsterType.Shape.CAT, 0xFFEB3B, 0x1E88E5, 2, 0),
            m("Облачный дух", MonsterType.Shape.GHOST, 0xFFFFFF, 0xB3E5FC, 2, 0),
            m("Золотой дракончик", MonsterType.Shape.SERPENT, 0xFFC107, 0xFFF8E1, 2, 2))
    );

    public static Zone forStage(int stage) {
        return ZONES.get(((stage - 1) / 10) % ZONES.size());
    }

    /** Название локации с номером круга (после всех локаций они повторяются сильнее). */
    public static String nameForStage(int stage) {
        int round = (stage - 1) / 10 / ZONES.size();
        String base = forStage(stage).name;
        return round == 0 ? base : base + " " + roman(round + 1);
    }

    private static String roman(int n) {
        String[] r = {"", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"};
        return n < r.length ? r[n] : String.valueOf(n);
    }
}
