package sakura;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.FontMetrics;
import java.awt.GradientPaint;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Rectangle;
import java.awt.Shape;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.awt.font.TextLayout;
import java.awt.geom.AffineTransform;
import java.awt.geom.Arc2D;
import java.awt.geom.Ellipse2D;
import java.awt.geom.GeneralPath;
import java.awt.geom.RoundRectangle2D;
import java.awt.image.BufferedImage;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Random;
import javax.swing.JPanel;

/** Поле боя: пейзаж, Сакура и её отряд, ёкай, эффекты, панель навыков. */
public final class BattlePanel extends JPanel implements Game.Listener {

    private final Game game;
    private final Random rnd = new Random();

    private double time;
    private double attackAnim;      // 0..1 — взмах катаны Сакуры
    private double hitFlash;        // вспышка ёкая при ударе
    private double shake;
    private double spawnAnim = 1;   // появление нового ёкая
    private Monster shownMonster;

    private final List<Particle> particles = new ArrayList<>();
    private final List<FloatText> texts = new ArrayList<>();
    private final List<Slash> slashes = new ArrayList<>();
    private final List<Petal> petals = new ArrayList<>();
    private final ArrayDeque<String> messages = new ArrayDeque<>();
    private String currentMessage;
    private double messageTime;

    private BufferedImage treeImage;
    private Color treeColor;
    private int treeHeight;

    // Области для кликов (вычисляются при отрисовке)
    private Rectangle prevBtn = new Rectangle(), nextBtn = new Rectangle(), autoBtn = new Rectangle();
    private final Rectangle[] skillBtns = new Rectangle[4];
    private int mouseX = -1, mouseY = -1;

    /** Вызывается после действий, меняющих магазин (например, смена этапа). */
    public Runnable onChange = () -> { };

    private static final class Particle {
        double x, y, vx, vy, life, max, size;
        Color color;
        boolean coin, smoke;
    }

    private static final class FloatText {
        double x, y, vy, life, max;
        boolean gold;
        String text;
        Color color;
        float size;
    }

    private static final class Slash {
        double x, y, angle, life;
    }

    private static final class Petal {
        double x, y, vy, sway, phase, rot, vr, size;
    }

    public BattlePanel(Game game) {
        this.game = game;
        for (int i = 0; i < 4; i++) skillBtns[i] = new Rectangle();
        setPreferredSize(new Dimension(780, 720));
        setMinimumSize(new Dimension(600, 560));
        setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        game.listener = this;

        MouseAdapter mouse = new MouseAdapter() {
            @Override
            public void mousePressed(MouseEvent e) {
                handlePress(e.getX(), e.getY());
            }

            @Override
            public void mouseMoved(MouseEvent e) {
                mouseX = e.getX();
                mouseY = e.getY();
            }

            @Override
            public void mouseExited(MouseEvent e) {
                mouseX = mouseY = -1;
            }
        };
        addMouseListener(mouse);
        addMouseMotionListener(mouse);
    }

    // ---------- Ввод ----------

    private void handlePress(int x, int y) {
        if (prevBtn.contains(x, y)) {
            game.setStage(game.stage - 1);
            game.autoAdvance = false;
            onChange.run();
            return;
        }
        if (nextBtn.contains(x, y)) {
            game.setStage(game.stage + 1);
            onChange.run();
            return;
        }
        if (autoBtn.contains(x, y)) {
            game.autoAdvance = !game.autoAdvance;
            if (game.autoAdvance && game.stageCleared(game.stage)) game.setStage(game.stage + 1);
            return;
        }
        for (int i = 0; i < 4; i++) {
            if (skillBtns[i].contains(x, y)) {
                useSkill(i);
                return;
            }
        }
        if (y > 95 && y < getHeight() - 80) game.click();
    }

