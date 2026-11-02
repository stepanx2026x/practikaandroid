package sakura;

import java.awt.Color;

/** Внешний вид героини: цвета причёски и наряда, аксессуары, оружие. */
public final class Look {
    public enum Weapon { SWORD, TALISMAN, BOW, KUNAI, SPEAR, FAN, LANTERN, STAFF }

    public final Color hair, top, lower, obi;
    public final Weapon weapon;
    public boolean foxEars, helmet, scarf, halo, flower, hood;

    public Look(Color hair, Color top, Color lower, Color obi, Weapon weapon) {
        this.hair = hair;
        this.top = top;
        this.lower = lower;
        this.obi = obi;
        this.weapon = weapon;
    }

    public Look withHair(Color newHair, Weapon newWeapon) {
        Look l = new Look(newHair, top, lower, obi, newWeapon);
        l.foxEars = foxEars;
        l.helmet = helmet;
        l.scarf = scarf;
        l.halo = halo;
        l.flower = flower;
        l.hood = hood;
        return l;
    }

    public static Color c(int rgb) {
        return new Color(rgb);
    }
}
