package sakura;

import java.util.ArrayList;
import java.util.List;

import static sakura.Look.c;

/** Наряд Сакуры. Открывается за первую победу над боссом этапа [unlockStage]. */
public final class Outfit {
    public final String id, name, bonusText;
    public final int unlockStage;
    public final Look look;
    public double clickMult = 1, dpsMult = 1, goldMult = 1, critAdd = 0, timerAdd = 0;
    public boolean owned;

    private Outfit(String id, String name, int unlockStage, String bonusText, Look look) {
        this.id = id;
        this.name = name;
        this.unlockStage = unlockStage;
        this.bonusText = bonusText;
        this.look = look;
        this.owned = unlockStage == 0;
    }

    public static List<Outfit> createAll() {
        List<Outfit> list = new ArrayList<>();
        Look casual = new Look(c(0xF06292), c(0xF8BBD0), c(0xF48FB1), c(0xC2185B), Look.Weapon.SWORD);
        list.add(new Outfit("casual", "Повседневное кимоно", 0, "Без бонусов", casual));

        Outfit yukata = new Outfit("yukata", "Юката фестиваля", 5, "Золото +25%",
            new Look(null, c(0x4FC3F7), c(0x29B6F6), c(0xFFEB3B), null));
        yukata.goldMult = 1.25;
        list.add(yukata);

        Outfit miko = new Outfit("miko", "Наряд мико", 10, "Урон клика ×1,5",
            new Look(null, c(0xFFFFFF), c(0xE53935), c(0xE53935), null));
        miko.clickMult = 1.5;
        list.add(miko);

        Look samuraiLook = new Look(null, c(0xB71C1C), c(0x37474F), c(0x212121), null);
        samuraiLook.helmet = true;
        Outfit samurai = new Outfit("samurai", "Доспехи самурая", 20, "Клик ×1,25 • героини ×1,25", samuraiLook);
        samurai.clickMult = 1.25;
        samurai.dpsMult = 1.25;
        list.add(samurai);

        Look ninjaLook = new Look(null, c(0x37474F), c(0x263238), c(0xE91E63), null);
        ninjaLook.scarf = true;
        Outfit ninja = new Outfit("ninja", "Одежда ниндзя", 30, "Шанс крита +7%", ninjaLook);
        ninja.critAdd = 0.07;
        list.add(ninja);

        Look winterLook = new Look(null, c(0xE3F2FD), c(0x90CAF9), c(0x1E88E5), null);
        winterLook.hood = true;
        Outfit winter = new Outfit("winter", "Зимнее кимоно", 40, "Время на босса +7 с", winterLook);
        winter.timerAdd = 7;
        list.add(winter);

        Look foxLook = new Look(null, c(0xFFF3E0), c(0xFF7043), c(0xBF360C), null);
        foxLook.foxEars = true;
        Outfit fox = new Outfit("kitsune", "Наряд лисьей жрицы", 50, "Золото ×1,75", foxLook);
        fox.goldMult = 1.75;
        list.add(fox);

        Outfit furisode = new Outfit("furisode", "Церемониальное фурисодэ", 75, "Клик ×1,5 • героини ×1,5",
            new Look(null, c(0x8E24AA), c(0x6A1B9A), c(0xFFD54F), null));
        furisode.clickMult = 1.5;
        furisode.dpsMult = 1.5;
        list.add(furisode);

        Look goddessLook = new Look(null, c(0xFFFFFF), c(0xF8BBD0), c(0xFFC107), null);
        goddessLook.halo = true;
        Outfit goddess = new Outfit("goddess", "Облачение богини сакуры", 100, "Весь урон ×2 • золото ×1,5", goddessLook);
        goddess.clickMult = 2;
        goddess.dpsMult = 2;
        goddess.goldMult = 1.5;
        list.add(goddess);
        return list;
    }

    /** Как выглядит Сакура в этом наряде. */
    public Look sakuraLook() {
        Look l = look.withHair(c(0xF06292), Look.Weapon.SWORD);
        l.flower = !look.helmet && !look.hood;
        return l;
    }
}