    public void useSkill(int index) {
        Skill s = game.skills.get(index);
        if (!game.isUnlocked(s)) {
            onMessage("«" + s.name + "» откроется на этапе " + s.unlockStage);
        } else if (!s.ready()) {
            onMessage("«" + s.name + "» перезаряжается: " + Fmt.time(s.cooldownLeft));
        } else if (game.useSkill(s)) {
            for (int i = 0; i < 40; i++) {
                Particle p = new Particle();
                p.x = getWidth() * 0.3;
                p.y = getHeight() * 0.6;
                double a = rnd.nextDouble() * Math.PI * 2, sp = 100 + rnd.nextDouble() * 300;
                p.vx = Math.cos(a) * sp;
                p.vy = Math.sin(a) * sp - 120;
                p.life = p.max = 1.2;
                p.size = 6 + rnd.nextDouble() * 6;
                p.color = new Color(0xF48FB1);
                particles.add(p);
            }
        }
    }

    // ---------- События игры ----------

    @Override
    public void onHit(double damage, boolean crit, boolean fromClick) {
        if (fromClick) attackAnim = 1;
        hitFlash = 1;
        shake = crit ? 1 : 0.5;
        double mx = monsterX(), my = monsterY();
        FloatText t = new FloatText();
        t.x = mx + rnd.nextInt(120) - 60;
        t.y = my - 40 + rnd.nextInt(40);
        t.vy = -90;
        t.life = t.max = crit ? 1.2 : 0.9;
        t.text = (crit ? "КРИТ! " : "") + Fmt.num(damage);
        t.color = crit ? new Color(0xFFD54F) : Color.WHITE;
        t.size = crit ? 30 : 22;
        texts.add(t);
        if (texts.size() > 60) texts.remove(0);

        Slash s = new Slash();
        s.x = mx + rnd.nextInt(60) - 30;
        s.y = my + rnd.nextInt(60) - 30;
        s.angle = rnd.nextDouble() * Math.PI;
        s.life = 0.25;
        slashes.add(s);
        if (slashes.size() > 20) slashes.remove(0);
        // Немного лепестков от удара
        for (int i = 0; i < (crit ? 8 : 3); i++) {
            Particle p = new Particle();
            p.x = s.x;
            p.y = s.y;
            double a = rnd.nextDouble() * Math.PI * 2, sp = 80 + rnd.nextDouble() * 200;
            p.vx = Math.cos(a) * sp;
            p.vy = Math.sin(a) * sp - 100;
            p.life = p.max = 0.8;
            p.size = 5 + rnd.nextDouble() * 4;
            p.color = crit ? new Color(0xFFD54F) : new Color(0xF8BBD0);
            particles.add(p);
        }
    }

    @Override
    public void onKill(double gold, boolean boss) {
        double mx = monsterX(), my = monsterY();
        for (int i = 0; i < (boss ? 30 : 12); i++) {
            Particle p = new Particle();
            p.x = mx;
            p.y = my;
            double a = -Math.PI / 2 + (rnd.nextDouble() - 0.5) * 2.2, sp = 200 + rnd.nextDouble() * 250;
            p.vx = Math.cos(a) * sp;
            p.vy = Math.sin(a) * sp;
            p.life = p.max = 1.1;
            p.size = 9 + rnd.nextDouble() * 4;
            p.coin = true;
            particles.add(p);
        }
        for (int i = 0; i < 10; i++) {
            Particle p = new Particle();
            p.x = mx + rnd.nextInt(120) - 60;
            p.y = my + rnd.nextInt(100) - 50;
            p.vx = rnd.nextInt(60) - 30;
            p.vy = -30 - rnd.nextInt(40);
            p.life = p.max = 0.7;
            p.size = 20 + rnd.nextInt(20);
            p.smoke = true;
            particles.add(p);
        }
        if (boss) onMessage("Босс повержен! +" + Fmt.num(gold) + " рё");
        // Не больше трёх надписей о золоте одновременно
        if (texts.stream().filter(ft -> ft.gold).count() >= 3) return;
        FloatText t = new FloatText();
        t.gold = true;
        t.x = mx;
        t.y = my - 120;
        t.vy = -50;
        t.life = t.max = 1.4;
        t.text = "+" + Fmt.num(gold) + " рё";
        t.color = new Color(0xFFC107);
        t.size = boss ? 32 : 24;
        texts.add(t);
    }

