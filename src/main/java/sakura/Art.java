package sakura;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.GradientPaint;
import java.awt.Graphics2D;
import java.awt.RadialGradientPaint;
import java.awt.RenderingHints;
import java.awt.geom.AffineTransform;
import java.awt.geom.Arc2D;
import java.awt.geom.Ellipse2D;
import java.awt.geom.GeneralPath;
import java.awt.geom.Path2D;
import java.awt.geom.RoundRectangle2D;
import java.awt.image.BufferedImage;
import java.util.Random;

/** Процедурная графика: героини в стиле чиби, ёкаи, сакура, лепестки. */
public final class Art {
    private Art() {}

    public static final Color SKIN = new Color(0xFFE3D3);
    public static final Color OUTLINE = new Color(0x4A2C3A);
    private static final BasicStroke THIN = new BasicStroke(2f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND);

    public static void smooth(Graphics2D g) {
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_STROKE_CONTROL, RenderingHints.VALUE_STROKE_PURE);
    }

    public static Color alpha(Color c, int a) {
        return new Color(c.getRed(), c.getGreen(), c.getBlue(), Math.max(0, Math.min(255, a)));
    }

    public static Color darker(Color c, double f) {
        return new Color((int) (c.getRed() * f), (int) (c.getGreen() * f), (int) (c.getBlue() * f), c.getAlpha());
    }

    public static Color lighter(Color c, double f) {
        return new Color(
            (int) (c.getRed() + (255 - c.getRed()) * f),
            (int) (c.getGreen() + (255 - c.getGreen()) * f),
            (int) (c.getBlue() + (255 - c.getBlue()) * f), c.getAlpha());
    }

    private static void fillOutlined(Graphics2D g, java.awt.Shape s, Color fill, Color line) {
        g.setColor(fill);
        g.fill(s);
        g.setColor(line);
        g.setStroke(THIN);
        g.draw(s);
    }

    // =====================================================================
    // Героини
    // =====================================================================

    /**
     * Рисует героиню-чиби. (x, y) — точка между ступнями, s — полный рост.
     * weaponAngle — поворот оружия в радианах (для анимации удара).
     */
    public static void drawChibi(Graphics2D g, Look look, double x, double y, double s, double weaponAngle, double time) {
        AffineTransform saved = g.getTransform();
        double headR = s * 0.23;
        double headY = y - s * 0.70;
        Color hairDark = darker(look.hair, 0.8);
        Color line = darker(look.hair, 0.55);

        // Нимб
        if (look.halo) {
            g.setStroke(new BasicStroke((float) (s * 0.025)));
            g.setColor(new Color(255, 213, 79, 220));
            g.draw(new Ellipse2D.Double(x - headR * 0.8, headY - headR * 1.55, headR * 1.6, headR * 0.45));
        }

        // Волосы сзади (длинные)
        GeneralPath backHair = new GeneralPath();
        backHair.moveTo(x - headR * 1.05, headY);
        backHair.curveTo(x - headR * 1.2, headY + headR * 1.4, x - headR * 0.9, y - s * 0.30, x - headR * 0.55, y - s * 0.30);
        backHair.lineTo(x + headR * 0.55, y - s * 0.30);
        backHair.curveTo(x + headR * 0.9, y - s * 0.30, x + headR * 1.2, headY + headR * 1.4, x + headR * 1.05, headY);
        backHair.closePath();
        if (!look.helmet && !look.hood) fillOutlined(g, backHair, hairDark, line);

        // Ноги (гэта)
        g.setColor(new Color(0x5D4037));
        g.fill(new RoundRectangle2D.Double(x - s * 0.13, y - s * 0.035, s * 0.1, s * 0.035, 4, 4));
        g.fill(new RoundRectangle2D.Double(x + s * 0.03, y - s * 0.035, s * 0.1, s * 0.035, 4, 4));

        // Нижняя часть наряда (хакама / подол)
        GeneralPath lower = new GeneralPath();
        lower.moveTo(x - s * 0.17, y - s * 0.30);
        lower.lineTo(x + s * 0.17, y - s * 0.30);
        lower.lineTo(x + s * 0.24, y - s * 0.03);
        lower.quadTo(x, y + s * 0.01, x - s * 0.24, y - s * 0.03);
        lower.closePath();
        fillOutlined(g, lower, look.lower, darker(look.lower, 0.6));
        g.setColor(darker(look.lower, 0.8));
        g.setStroke(new BasicStroke(1.5f));
        g.draw(new java.awt.geom.Line2D.Double(x, y - s * 0.28, x, y - s * 0.04));

        // Верх кимоно
        GeneralPath top = new GeneralPath();
        top.moveTo(x - s * 0.13, y - s * 0.47);
        top.lineTo(x + s * 0.13, y - s * 0.47);
        top.lineTo(x + s * 0.18, y - s * 0.28);
        top.lineTo(x - s * 0.18, y - s * 0.28);
        top.closePath();
        fillOutlined(g, top, look.top, darker(look.top, 0.6));
        // Ворот
        g.setColor(Color.WHITE);
        g.setStroke(new BasicStroke((float) (s * 0.018)));
        g.draw(new java.awt.geom.Line2D.Double(x - s * 0.07, y - s * 0.47, x + s * 0.02, y - s * 0.36));
        g.draw(new java.awt.geom.Line2D.Double(x + s * 0.07, y - s * 0.47, x - s * 0.02, y - s * 0.38));
        // Пояс-оби
        fillOutlined(g, new RoundRectangle2D.Double(x - s * 0.185, y - s * 0.34, s * 0.37, s * 0.07, 6, 6),
            look.obi, darker(look.obi, 0.6));
        g.setColor(lighter(look.obi, 0.4));
        g.fill(new Ellipse2D.Double(x - s * 0.03, y - s * 0.325, s * 0.06, s * 0.045));

        // Рукава
        double sway = Math.sin(time * 2) * s * 0.01;
        fillOutlined(g, new RoundRectangle2D.Double(x - s * 0.30, y - s * 0.46 + sway, s * 0.15, s * 0.20, 14, 14),
            look.top, darker(look.top, 0.6));
        fillOutlined(g, new RoundRectangle2D.Double(x + s * 0.15, y - s * 0.46 - sway, s * 0.15, s * 0.20, 14, 14),
            look.top, darker(look.top, 0.6));

        // Голова
        fillOutlined(g, new Ellipse2D.Double(x - headR, headY - headR, headR * 2, headR * 2), SKIN, new Color(0xC9A28E));

        // Глаза
        double eyeY = headY + headR * 0.15;
        for (int side = -1; side <= 1; side += 2) {
            double ex = x + side * headR * 0.42;
            Color iris = darker(look.hair, 0.5);
            g.setColor(iris);
            g.fill(new Ellipse2D.Double(ex - headR * 0.17, eyeY - headR * 0.26, headR * 0.34, headR * 0.44));
            g.setColor(Color.WHITE);
            g.fill(new Ellipse2D.Double(ex - headR * 0.09, eyeY - headR * 0.20, headR * 0.13, headR * 0.15));
            g.fill(new Ellipse2D.Double(ex + headR * 0.03, eyeY + headR * 0.05, headR * 0.06, headR * 0.06));
            // Румянец
            g.setColor(new Color(255, 138, 128, 110));
            g.fill(new Ellipse2D.Double(ex - headR * 0.18 + side * headR * 0.12, eyeY + headR * 0.28, headR * 0.3, headR * 0.14));
        }
        // Рот
        if (!look.scarf) {
            g.setColor(new Color(0xB5546A));
            g.setStroke(new BasicStroke(1.6f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
            g.draw(new Arc2D.Double(x - headR * 0.12, eyeY + headR * 0.28, headR * 0.24, headR * 0.16, 200, 140, Arc2D.OPEN));
        } else {
            fillOutlined(g, new RoundRectangle2D.Double(x - headR * 0.85, eyeY + headR * 0.3, headR * 1.7, headR * 0.45, 10, 10),
                look.obi, darker(look.obi, 0.6));
        }

        // Чёлка
        if (look.helmet) {
            drawHelmet(g, x, headY, headR);
        } else if (look.hood) {
            GeneralPath hood = new GeneralPath();
            hood.moveTo(x - headR * 1.2, headY + headR * 0.6);
            hood.curveTo(x - headR * 1.3, headY - headR * 1.6, x + headR * 1.3, headY - headR * 1.6, x + headR * 1.2, headY + headR * 0.6);
            hood.curveTo(x + headR * 0.9, headY - headR * 0.6, x - headR * 0.9, headY - headR * 0.6, x - headR * 1.2, headY + headR * 0.6);
            hood.closePath();
            fillOutlined(g, hood, look.top, darker(look.top, 0.6));
            g.setColor(look.hair);
            g.fill(new Ellipse2D.Double(x - headR * 0.75, headY - headR * 0.75, headR * 1.5, headR * 0.6));
        } else {
            GeneralPath bangs = new GeneralPath();
            bangs.moveTo(x - headR * 1.05, headY + headR * 0.2);
            bangs.curveTo(x - headR * 1.15, headY - headR * 1.3, x + headR * 1.15, headY - headR * 1.3, x + headR * 1.05, headY + headR * 0.2);
            bangs.lineTo(x + headR * 0.75, headY - headR * 0.15);
            bangs.lineTo(x + headR * 0.45, headY + headR * 0.05);
            bangs.lineTo(x + headR * 0.2, headY - headR * 0.25);
            bangs.lineTo(x - headR * 0.1, headY + headR * 0.0);
            bangs.lineTo(x - headR * 0.35, headY - headR * 0.25);
            bangs.lineTo(x - headR * 0.65, headY + headR * 0.05);
            bangs.lineTo(x - headR * 0.85, headY - headR * 0.1);
            bangs.closePath();
            fillOutlined(g, bangs, look.hair, line);
            // Блик на волосах
            g.setColor(new Color(255, 255, 255, 90));
            g.setStroke(new BasicStroke((float) (headR * 0.08), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
            g.draw(new Arc2D.Double(x - headR * 0.7, headY - headR * 0.85, headR * 1.4, headR * 0.9, 120, 60, Arc2D.OPEN));
        }

        // Лисьи ушки
        if (look.foxEars) {
            for (int side = -1; side <= 1; side += 2) {
                GeneralPath ear = new GeneralPath();
                ear.moveTo(x + side * headR * 0.35, headY - headR * 0.8);
                ear.lineTo(x + side * headR * 0.95, headY - headR * 1.55);
                ear.lineTo(x + side * headR * 1.0, headY - headR * 0.5);
                ear.closePath();
                Color earColor = look.hair == null ? new Color(0xFFA726) : look.hair;
                fillOutlined(g, ear, earColor, line);
                GeneralPath inner = new GeneralPath();
                inner.moveTo(x + side * headR * 0.55, headY - headR * 0.8);
                inner.lineTo(x + side * headR * 0.9, headY - headR * 1.3);
                inner.lineTo(x + side * headR * 0.92, headY - headR * 0.7);
                inner.closePath();
                g.setColor(new Color(0xFFCDD2));
                g.fill(inner);
            }
        }

        // Цветок сакуры в волосах
        if (look.flower) drawBlossom(g, x + headR * 0.8, headY - headR * 0.55, headR * 0.32, new Color(0xFFFFFF), new Color(0xF06292));

        // Оружие
        drawWeapon(g, look.weapon, x + s * 0.24, y - s * 0.35, s, weaponAngle);
        g.setTransform(saved);
    }

    private static void drawHelmet(Graphics2D g, double x, double headY, double headR) {
        Arc2D dome = new Arc2D.Double(x - headR * 1.15, headY - headR * 1.2, headR * 2.3, headR * 2.1, 0, 180, Arc2D.CHORD);
        fillOutlined(g, dome, new Color(0x37474F), new Color(0x212121));
        g.setColor(new Color(0x263238));
        g.fill(new RoundRectangle2D.Double(x - headR * 1.25, headY - headR * 0.2, headR * 2.5, headR * 0.25, 6, 6));
        // Золотые рога-кувагата
        g.setColor(new Color(0xFFD54F));
        g.setStroke(new BasicStroke((float) (headR * 0.14), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        Path2D horns = new Path2D.Double();
        horns.moveTo(x - headR * 0.1, headY - headR * 0.9);
        horns.quadTo(x - headR * 0.9, headY - headR * 1.4, x - headR * 1.0, headY - headR * 2.0);
        horns.moveTo(x + headR * 0.1, headY - headR * 0.9);
        horns.quadTo(x + headR * 0.9, headY - headR * 1.4, x + headR * 1.0, headY - headR * 2.0);
        g.draw(horns);
    }

    private static void drawWeapon(Graphics2D g, Look.Weapon w, double hx, double hy, double s, double angle) {
        if (w == null) return;
        AffineTransform saved = g.getTransform();
        g.translate(hx, hy);
        g.rotate(angle);
        switch (w) {
            case SWORD -> {
                g.setColor(new Color(0x3E2723));
                g.fill(new RoundRectangle2D.Double(-s * 0.02, -s * 0.02, s * 0.04, s * 0.12, 4, 4));
                g.setColor(new Color(0xFFD54F));
                g.fill(new Ellipse2D.Double(-s * 0.04, -s * 0.04, s * 0.08, s * 0.03));
                GeneralPath blade = new GeneralPath();
                blade.moveTo(-s * 0.015, -s * 0.03);
                blade.lineTo(s * 0.015, -s * 0.03);
                blade.quadTo(s * 0.03, -s * 0.4, -s * 0.03, -s * 0.62);
                blade.quadTo(-s * 0.02, -s * 0.4, -s * 0.015, -s * 0.03);
                fillOutlined(g, blade, new Color(0xECEFF1), new Color(0x90A4AE));
            }
            case SPEAR -> {
                g.setColor(new Color(0x6D4C41));
                g.fill(new RoundRectangle2D.Double(-s * 0.012, -s * 0.6, s * 0.024, s * 0.8, 4, 4));
                GeneralPath tip = new GeneralPath();
                tip.moveTo(-s * 0.012, -s * 0.6);
                tip.quadTo(s * 0.08, -s * 0.7, s * 0.0, -s * 0.85);
                tip.quadTo(-s * 0.02, -s * 0.7, -s * 0.012, -s * 0.6);
                fillOutlined(g, tip, new Color(0xECEFF1), new Color(0x90A4AE));
            }
            case BOW -> {
                g.setColor(new Color(0x8D6E63));
                g.setStroke(new BasicStroke((float) (s * 0.025), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                g.draw(new Arc2D.Double(-s * 0.12, -s * 0.35, s * 0.2, s * 0.6, -80, 160, Arc2D.OPEN));
                g.setColor(new Color(0xEEEEEE));
                g.setStroke(new BasicStroke(1.2f));
                g.draw(new java.awt.geom.Line2D.Double(-s * 0.0, -s * 0.34, -s * 0.0, s * 0.24));
            }
            case KUNAI -> {
                GeneralPath k = new GeneralPath();
                k.moveTo(0, -s * 0.25);
                k.lineTo(s * 0.035, -s * 0.08);
                k.lineTo(-s * 0.035, -s * 0.08);
                k.closePath();
                fillOutlined(g, k, new Color(0xB0BEC5), new Color(0x546E7A));
                g.setColor(new Color(0x212121));
                g.fill(new RoundRectangle2D.Double(-s * 0.012, -s * 0.08, s * 0.024, s * 0.1, 4, 4));
            }
            case TALISMAN -> {
                for (int i = 0; i < 3; i++) {
                    AffineTransform t = g.getTransform();
                    g.rotate((i - 1) * 0.35);
                    fillOutlined(g, new java.awt.geom.Rectangle2D.Double(-s * 0.035, -s * 0.24, s * 0.07, s * 0.18),
                        new Color(0xFFFDE7), new Color(0xD7CCC8));
                    g.setColor(new Color(0xE53935));
                    g.fill(new java.awt.geom.Rectangle2D.Double(-s * 0.01, -s * 0.22, s * 0.02, s * 0.12));
                    g.setTransform(t);
                }
            }
            case FAN -> {
                Arc2D fan = new Arc2D.Double(-s * 0.18, -s * 0.2, s * 0.36, s * 0.36, 20, 140, Arc2D.PIE);
                fillOutlined(g, fan, new Color(0xFFF8E1), new Color(0xBCAAA4));
                g.setColor(new Color(0xE57373));
                g.fill(new Arc2D.Double(-s * 0.08, -s * 0.1, s * 0.16, s * 0.16, 20, 140, Arc2D.PIE));
            }
            case LANTERN -> {
                g.setColor(new Color(0x5D4037));
                g.setStroke(new BasicStroke((float) (s * 0.015)));
                g.draw(new java.awt.geom.Line2D.Double(0, 0, 0, -s * 0.3));
                g.setPaint(new RadialGradientPaint(new java.awt.geom.Point2D.Double(0, -s * 0.36), (float) (s * 0.12),
                    new float[] {0f, 1f}, new Color[] {new Color(0xFFF59D), new Color(0xFF7043)}));
                g.fill(new Ellipse2D.Double(-s * 0.07, -s * 0.44, s * 0.14, s * 0.16));
                g.setColor(new Color(255, 213, 79, 70));
                g.fill(new Ellipse2D.Double(-s * 0.16, -s * 0.52, s * 0.32, s * 0.32));
            }
            case STAFF -> {
                g.setColor(new Color(0x8D6E63));
                g.fill(new RoundRectangle2D.Double(-s * 0.012, -s * 0.55, s * 0.024, s * 0.75, 4, 4));
                g.setPaint(new RadialGradientPaint(new java.awt.geom.Point2D.Double(0, -s * 0.6), (float) (s * 0.08),
                    new float[] {0f, 1f}, new Color[] {Color.WHITE, new Color(0x7E57C2)}));
                g.fill(new Ellipse2D.Double(-s * 0.06, -s * 0.66, s * 0.12, s * 0.12));
            }
        }
        g.setTransform(saved);
    }

    /** Цветок сакуры из пяти лепестков. */
    public static void drawBlossom(Graphics2D g, double cx, double cy, double r, Color petal, Color center) {
        AffineTransform saved = g.getTransform();
        g.translate(cx, cy);
        for (int i = 0; i < 5; i++) {
            g.rotate(Math.PI * 2 / 5);
            g.setColor(petal);
            g.fill(new Ellipse2D.Double(-r * 0.35, -r * 1.0, r * 0.7, r * 0.9));
            g.setColor(new Color(0xF8BBD0));
            g.fill(new Ellipse2D.Double(-r * 0.12, -r * 0.95, r * 0.24, r * 0.2));
        }
        g.setColor(center);
        g.fill(new Ellipse2D.Double(-r * 0.25, -r * 0.25, r * 0.5, r * 0.5));
        g.setTransform(saved);
    }

    /** Один лепесток (для частиц). */
    public static void drawPetal(Graphics2D g, double x, double y, double size, double angle, Color c) {
        AffineTransform saved = g.getTransform();
        g.translate(x, y);
        g.rotate(angle);
        GeneralPath p = new GeneralPath();
        p.moveTo(0, -size);
        p.curveTo(size * 0.8, -size * 0.6, size * 0.6, size * 0.6, 0, size);
        p.curveTo(-size * 0.6, size * 0.6, -size * 0.8, -size * 0.6, 0, -size);
        g.setColor(c);
        g.fill(p);
        g.setTransform(saved);
    }

    // =====================================================================
    // Ёкаи
    // =====================================================================

    /** Рисует ёкая. (cx, cy) — центр, size — примерная высота. */
    public static void drawMonster(Graphics2D g, MonsterType t, double cx, double cy, double size, double time, boolean boss) {
        if (boss) {
            g.setPaint(new RadialGradientPaint(new java.awt.geom.Point2D.Double(cx, cy), (float) (size * 0.95),
                new float[] {0f, 1f}, new Color[] {alpha(t.body, 110), alpha(t.body, 0)}));
            g.fill(new Ellipse2D.Double(cx - size * 0.95, cy - size * 0.95, size * 1.9, size * 1.9));
        }
        // Тень
        g.setColor(new Color(0, 0, 0, 40));
        g.fill(new Ellipse2D.Double(cx - size * 0.45, cy + size * 0.48, size * 0.9, size * 0.14));

        Color body = t.body;
        Color line = darker(body, 0.55);
        switch (t.shape) {
            case BLOB -> {
                Ellipse2D e = new Ellipse2D.Double(cx - size * 0.5, cy - size * 0.45, size, size * 0.98);
                g.setPaint(new GradientPaint((float) cx, (float) (cy - size * 0.45), lighter(body, 0.35), (float) cx, (float) (cy + size * 0.5), body));
                g.fill(e);
                g.setColor(line);
                g.setStroke(new BasicStroke(3f));
                g.draw(e);
                drawHorns(g, t, cx, cy - size * 0.4, size);
                if (t.name.equals("Каппа")) {
                    fillOutlined(g, new Ellipse2D.Double(cx - size * 0.25, cy - size * 0.52, size * 0.5, size * 0.16), t.accent, line);
                }
                drawFace(g, t, cx, cy - size * 0.05, size, boss, time);
                // Лапки
                g.setColor(darker(body, 0.85));
                g.fill(new Ellipse2D.Double(cx - size * 0.35, cy + size * 0.42, size * 0.22, size * 0.12));
                g.fill(new Ellipse2D.Double(cx + size * 0.13, cy + size * 0.42, size * 0.22, size * 0.12));
            }
            case GHOST -> {
                GeneralPath p = new GeneralPath();
                p.moveTo(cx - size * 0.42, cy + size * 0.45);
                p.lineTo(cx - size * 0.42, cy - size * 0.05);
                p.curveTo(cx - size * 0.42, cy - size * 0.6, cx + size * 0.42, cy - size * 0.6, cx + size * 0.42, cy - size * 0.05);
                p.lineTo(cx + size * 0.42, cy + size * 0.45);
                int waves = 4;
                for (int i = 0; i < waves; i++) {
                    double x1 = cx + size * 0.42 - size * 0.84 * (i + 0.5) / waves;
                    double x2 = cx + size * 0.42 - size * 0.84 * (i + 1) / waves;
                    double wob = Math.sin(time * 4 + i) * size * 0.04;
                    p.quadTo(x1, cy + size * 0.30 + wob, x2, cy + size * 0.45);
                }
                p.closePath();
                g.setPaint(new GradientPaint((float) cx, (float) (cy - size * 0.5), lighter(body, 0.3), (float) cx, (float) (cy + size * 0.45), alpha(body, 200)));
                g.fill(p);
                g.setColor(line);
                g.setStroke(new BasicStroke(2.5f));
                g.draw(p);
                drawHorns(g, t, cx, cy - size * 0.4, size);
                drawFace(g, t, cx, cy - size * 0.08, size, boss, time);
            }
            case LANTERN -> {
                Ellipse2D e = new Ellipse2D.Double(cx - size * 0.38, cy - size * 0.5, size * 0.76, size * 0.98);
                g.setPaint(new RadialGradientPaint(new java.awt.geom.Point2D.Double(cx, cy), (float) (size * 0.55),
                    new float[] {0f, 1f}, new Color[] {new Color(0xFFF3E0), body}));
                g.fill(e);
                g.setColor(alpha(t.accent, 160));
                g.setStroke(new BasicStroke(2f));
                for (int i = 1; i < 6; i++) {
                    double yy = cy - size * 0.5 + size * 0.98 * i / 6;
                    double half = Math.sqrt(Math.max(0, 1 - Math.pow((yy - (cy - size * 0.01)) / (size * 0.49), 2))) * size * 0.38;
                    g.draw(new java.awt.geom.Line2D.Double(cx - half, yy, cx + half, yy));
                }
                g.setColor(line);
                g.setStroke(new BasicStroke(3f));
                g.draw(e);
                g.setColor(new Color(0x3E2723));
                g.fill(new RoundRectangle2D.Double(cx - size * 0.22, cy - size * 0.58, size * 0.44, size * 0.1, 6, 6));
                g.fill(new RoundRectangle2D.Double(cx - size * 0.22, cy + size * 0.44, size * 0.44, size * 0.1, 6, 6));
                drawFace(g, t, cx, cy - size * 0.08, size, boss, time);
                // Язык
                GeneralPath tongue = new GeneralPath();
                tongue.moveTo(cx - size * 0.06, cy + size * 0.12);
                tongue.quadTo(cx, cy + size * (0.42 + 0.03 * Math.sin(time * 5)), cx + size * 0.08, cy + size * 0.12);
                fillOutlined(g, tongue, new Color(0xE57373), new Color(0xC62828));
            }
            case UMBRELLA -> {
                GeneralPath canopy = new GeneralPath();
                canopy.moveTo(cx, cy - size * 0.6);
                canopy.lineTo(cx + size * 0.55, cy + size * 0.12);
                canopy.quadTo(cx, cy + size * 0.02, cx - size * 0.55, cy + size * 0.12);
                canopy.closePath();
                fillOutlined(g, canopy, body, line);
                g.setColor(alpha(line, 140));
                g.setStroke(new BasicStroke(1.5f));
                for (int i = -2; i <= 2; i++) {
                    g.draw(new java.awt.geom.Line2D.Double(cx, cy - size * 0.6, cx + i * size * 0.22, cy + size * 0.08));
                }
                drawFace(g, t, cx, cy - size * 0.18, size * 0.9, boss, time);
                // Нога и гэта
                double hop = Math.abs(Math.sin(time * 4)) * size * 0.06;
                g.setColor(new Color(0xFFCCBC));
                g.setStroke(new BasicStroke((float) (size * 0.06), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                g.draw(new java.awt.geom.Line2D.Double(cx, cy + size * 0.08, cx, cy + size * 0.42 - hop));
                g.setColor(new Color(0x6D4C41));
                g.fill(new RoundRectangle2D.Double(cx - size * 0.1, cy + size * 0.42 - hop, size * 0.2, size * 0.06, 4, 4));
            }
            case CAT -> {
                // Хвост
                g.setColor(darker(body, 0.9));
                g.setStroke(new BasicStroke((float) (size * 0.09), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                Path2D tail = new Path2D.Double();
                tail.moveTo(cx + size * 0.35, cy + size * 0.3);
                tail.curveTo(cx + size * 0.75, cy + size * 0.2, cx + size * 0.55, cy - size * (0.3 + 0.08 * Math.sin(time * 3)), cx + size * 0.7, cy - size * 0.45);
                g.draw(tail);
                Ellipse2D e = new Ellipse2D.Double(cx - size * 0.45, cy - size * 0.35, size * 0.9, size * 0.85);
                for (int side = -1; side <= 1; side += 2) {
                    GeneralPath ear = new GeneralPath();
                    ear.moveTo(cx + side * size * 0.12, cy - size * 0.32);
                    ear.lineTo(cx + side * size * 0.38, cy - size * 0.62);
                    ear.lineTo(cx + side * size * 0.42, cy - size * 0.12);
                    ear.closePath();
                    fillOutlined(g, ear, body, line);
                }
                g.setPaint(new GradientPaint((float) cx, (float) (cy - size * 0.35), lighter(body, 0.3), (float) cx, (float) (cy + size * 0.5), body));
                g.fill(e);
                g.setColor(line);
                g.setStroke(new BasicStroke(3f));
                g.draw(e);
                drawHorns(g, t, cx, cy - size * 0.32, size * 0.8);
                drawFace(g, t, cx, cy + size * 0.02, size, boss, time);
                // Усы
                g.setColor(alpha(line, 180));
                g.setStroke(new BasicStroke(1.5f));
                for (int side = -1; side <= 1; side += 2) {
                    for (int k = -1; k <= 1; k++) {
                        g.draw(new java.awt.geom.Line2D.Double(cx + side * size * 0.18, cy + size * 0.18,
                            cx + side * size * 0.45, cy + size * (0.13 + k * 0.06)));
                    }
                }
            }
            case SPIDER -> {
                g.setColor(darker(body, 0.7));
                g.setStroke(new BasicStroke((float) (size * 0.05), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                for (int side = -1; side <= 1; side += 2) {
                    for (int i = 0; i < 4; i++) {
                        double wiggle = Math.sin(time * 6 + i) * size * 0.03;
                        double ky = cy - size * 0.1 + i * size * 0.12;
                        Path2D leg = new Path2D.Double();
                        leg.moveTo(cx + side * size * 0.2, ky);
                        leg.lineTo(cx + side * size * (0.55 + i * 0.03), ky - size * 0.25 + wiggle);
                        leg.lineTo(cx + side * size * (0.7 + i * 0.02), ky + size * 0.3);
                        g.draw(leg);
                    }
                }
                Ellipse2D abd = new Ellipse2D.Double(cx - size * 0.38, cy - size * 0.2, size * 0.76, size * 0.68);
                fillOutlined(g, abd, body, line);
                g.setColor(t.accent);
                g.fill(new Ellipse2D.Double(cx - size * 0.1, cy + size * 0.12, size * 0.2, size * 0.22));
                Ellipse2D head = new Ellipse2D.Double(cx - size * 0.25, cy - size * 0.5, size * 0.5, size * 0.42);
                fillOutlined(g, head, lighter(body, 0.15), line);
                for (int i = 0; i < 4; i++) {
                    g.setColor(t.accent);
                    double ex = cx - size * 0.15 + i * size * 0.1;
                    g.fill(new Ellipse2D.Double(ex - size * 0.035, cy - size * 0.36 - (i % 3 == 0 ? 0 : size * 0.04), size * 0.07, size * 0.07));
                }
                g.setColor(new Color(0xFFF8E1));
                g.fill(new java.awt.geom.Rectangle2D.Double(cx - size * 0.07, cy - size * 0.16, size * 0.04, size * 0.07));
                g.fill(new java.awt.geom.Rectangle2D.Double(cx + size * 0.03, cy - size * 0.16, size * 0.04, size * 0.07));
            }
            case WALL -> {
                RoundRectangle2D r = new RoundRectangle2D.Double(cx - size * 0.45, cy - size * 0.5, size * 0.9, size * 0.98, 24, 24);
                fillOutlined(g, r, body, line);
                g.setColor(alpha(t.accent, 160));
                g.setStroke(new BasicStroke(2f));
                for (int row = 1; row < 5; row++) {
                    double yy = cy - size * 0.5 + row * size * 0.196;
                    g.draw(new java.awt.geom.Line2D.Double(cx - size * 0.45, yy, cx + size * 0.45, yy));
                    for (int col = 0; col < 3; col++) {
                        double xx = cx - size * 0.3 + col * size * 0.3 + (row % 2) * size * 0.15;
                        g.draw(new java.awt.geom.Line2D.Double(xx, yy, xx, yy - size * 0.196));
                    }
                }
                drawFace(g, t, cx, cy - size * 0.05, size, boss, time);
                g.setColor(darker(body, 0.85));
                g.fill(new Ellipse2D.Double(cx - size * 0.6, cy + size * 0.05, size * 0.18, size * 0.14));
                g.fill(new Ellipse2D.Double(cx + size * 0.42, cy + size * 0.05, size * 0.18, size * 0.14));
            }
            case SERPENT -> {
                int heads = boss ? 3 : 1;
                for (int hIdx = 0; hIdx < heads; hIdx++) {
                    double off = (hIdx - (heads - 1) / 2.0) * size * 0.42;
                    double sway = Math.sin(time * 2 + hIdx) * size * 0.05;
                    Path2D neck = new Path2D.Double();
                    neck.moveTo(cx + off * 0.4, cy + size * 0.5);
                    neck.curveTo(cx + off - size * 0.35, cy + size * 0.2, cx + off + size * 0.35, cy - size * 0.05, cx + off + sway, cy - size * 0.28);
                    g.setColor(line);
                    g.setStroke(new BasicStroke((float) (size * 0.24), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                    g.draw(neck);
                    g.setColor(body);
                    g.setStroke(new BasicStroke((float) (size * 0.2), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                    g.draw(neck);
                    g.setColor(alpha(t.accent, 200));
                    g.setStroke(new BasicStroke((float) (size * 0.06), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                    g.draw(neck);
                    double hx = cx + off + sway, hy = cy - size * 0.36;
                    double hs = size * (boss ? 0.55 : 0.7);
                    drawHorns(g, t, hx, hy - hs * 0.15, hs);
                    Ellipse2D head = new Ellipse2D.Double(hx - hs * 0.32, hy - hs * 0.22, hs * 0.64, hs * 0.48);
                    fillOutlined(g, head, lighter(body, 0.15), line);
                    drawFace(g, t, hx, hy, hs * 0.75, boss, time);
                }
            }
            case FIREBALL -> {
                for (int i = 0; i < 5; i++) {
                    double fx = cx + (i - 2) * size * 0.15;
                    double fh = size * (0.4 + 0.12 * Math.sin(time * 8 + i * 1.7));
                    GeneralPath flame = new GeneralPath();
                    flame.moveTo(fx - size * 0.12, cy - size * 0.1);
                    flame.quadTo(fx, cy - size * 0.1 - fh * 1.4, fx + size * 0.12, cy - size * 0.1);
                    flame.closePath();
                    g.setColor(alpha(t.accent, 200));
                    g.fill(flame);
                }
                g.setPaint(new RadialGradientPaint(new java.awt.geom.Point2D.Double(cx, cy), (float) (size * 0.45),
                    new float[] {0f, 0.6f, 1f}, new Color[] {new Color(0xFFF9C4), t.accent, body}));
                g.fill(new Ellipse2D.Double(cx - size * 0.42, cy - size * 0.35, size * 0.84, size * 0.84));
                drawFace(g, t, cx, cy + size * 0.05, size, boss, time);
            }
            case WHEEL -> {
                double r = size * 0.48;
                for (int i = 0; i < 10; i++) {
                    double a = time * 1.5 + i * Math.PI / 5;
                    double fl = r * (1.25 + 0.1 * Math.sin(time * 9 + i));
                    g.setColor(new Color(255, 152, 0, 170));
                    g.fill(new Ellipse2D.Double(cx + Math.cos(a) * fl - size * 0.08, cy + Math.sin(a) * fl - size * 0.08, size * 0.16, size * 0.16));
                }
                g.setColor(new Color(0x5D4037));
                g.setStroke(new BasicStroke((float) (size * 0.1)));
                g.draw(new Ellipse2D.Double(cx - r, cy - r, r * 2, r * 2));
                g.setStroke(new BasicStroke((float) (size * 0.04)));
                for (int i = 0; i < 8; i++) {
                    double a = time * 1.5 + i * Math.PI / 4;
                    g.draw(new java.awt.geom.Line2D.Double(cx + Math.cos(a) * r * 0.5, cy + Math.sin(a) * r * 0.5, cx + Math.cos(a) * r, cy + Math.sin(a) * r));
                }
                Ellipse2D face = new Ellipse2D.Double(cx - r * 0.6, cy - r * 0.6, r * 1.2, r * 1.2);
                fillOutlined(g, face, new Color(0xFFCCBC), new Color(0x8D6E63));
                drawHorns(g, t, cx, cy - r * 0.5, size * 0.6);
                drawFace(g, t, cx, cy, size * 0.75, true, time);
            }
        }
        if (boss) drawCrown(g, cx, cy - size * 0.62, size * 0.32);
    }

    private static void drawHorns(Graphics2D g, MonsterType t, double cx, double topY, double size) {
        if (t.horns <= 0) return;
        Color horn = new Color(0xFFF8E1);
        Color line = new Color(0xBCAAA4);
        double[] xs = t.horns == 1 ? new double[] {0} : new double[] {-0.22, 0.22};
        for (double dx : xs) {
            GeneralPath h = new GeneralPath();
            h.moveTo(cx + dx * size - size * 0.08, topY + size * 0.05);
            h.quadTo(cx + dx * size * 1.4, topY - size * 0.2, cx + dx * size * 1.5, topY - size * 0.28);
            h.quadTo(cx + dx * size + size * 0.02, topY - size * 0.05, cx + dx * size + size * 0.08, topY + size * 0.05);
            h.closePath();
            fillOutlined(g, h, horn, line);
        }
    }

    private static void drawFace(Graphics2D g, MonsterType t, double cx, double cy, double size, boolean angry, double time) {
        double blink = (time % 4) > 3.85 ? 0.15 : 1.0;
        double eyeR = size * (t.eyes == 1 ? 0.16 : 0.1);
        if (t.eyes == 0) {
            // Безликий — только румянец
        } else if (t.eyes == 1) {
            drawEye(g, cx, cy - size * 0.02, eyeR, blink, t.accent);
        } else {
            for (int side = -1; side <= 1; side += 2) drawEye(g, cx + side * size * 0.17, cy, eyeR, blink, null);
            if (angry) {
                g.setColor(new Color(0x3E2723));
                g.setStroke(new BasicStroke((float) (size * 0.035), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                g.draw(new java.awt.geom.Line2D.Double(cx - size * 0.28, cy - size * 0.17, cx - size * 0.09, cy - size * 0.11));
                g.draw(new java.awt.geom.Line2D.Double(cx + size * 0.28, cy - size * 0.17, cx + size * 0.09, cy - size * 0.11));
            }
        }
        if (t.longNose) {
            GeneralPath nose = new GeneralPath();
            nose.moveTo(cx - size * 0.05, cy + size * 0.06);
            nose.quadTo(cx + size * 0.35, cy + size * 0.02, cx + size * 0.42, cy + size * 0.12);
            nose.quadTo(cx + size * 0.2, cy + size * 0.18, cx - size * 0.05, cy + size * 0.16);
            nose.closePath();
            fillOutlined(g, nose, darker(t.body, 0.9), darker(t.body, 0.5));
        }
        // Румянец
        g.setColor(new Color(255, 138, 128, 120));
        g.fill(new Ellipse2D.Double(cx - size * 0.36, cy + size * 0.1, size * 0.14, size * 0.07));
        g.fill(new Ellipse2D.Double(cx + size * 0.22, cy + size * 0.1, size * 0.14, size * 0.07));
        // Рот
        g.setColor(new Color(0x3E2723));
        g.setStroke(new BasicStroke((float) Math.max(1.5, size * 0.02), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        double my = cy + size * 0.18;
        if (angry) {
            Path2D m = new Path2D.Double();
            m.moveTo(cx - size * 0.1, my);
            m.lineTo(cx - size * 0.05, my - size * 0.03);
            m.lineTo(cx, my);
            m.lineTo(cx + size * 0.05, my - size * 0.03);
            m.lineTo(cx + size * 0.1, my);
            g.draw(m);
            g.setColor(Color.WHITE);
            g.fill(new Arc2D.Double(cx - size * 0.1, my - size * 0.01, size * 0.05, size * 0.06, 180, 180, Arc2D.CHORD));
            g.fill(new Arc2D.Double(cx + size * 0.05, my - size * 0.01, size * 0.05, size * 0.06, 180, 180, Arc2D.CHORD));
        } else {
            g.draw(new Arc2D.Double(cx - size * 0.07, my - size * 0.05, size * 0.14, size * 0.08, 200, 140, Arc2D.OPEN));
        }
    }

    private static void drawEye(Graphics2D g, double x, double y, double r, double blink, Color iris) {
        Ellipse2D white = new Ellipse2D.Double(x - r, y - r * blink, r * 2, r * 2 * blink);
        g.setColor(Color.WHITE);
        g.fill(white);
        g.setColor(new Color(0x3E2723));
        g.setStroke(new BasicStroke(1.5f));
        g.draw(white);
        if (blink < 0.5) return;
        g.setColor(iris != null ? darker(iris, 0.6) : new Color(0x212121));
        g.fill(new Ellipse2D.Double(x - r * 0.55, y - r * 0.55, r * 1.1, r * 1.1));
        g.setColor(Color.WHITE);
        g.fill(new Ellipse2D.Double(x - r * 0.35, y - r * 0.45, r * 0.4, r * 0.4));
    }

    private static void drawCrown(Graphics2D g, double cx, double y, double w) {
        GeneralPath c = new GeneralPath();
        c.moveTo(cx - w / 2, y);
        c.lineTo(cx - w / 2, y - w * 0.35);
        c.lineTo(cx - w / 4, y - w * 0.15);
        c.lineTo(cx, y - w * 0.45);
        c.lineTo(cx + w / 4, y - w * 0.15);
        c.lineTo(cx + w / 2, y - w * 0.35);
        c.lineTo(cx + w / 2, y);
        c.closePath();
        fillOutlined(g, c, new Color(0xFFD54F), new Color(0xF57F17));
        g.setColor(new Color(0xE91E63));
        g.fill(new Ellipse2D.Double(cx - w * 0.06, y - w * 0.2, w * 0.12, w * 0.12));
    }

    // =====================================================================
    // Пейзаж
    // =====================================================================

    /** Дерево сакуры (рисуется один раз в картинку). */
    public static BufferedImage renderTree(int w, int h, Color blossom, long seed) {
        BufferedImage img = new BufferedImage(w, h, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        smooth(g);
        Random r = new Random(seed);
        Color trunk = new Color(0x5D4037);
        g.setColor(trunk);
        GeneralPath tr = new GeneralPath();
        tr.moveTo(w * 0.40, h);
        tr.curveTo(w * 0.45, h * 0.75, w * 0.38, h * 0.55, w * 0.48, h * 0.38);
        tr.lineTo(w * 0.56, h * 0.40);
        tr.curveTo(w * 0.50, h * 0.58, w * 0.58, h * 0.78, w * 0.62, h);
        tr.closePath();
        g.fill(tr);
        g.setStroke(new BasicStroke(w * 0.035f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        double[][] branches = {
            {0.5, 0.42, 0.2, 0.22}, {0.52, 0.42, 0.85, 0.2}, {0.5, 0.5, 0.12, 0.42}, {0.52, 0.48, 0.9, 0.42}, {0.5, 0.4, 0.55, 0.12},
        };
        for (double[] b : branches) {
            g.draw(new java.awt.geom.QuadCurve2D.Double(w * b[0], h * b[1], w * (b[0] + b[2]) / 2, h * (b[1] + b[3]) / 2 - h * 0.05, w * b[2], h * b[3]));
        }
        Color[] pinks = {blossom, lighter(blossom, 0.35), lighter(blossom, 0.6), darker(blossom, 0.92)};
        for (int i = 0; i < 260; i++) {
            double a = r.nextDouble() * Math.PI * 2;
            double d = Math.sqrt(r.nextDouble());
            double px = w * 0.5 + Math.cos(a) * d * w * 0.46;
            double py = h * 0.28 + Math.sin(a) * d * h * 0.22;
            double rad = w * (0.04 + r.nextDouble() * 0.07);
            g.setColor(alpha(pinks[r.nextInt(pinks.length)], 200));
            g.fill(new Ellipse2D.Double(px - rad, py - rad, rad * 2, rad * 2));
        }
        for (int i = 0; i < 40; i++) {
            double a = r.nextDouble() * Math.PI * 2;
            double d = Math.sqrt(r.nextDouble());
            drawBlossom(g, w * 0.5 + Math.cos(a) * d * w * 0.44, h * 0.28 + Math.sin(a) * d * h * 0.2, w * 0.022,
                Color.WHITE, new Color(0xF06292));
        }
        g.dispose();
        return img;
    }

    /** Ворота тории. */
    public static void drawTorii(Graphics2D g, double cx, double baseY, double w, Color color) {
        double h = w * 0.85;
        g.setColor(color);
        g.fill(new java.awt.geom.Rectangle2D.Double(cx - w * 0.36, baseY - h, w * 0.08, h));
        g.fill(new java.awt.geom.Rectangle2D.Double(cx + w * 0.28, baseY - h, w * 0.08, h));
        g.fill(new java.awt.geom.Rectangle2D.Double(cx - w * 0.42, baseY - h * 0.78, w * 0.84, w * 0.06));
        GeneralPath top = new GeneralPath();
        top.moveTo(cx - w * 0.55, baseY - h * 1.02);
        top.quadTo(cx, baseY - h * 0.92, cx + w * 0.55, baseY - h * 1.02);
        top.lineTo(cx + w * 0.5, baseY - h * 0.92);
        top.quadTo(cx, baseY - h * 0.84, cx - w * 0.5, baseY - h * 0.92);
        top.closePath();
        g.fill(top);
        g.setColor(darker(color, 0.4));
        g.fill(new java.awt.geom.Rectangle2D.Double(cx - w * 0.56, baseY - h * 1.06, w * 1.12, w * 0.04));
    }
}