    @Override
    public void onMessage(String text) {
        if (messages.size() < 5 && !text.equals(currentMessage) && !messages.contains(text)) messages.add(text);
    }

    // ---------- Анимация ----------

    public void tick(double dt) {
        time += dt;
        attackAnim = Math.max(0, attackAnim - dt * 4);
        hitFlash = Math.max(0, hitFlash - dt * 6);
        shake = Math.max(0, shake - dt * 5);
        if (game.monster != shownMonster) {
            shownMonster = game.monster;
            spawnAnim = 0;
        }
        spawnAnim = Math.min(1, spawnAnim + dt * 4);

        for (Iterator<Particle> it = particles.iterator(); it.hasNext(); ) {
            Particle p = it.next();
            p.life -= dt;
            if (p.life <= 0) {
                it.remove();
                continue;
            }
            if (!p.smoke) p.vy += 600 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            if (p.smoke) p.size += 40 * dt;
        }
        for (Iterator<FloatText> it = texts.iterator(); it.hasNext(); ) {
            FloatText t = it.next();
            t.life -= dt;
            t.y += t.vy * dt;
            if (t.life <= 0) it.remove();
        }
        slashes.removeIf(s -> (s.life -= dt) <= 0);

        // Лепестки / снег / искры
        int w = Math.max(1, getWidth()), h = Math.max(1, getHeight());
        while (petals.size() < 45) {
            Petal p = new Petal();
            p.x = rnd.nextDouble() * w * 1.2;
            p.y = petals.isEmpty() && time < 0.1 ? rnd.nextDouble() * h : -20 - rnd.nextDouble() * h * 0.5;
            p.vy = 30 + rnd.nextDouble() * 50;
            p.sway = 20 + rnd.nextDouble() * 40;
            p.phase = rnd.nextDouble() * 6;
            p.rot = rnd.nextDouble() * 6;
            p.vr = (rnd.nextDouble() - 0.5) * 3;
            p.size = 4 + rnd.nextDouble() * 5;
            petals.add(p);
        }
        Zone.Particles kind = Zone.forStage(game.stage).particles;
        for (Petal p : petals) {
            double dir = kind == Zone.Particles.EMBERS || kind == Zone.Particles.FIREFLIES ? -0.6 : 1;
            p.y += p.vy * dt * dir;
            p.x += (Math.sin(time + p.phase) * p.sway - 25) * dt;
            p.rot += p.vr * dt;
        }
        petals.removeIf(p -> p.y > h + 30 || p.y < -h || p.x < -40);
        if (currentMessage == null && !messages.isEmpty()) {
            currentMessage = messages.poll();
            messageTime = 4;
        }
        if (currentMessage != null && (messageTime -= dt) <= 0) currentMessage = null;
    }

    private double monsterX() {
        return getWidth() * 0.66;
    }

    private double monsterY() {
        return getHeight() * 0.50;
    }

    private double groundY() {
        return getHeight() * 0.74;
    }

    // ---------- Отрисовка ----------

    @Override
    protected void paintComponent(Graphics g0) {
        Graphics2D g = (Graphics2D) g0.create();
        Art.smooth(g);
        int w = getWidth(), h = getHeight();
        Zone zone = Zone.forStage(game.stage);

        drawBackground(g, zone, w, h);
        drawParty(g, w, h);
        drawMonster(g, zone);
        drawEffects(g, zone);
        drawHud(g, w);
        drawMonsterInfo(g);
        drawSkills(g, w, h);
        drawMessage(g, w);
        g.dispose();
    }

    private void drawBackground(Graphics2D g, Zone zone, int w, int h) {
        double gy = groundY();
        g.setPaint(new GradientPaint(0, 0, zone.skyTop, 0, (float) gy, zone.skyBottom));
        g.fillRect(0, 0, w, h);

        if (zone.night) {
            g.setColor(new Color(255, 255, 255, 200));
            Random r = new Random(7);
            for (int i = 0; i < 60; i++) {
                double tw = 0.5 + 0.5 * Math.sin(time * 2 + i);
                g.setColor(new Color(255, 255, 255, (int) (90 + 120 * tw)));
                double sx = r.nextDouble() * w, sy = r.nextDouble() * gy * 0.7;
                g.fill(new Ellipse2D.Double(sx, sy, 2.5, 2.5));
            }
            g.setColor(new Color(0xFFF9C4));
            g.fill(new Ellipse2D.Double(w * 0.78, h * 0.12, 70, 70));
            g.setColor(zone.skyTop);
            g.fill(new Ellipse2D.Double(w * 0.78 + 18, h * 0.12 - 6, 70, 70));
        } else {
            g.setColor(new Color(255, 255, 255, 120));
            g.fill(new Ellipse2D.Double(w * 0.80, h * 0.10, 90, 90));
        }

        // Дальние холмы
        GeneralPath hills = new GeneralPath();
        hills.moveTo(0, gy);
        for (int i = 0; i <= 8; i++) {
            double x = w * i / 8.0;
            double y = gy - 60 - 50 * Math.sin(i * 1.3 + 0.5) - 30 * Math.cos(i * 2.1);
            hills.lineTo(x, y);
        }
        hills.lineTo(w, gy);
        hills.closePath();
        g.setColor(Art.alpha(zone.hills, 170));
        g.fill(hills);

        if (zone.torii) Art.drawTorii(g, w * 0.86, gy - 8, 110, new Color(0xD84343));

        if (zone.night) {
            // Ряд фонариков
            for (int i = 0; i < 7; i++) {
                double lx = w * (0.35 + i * 0.1), ly = gy - 210 + Math.sin(i) * 10;
                g.setColor(new Color(255, 183, 77, 60));
                g.fill(new Ellipse2D.Double(lx - 22, ly - 22, 44, 44));
                g.setColor(new Color(0xFF7043));
                g.fill(new Ellipse2D.Double(lx - 10, ly - 13, 20, 26));
            }
        }

        // Дерево сакуры
        int treeH = (int) (h * 0.75);
        Color blossom = zone.particles == Zone.Particles.SNOW ? new Color(0xF8BBD0) : new Color(0xF48FB1);
        if (treeImage == null || treeHeight != treeH || !blossom.equals(treeColor)) {
            treeImage = Art.renderTree((int) (treeH * 0.8), treeH, blossom, 42);
            treeHeight = treeH;
            treeColor = blossom;
        }
        g.drawImage(treeImage, (int) (-treeH * 0.22), (int) (gy - treeH + 10), null);

        // Земля
        g.setPaint(new GradientPaint(0, (float) gy, zone.ground, 0, h, Art.darker(zone.ground, 0.85)));
        g.fill(new java.awt.geom.Rectangle2D.Double(0, gy, w, h - gy));
        g.setColor(Art.darker(zone.ground, 0.8));
        g.setStroke(new BasicStroke(2f));
        g.drawLine(0, (int) gy, w, (int) gy);

        // Частицы погоды (позади персонажей)
        for (Petal p : petals) drawPetal(g, zone, p, 150);
    }

    private void drawPetal(Graphics2D g, Zone zone, Petal p, int alpha) {
        switch (zone.particles) {
            case SNOW -> {
                g.setColor(new Color(255, 255, 255, alpha + 60));
                g.fill(new Ellipse2D.Double(p.x, p.y, p.size, p.size));
            }
            case EMBERS -> {
                g.setColor(new Color(255, 138, 50, alpha));
                g.fill(new Ellipse2D.Double(p.x, p.y, p.size * 0.6, p.size * 0.6));
            }
            case FIREFLIES -> {
                double glow = 0.5 + 0.5 * Math.sin(time * 3 + p.phase);
                g.setColor(new Color(255, 241, 118, (int) (60 + 140 * glow)));
                g.fill(new Ellipse2D.Double(p.x, p.y, p.size * 0.7, p.size * 0.7));
            }
            default -> Art.drawPetal(g, p.x, p.y, p.size, p.rot, new Color(248, 160, 190, alpha));
        }
    }

    private void drawParty(Graphics2D g, int w, int h) {
        double gy = groundY();
        // Героини отряда — маленькие, в два ряда позади Сакуры
        int idx = 0;
        for (Heroine hr : game.heroines) {
            if (hr.clicker || hr.level <= 0) continue;
            int row = idx % 2;
            int col = idx / 2;
            double x = w * 0.08 + col * 52 + row * 26;
            double y = gy + 18 + row * 40;
            double hop = Math.max(0, Math.sin(time * 4 + idx * 1.3)) * 6;
            double angle = 0.5 + Math.sin(time * 4 + idx * 1.3) * 0.5;
            Art.drawChibi(g, hr.look, x, y - hop, 62, angle, time + idx);
            idx++;
        }
        // Сакура
        double swing = attackAnim > 0 ? (-0.4 + (1 - attackAnim) * 2.4) : 0.55 + Math.sin(time * 1.5) * 0.08;
        double sx = w * 0.30 + attackAnim * 30;
        double bob = Math.sin(time * 2) * 2;
        Art.drawChibi(g, game.outfit().sakuraLook(), sx, gy + 6 + bob, 190, swing, time);
    }

    private void drawMonster(Graphics2D g, Zone zone) {
        Monster m = game.monster;
        double mx = monsterX(), my = monsterY();
        double size = m.boss ? 230 : 170;
        double sx = shake > 0 ? (rnd.nextDouble() - 0.5) * 14 * shake : 0;
        double bounce = Math.sin(time * 3) * 5;
        double sc = 0.3 + 0.7 * easeOutBack(spawnAnim);
        double squash = 1 - hitFlash * 0.08;

        AffineTransform saved = g.getTransform();
        g.translate(mx + sx, my + bounce);
        g.scale(sc / squash, sc * squash);
        Art.drawMonster(g, m.type, 0, 0, size, time, m.boss);
        if (hitFlash > 0) {
            g.setColor(new Color(255, 255, 255, (int) (55 * hitFlash)));
            g.fill(new Ellipse2D.Double(-size * 0.55, -size * 0.55, size * 1.1, size * 1.1));
        }
        g.setTransform(saved);
    }

    private static double easeOutBack(double t) {
        double c1 = 1.70158, c3 = c1 + 1;
        return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
    }

    private void drawEffects(Graphics2D g, Zone zone) {
        for (Slash s : slashes) {
            double a = s.life / 0.25;
            AffineTransform saved = g.getTransform();
            g.translate(s.x, s.y);
            g.rotate(s.angle);
            g.setColor(new Color(255, 255, 255, (int) (230 * a)));
            g.setStroke(new BasicStroke((float) (8 * a), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
            g.draw(new Arc2D.Double(-90, -50, 180, 100, 20, 140, Arc2D.OPEN));
            g.setColor(new Color(244, 143, 177, (int) (200 * a)));
            g.setStroke(new BasicStroke((float) (3 * a)));
            g.draw(new Arc2D.Double(-80, -40, 160, 90, 25, 130, Arc2D.OPEN));
            g.setTransform(saved);
        }
        for (Particle p : particles) {
            double a = Math.max(0, p.life / p.max);
            if (p.coin) {
                g.setColor(new Color(255, 193, 7, (int) (255 * a)));
                g.fill(new Ellipse2D.Double(p.x - p.size / 2, p.y - p.size / 2, p.size, p.size));
                g.setColor(new Color(255, 236, 179, (int) (255 * a)));
                g.fill(new Ellipse2D.Double(p.x - p.size / 4, p.y - p.size / 4, p.size / 2.5, p.size / 2.5));
            } else if (p.smoke) {
                g.setColor(new Color(255, 255, 255, (int) (160 * a)));
                g.fill(new Ellipse2D.Double(p.x - p.size / 2, p.y - p.size / 2, p.size, p.size));
            } else {
                Art.drawPetal(g, p.x, p.y, p.size, p.life * 8, Art.alpha(p.color, (int) (255 * a)));
            }
        }
        for (FloatText t : texts) {
            double a = Math.max(0, Math.min(1, t.life / t.max * 1.5));
            outlined(g, t.text, t.x, t.y, UI.font(Font.BOLD, (int) t.size), Art.alpha(t.color, (int) (255 * a)),
                new Color(74, 44, 58, (int) (230 * a)), true);
        }
    }

    private void drawHud(Graphics2D g, int w) {
        // Полупрозрачная плашка сверху
        g.setColor(new Color(255, 255, 255, 150));
        g.fill(new RoundRectangle2D.Double(10, 10, w - 20, 80, 22, 22));

        // Золото
        drawCoin(g, 36, 38, 14);
        outlined(g, Fmt.num(game.gold) + " рё", 58, 48, UI.font(Font.BOLD, 26), new Color(0xF2A900), new Color(0x5D4037), false);
        g.setFont(UI.font(Font.PLAIN, 13));
        g.setColor(UI.TEXT);
        g.drawString("Клик: " + Fmt.num(game.clickDamage()) + "   •   Отряд: " + Fmt.num(game.dps()) + " / с", 24, 76);

        // Переключатель этапов
        int cx = w - 190;
        prevBtn = new Rectangle(cx - 112, 24, 34, 34);
        nextBtn = new Rectangle(cx + 78, 24, 34, 34);
        drawArrow(g, prevBtn, true, game.stage > 1);
        drawArrow(g, nextBtn, false, game.stage < game.maxStage);
        String stageText = "Этап " + game.stage;
        Font f = UI.font(Font.BOLD, 22);
        g.setFont(f);
        FontMetrics fm = g.getFontMetrics();
        outlined(g, stageText, cx - 17, 50, f, Color.WHITE, UI.PRIMARY_DARK, true);
        g.setFont(UI.font(Font.PLAIN, 12));
        g.setColor(UI.TEXT);
        String zoneName = Zone.nameForStage(game.stage);
        fm = g.getFontMetrics();
        g.drawString(zoneName, cx - 17 - fm.stringWidth(zoneName) / 2, 76);

        // Авто-продвижение
        autoBtn = new Rectangle(w - 120, 26, 96, 30);
        boolean hover = autoBtn.contains(mouseX, mouseY);
        g.setColor(game.autoAdvance ? UI.PRIMARY : (hover ? new Color(0xD8C4CE) : new Color(0xCDB8C2)));
        g.fill(new RoundRectangle2D.Double(autoBtn.x, autoBtn.y, autoBtn.width, autoBtn.height, 30, 30));
        g.setColor(Color.WHITE);
        g.setFont(UI.font(Font.BOLD, 12));
        String autoText = game.autoAdvance ? "Вперёд: вкл" : "Вперёд: выкл";
        fm = g.getFontMetrics();
        g.drawString(autoText, autoBtn.x + (autoBtn.width - fm.stringWidth(autoText)) / 2, autoBtn.y + 20);
        g.setColor(UI.MUTED);
        g.setFont(UI.font(Font.PLAIN, 11));
        g.drawString("рекорд: " + game.bestStage, autoBtn.x + 20, autoBtn.y + 48);
    }

    private void drawArrow(Graphics2D g, Rectangle r, boolean left, boolean enabled) {
        boolean hover = enabled && r.contains(mouseX, mouseY);
        g.setColor(enabled ? (hover ? UI.PRIMARY_DARK : UI.PRIMARY) : UI.DISABLED);
        g.fill(new Ellipse2D.Double(r.x, r.y, r.width, r.height));
        GeneralPath p = new GeneralPath();
        double cx = r.getCenterX(), cy = r.getCenterY();
        if (left) {
            p.moveTo(cx + 5, cy - 8);
            p.lineTo(cx - 6, cy);
            p.lineTo(cx + 5, cy + 8);
        } else {
            p.moveTo(cx - 5, cy - 8);
            p.lineTo(cx + 6, cy);
            p.lineTo(cx - 5, cy + 8);
        }
        p.closePath();
        g.setColor(Color.WHITE);
        g.fill(p);
    }

    private void drawCoin(Graphics2D g, double cx, double cy, double r) {
        g.setColor(new Color(0xFFC107));
        g.fill(new Ellipse2D.Double(cx - r, cy - r, r * 2, r * 2));
        g.setColor(new Color(0xF57F17));
        g.setStroke(new BasicStroke(2f));
        g.draw(new Ellipse2D.Double(cx - r, cy - r, r * 2, r * 2));
        g.setColor(new Color(0xF57F17));
        g.fill(new java.awt.geom.Rectangle2D.Double(cx - r * 0.3, cy - r * 0.3, r * 0.6, r * 0.6));
    }

    private void drawMonsterInfo(Graphics2D g) {
        Monster m = game.monster;
        double cx = monsterX();
        double top = 112;
        int barW = 300;
        String name = m.type.name + (m.boss ? " — БОСС" : "");
        outlined(g, name, cx, top, UI.font(Font.BOLD, 20), m.boss ? new Color(0xFFD54F) : Color.WHITE, UI.TEXT, true);

        double frac = Math.max(0, m.hp / m.maxHp);
        RoundRectangle2D back = new RoundRectangle2D.Double(cx - barW / 2.0, top + 10, barW, 18, 18, 18);
        g.setColor(new Color(74, 44, 58, 170));
        g.fill(back);
        g.setPaint(new GradientPaint((float) (cx - barW / 2.0), 0, new Color(0xFF6F91), (float) (cx + barW / 2.0), 0, new Color(0xE53950)));
        g.fill(new RoundRectangle2D.Double(cx - barW / 2.0 + 2, top + 12, (barW - 4) * frac, 14, 14, 14));
        g.setFont(UI.font(Font.BOLD, 11));
        g.setColor(Color.WHITE);
        String hpText = Fmt.num(Math.max(0, m.hp)) + " / " + Fmt.num(m.maxHp);
        FontMetrics fm = g.getFontMetrics();
        g.drawString(hpText, (int) (cx - fm.stringWidth(hpText) / 2.0), (int) top + 23);

        double y2 = top + 44;
        if (m.boss) {
            double tf = Math.max(0, game.bossTimeLeft / game.bossTime());
            g.setColor(new Color(74, 44, 58, 150));
            g.fill(new RoundRectangle2D.Double(cx - barW / 2.0, y2 - 10, barW, 12, 12, 12));
            g.setColor(tf < 0.3 ? new Color(0xFF5252) : new Color(0xFFB300));
            g.fill(new RoundRectangle2D.Double(cx - barW / 2.0 + 2, y2 - 8, (barW - 4) * tf, 8, 8, 8));
            outlined(g, "Время: " + String.format("%.1f", Math.max(0, game.bossTimeLeft)) + " с", cx, y2 + 18,
                UI.font(Font.BOLD, 14), Color.WHITE, UI.TEXT, true);
        } else {
            String progress = game.stageCleared(game.stage)
                ? "Этап пройден • ёкаи: " + game.kills
                : "Ёкаи: " + game.kills + " / " + Game.MONSTERS_PER_STAGE;
            outlined(g, progress, cx, y2 + 2, UI.font(Font.BOLD, 14), Color.WHITE, UI.TEXT, true);
            // Точки прогресса
            if (!game.stageCleared(game.stage)) {
                for (int i = 0; i < Game.MONSTERS_PER_STAGE; i++) {
                    double dx = cx - 72 + i * 16;
                    g.setColor(i < game.kills ? UI.PRIMARY : new Color(255, 255, 255, 170));
                    g.fill(new Ellipse2D.Double(dx - 5, y2 + 10, 10, 10));
                }
            }
        }
    }

    private void drawSkills(Graphics2D g, int w, int h) {
        int bw = 150, bh = 56, gap = 10;
        int total = 4 * bw + 3 * gap;
        int x0 = (w - total) / 2, y = h - bh - 14;
        for (int i = 0; i < 4; i++) {
            Skill s = game.skills.get(i);
            Rectangle r = new Rectangle(x0 + i * (bw + gap), y, bw, bh);
            skillBtns[i] = r;
            boolean unlocked = game.isUnlocked(s);
            boolean hover = r.contains(mouseX, mouseY);
            RoundRectangle2D rr = new RoundRectangle2D.Double(r.x, r.y, r.width, r.height, 18, 18);
            Color base = !unlocked ? new Color(0xBDA9B3) : s.active() ? new Color(0xFFB300) : UI.PRIMARY;
            if (hover && unlocked) base = Art.lighter(base, 0.12);
            g.setColor(new Color(0, 0, 0, 40));
            g.fill(new RoundRectangle2D.Double(r.x, r.y + 3, r.width, r.height, 18, 18));
            g.setColor(base);
            g.fill(rr);

            if (unlocked && !s.ready() && !s.active()) {
                double frac = s.cooldownLeft / s.cooldown;
                Shape clip = g.getClip();
                g.clip(rr);
                g.setColor(new Color(60, 30, 45, 120));
                g.fill(new java.awt.geom.Rectangle2D.Double(r.x, r.y, r.width * frac, r.height));
                g.setClip(clip);
            }
            g.setColor(Color.WHITE);
            g.setFont(UI.font(Font.BOLD, 13));
            g.drawString(s.key + ". " + s.name, r.x + 10, r.y + 22);
            g.setFont(UI.font(Font.PLAIN, 11));
            String sub;
            if (!unlocked) sub = "Откроется на этапе " + s.unlockStage;
            else if (s.active()) sub = "Активно: " + Fmt.time(s.activeLeft + 0.99);
            else if (!s.ready()) sub = "Ждать: " + Fmt.time(s.cooldownLeft + 0.99);
            else sub = s.description;
            drawFitted(g, sub, r.x + 10, r.y + 42, r.width - 16);

            if (hover) {
                // Подсказка над кнопкой
                String tip = s.name + ": " + s.description + " (перезарядка " + Fmt.time(s.cooldown) + ")";
                g.setFont(UI.font(Font.PLAIN, 12));
                FontMetrics fm = g.getFontMetrics();
                int tw = fm.stringWidth(tip) + 20;
                int tx = Math.max(8, Math.min(w - tw - 8, r.x + r.width / 2 - tw / 2));
                g.setColor(new Color(74, 44, 58, 230));
                g.fill(new RoundRectangle2D.Double(tx, r.y - 36, tw, 26, 12, 12));
                g.setColor(Color.WHITE);
                g.drawString(tip, tx + 10, r.y - 18);
            }
        }
    }

    private void drawFitted(Graphics2D g, String text, int x, int y, int maxW) {
        FontMetrics fm = g.getFontMetrics();
        String t = text;
        while (fm.stringWidth(t) > maxW && t.length() > 3) t = t.substring(0, t.length() - 2);
        if (!t.equals(text)) t = t.substring(0, Math.max(0, t.length() - 1)) + "…";
        g.drawString(t, x, y);
    }

    private void drawMessage(Graphics2D g, int w) {
        if (currentMessage == null) return;
        double a = Math.min(1, messageTime * 2);
        Font f = UI.font(Font.BOLD, 15);
        g.setFont(f);
        FontMetrics fm = g.getFontMetrics();
        int tw = fm.stringWidth(currentMessage) + 36;
        int x = (w - tw) / 2, y = getHeight() - 122;
        if (tw > w - 20) {
            x = 10;
            tw = w - 20;
        }
        g.setColor(new Color(74, 44, 58, (int) (220 * a)));
        g.fill(new RoundRectangle2D.Double(x, y, tw, 34, 20, 20));
        Art.drawBlossom(g, x + 18, y + 17, 8, new Color(255, 255, 255, (int) (255 * a)), UI.PRIMARY);
        g.setColor(new Color(255, 255, 255, (int) (255 * a)));
        g.drawString(currentMessage, x + 30, y + 22);
    }

    /** Текст с обводкой. */
    private void outlined(Graphics2D g, String text, double x, double y, Font font, Color fill, Color outline, boolean centered) {
        TextLayout tl = new TextLayout(text, font, g.getFontRenderContext());
        double tx = centered ? x - tl.getAdvance() / 2 : x;
        Shape shape = tl.getOutline(AffineTransform.getTranslateInstance(tx, y));
        g.setColor(outline);
        g.setStroke(new BasicStroke(Math.max(3f, font.getSize() / 6f), BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        g.draw(shape);
        g.setColor(fill);
        g.fill(shape);
    }
}
