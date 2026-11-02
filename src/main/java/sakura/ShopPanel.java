package sakura;

import java.awt.BorderLayout;
import java.awt.CardLayout;
import java.awt.Color;
import java.awt.Component;
import java.awt.Dimension;
import java.awt.FlowLayout;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.GridLayout;
import java.awt.Rectangle;
import java.awt.geom.Ellipse2D;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;
import javax.swing.BorderFactory;
import javax.swing.Box;
import javax.swing.BoxLayout;
import javax.swing.JComponent;
import javax.swing.JLabel;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.ScrollPaneConstants;
import javax.swing.Scrollable;
import javax.swing.SwingConstants;

/** Правая панель: вкладки «Отряд», «Наряды», «Перерождение», «Статистика». */
public final class ShopPanel extends JPanel {

    private final Game game;
    private final List<Runnable> updaters = new ArrayList<>();
    private final CardLayout cards = new CardLayout();
    private final JPanel content = new JPanel(cards);
    private final List<UI.Button> tabButtons = new ArrayList<>();
    private static final String[] TABS = {"Отряд", "Наряды", "Перерождение", "Статистика"};
    private static final int[] BUY_AMOUNTS = {1, 10, 25, 100, 0};

    /** Вызывается, когда нужно сохранить игру (после покупки). */
    public Runnable onPurchase = () -> { };

    public ShopPanel(Game game) {
        super(new BorderLayout());
        this.game = game;
        setBackground(UI.BG);
        setPreferredSize(new Dimension(450, 720));
        setBorder(BorderFactory.createMatteBorder(0, 2, 0, 0, UI.CARD_BORDER));

        JPanel tabBar = new JPanel(new GridLayout(1, TABS.length, 6, 0));
        tabBar.setOpaque(false);
        tabBar.setBorder(BorderFactory.createEmptyBorder(12, 10, 8, 10));
        for (int i = 0; i < TABS.length; i++) {
            UI.Button b = new UI.Button(TABS[i]);
            b.setMargin(new java.awt.Insets(2, 2, 2, 2));
            b.setFont(UI.font(Font.BOLD, 12));
            b.setPreferredSize(new Dimension(10, 34));
            final int idx = i;
            b.addActionListener(e -> selectTab(idx));
            tabButtons.add(b);
            tabBar.add(b);
        }
        add(tabBar, BorderLayout.NORTH);

        content.setOpaque(false);
        content.add(scroll(buildHeroinesPage()), TABS[0]);
        content.add(scroll(buildOutfitsPage()), TABS[1]);
        content.add(scroll(buildRebirthPage()), TABS[2]);
        content.add(scroll(buildStatsPage()), TABS[3]);
        add(content, BorderLayout.CENTER);
        selectTab(0);
    }

    private void selectTab(int idx) {
        cards.show(content, TABS[idx]);
        for (int i = 0; i < tabButtons.size(); i++) {
            tabButtons.get(i).setColor(i == idx ? UI.PRIMARY_DARK : new Color(0xEDA3BF));
        }
    }

    public void refresh() {
        for (Runnable r : updaters) r.run();
    }

    // ---------- Общие элементы ----------

    /** Панель, которая всегда подстраивается под ширину области прокрутки. */
    private static final class WidthTrackingPanel extends JPanel implements Scrollable {
        WidthTrackingPanel() {
            super(new BorderLayout());
        }

        @Override
        public Dimension getPreferredScrollableViewportSize() {
            return getPreferredSize();
        }

        @Override
        public int getScrollableUnitIncrement(Rectangle r, int orientation, int direction) {
            return 24;
        }

        @Override
        public int getScrollableBlockIncrement(Rectangle r, int orientation, int direction) {
            return r.height - 24;
        }

        @Override
        public boolean getScrollableTracksViewportWidth() {
            return true;
        }

        @Override
        public boolean getScrollableTracksViewportHeight() {
            return false;
        }
    }

    private JScrollPane scroll(JPanel page) {
        JPanel wrapper = new WidthTrackingPanel();
        wrapper.setOpaque(false);
        wrapper.add(page, BorderLayout.NORTH);
        JScrollPane sp = new JScrollPane(wrapper, ScrollPaneConstants.VERTICAL_SCROLLBAR_AS_NEEDED,
            ScrollPaneConstants.HORIZONTAL_SCROLLBAR_NEVER);
        sp.setBorder(BorderFactory.createEmptyBorder());
        sp.getViewport().setBackground(UI.BG);
        sp.getVerticalScrollBar().setUnitIncrement(24);
        return sp;
    }

    private JPanel page() {
        JPanel p = new JPanel();
        p.setLayout(new BoxLayout(p, BoxLayout.Y_AXIS));
        p.setOpaque(false);
        p.setBorder(BorderFactory.createEmptyBorder(0, 10, 12, 10));
        return p;
    }

    private void addRow(JPanel page, JComponent row) {
        row.setAlignmentX(Component.LEFT_ALIGNMENT);
        page.add(row);
        page.add(Box.createVerticalStrut(8));
    }

    private JPanel column() {
        JPanel p = new JPanel();
        p.setLayout(new BoxLayout(p, BoxLayout.Y_AXIS));
        p.setOpaque(false);
        return p;
    }

    /** Портрет героини в кружке. */
    private static final class Portrait extends JComponent {
        private final Supplier<Look> look;
        private final Supplier<Boolean> dim;

        Portrait(Supplier<Look> look, Supplier<Boolean> dim) {
            this.look = look;
            this.dim = dim;
            setPreferredSize(new Dimension(70, 70));
            setMinimumSize(new Dimension(70, 70));
            setMaximumSize(new Dimension(70, 70));
        }

        @Override
        protected void paintComponent(Graphics g0) {
            Graphics2D g = (Graphics2D) g0.create();
            Art.smooth(g);
            Ellipse2D circle = new Ellipse2D.Double(2, 2, 66, 66);
            g.setColor(new Color(0xFCE4EC));
            g.fill(circle);
            g.clip(circle);
            Art.drawChibi(g, look.get(), 35, 104, 105, 0.6, 0);
            if (dim.get()) {
                g.setColor(new Color(255, 240, 245, 150));
                g.fill(circle);
            }
            g.dispose();
        }
    }

    // ---------- Отряд ----------

    private JPanel buildHeroinesPage() {
        JPanel page = page();

        UI.Card amountCard = new UI.Card(new BorderLayout());
        amountCard.add(UI.label("Покупать уровни:", Font.BOLD, 13, UI.TEXT), BorderLayout.WEST);
        JPanel amounts = new JPanel(new FlowLayout(FlowLayout.RIGHT, 4, 0));
        amounts.setOpaque(false);
        for (int amount : BUY_AMOUNTS) {
            UI.Button b = new UI.Button(amount == 0 ? "Макс" : "×" + amount);
            b.setMargin(new java.awt.Insets(4, 8, 4, 8));
            b.addActionListener(e -> {
                game.buyAmount = amount;
                refresh();
            });
            amounts.add(b);
            updaters.add(() -> b.setColor(game.buyAmount == amount ? UI.PRIMARY_DARK : new Color(0xEDA3BF)));
        }
        amountCard.add(amounts, BorderLayout.EAST);
        addRow(page, amountCard);

        for (Heroine h : game.heroines) addRow(page, heroineRow(h));
        return page;
    }

    private JComponent heroineRow(Heroine h) {
        UI.Card card = new UI.Card(new BorderLayout(10, 0));
        card.add(new Portrait(() -> h.clicker ? game.outfit().sakuraLook() : h.look, () -> !h.clicker && h.level == 0), BorderLayout.WEST);

        JPanel info = column();
        JLabel name = UI.label(h.name, Font.BOLD, 15, UI.TEXT);
        JLabel title = UI.label(h.title, Font.PLAIN, 12, UI.MUTED);
        JLabel stats = UI.label("", Font.PLAIN, 12, UI.PRIMARY_DARK);
        JLabel next = UI.label("", Font.PLAIN, 11, UI.MUTED);
        info.add(name);
        info.add(title);
        info.add(Box.createVerticalStrut(3));
        info.add(stats);
        info.add(next);
        card.add(info, BorderLayout.CENTER);

        JPanel buttons = column();
        UI.Button buy = new UI.Button("");
        UI.Button upgrade = new UI.Button("");
        upgrade.setColor(UI.SOUL);
        for (UI.Button b : new UI.Button[] {buy, upgrade}) {
            Dimension d = new Dimension(132, 40);
            b.setPreferredSize(d);
            b.setMaximumSize(d);
            b.setMinimumSize(d);
        }
        buttons.add(buy);
        buttons.add(Box.createVerticalStrut(6));
        buttons.add(upgrade);
        card.add(buttons, BorderLayout.EAST);

        buy.addActionListener(e -> {
            if (game.buy(h)) {
                onPurchase.run();
                refresh();
            }
        });
        upgrade.addActionListener(e -> {
            if (game.buyUpgrade(h)) {
                onPurchase.run();
                refresh();
            }
        });

        updaters.add(() -> {
            boolean visible = game.isVisible(h);
            if (card.isVisible() != visible) card.setVisible(visible);
            if (!visible) return;
            UI.setText(name, h.name + "   ур. " + h.level);
            double soulEtc = game.soulMult() * game.collectionMult();
            if (h.clicker) {
                UI.setText(stats, "Урон клика: " + Fmt.num(game.clickDamage()));
            } else if (h.level == 0) {
                UI.setText(stats, "Урон: " + Fmt.num(h.baseDps * h.multiplier() * soulEtc) + " / с за уровень");
            } else {
                UI.setText(stats, "Урон: " + Fmt.num(h.rawDps() * soulEtc * game.outfit().dpsMult) + " / с   (×" + Fmt.num(h.multiplier()) + ")");
            }
            int n = game.amountFor(h);
            double cost = h.costFor(n);
            String verb = h.clicker ? "Тренировать" : (h.level == 0 ? "Нанять" : "Улучшить");
            UI.setText(buy, "<html><center>" + verb + (n > 1 ? " +" + n : "") + "<br>" + Fmt.num(cost) + " рё</center></html>");
            buy.setEnabled(game.gold >= cost);

            if (h.canUpgrade()) {
                if (!upgrade.isVisible()) upgrade.setVisible(true);
                UI.setText(upgrade, "<html><center>Свиток силы ×2<br>" + Fmt.num(h.upgradeCost()) + " рё</center></html>");
                upgrade.setEnabled(game.gold >= h.upgradeCost());
                UI.setText(next, "Доступен свиток силы!");
            } else {
                if (upgrade.isVisible()) upgrade.setVisible(false);
                UI.setText(next, h.hasMoreUpgrades()
                    ? "Свиток силы ×2 на уровне " + h.nextMilestone()
                    : "Все свитки силы изучены");
            }
            card.setColors(UI.CARD, h.canUpgrade() && game.gold >= h.upgradeCost() ? UI.SOUL : UI.CARD_BORDER);
        });
        return card;
    }

    // ---------- Наряды ----------

    private JPanel buildOutfitsPage() {
        JPanel page = page();
        UI.Card head = new UI.Card(new BorderLayout());
        JLabel summary = UI.label("", Font.BOLD, 14, UI.TEXT);
        head.add(summary, BorderLayout.NORTH);
        head.add(UI.label("<html>Наряды выпадают с боссов. Надетый наряд даёт свой бонус, а каждый собранный — ещё +5% ко всему урону.</html>",
            Font.PLAIN, 12, UI.MUTED), BorderLayout.CENTER);
        head.setPreferredSize(new Dimension(400, 78));
        addRow(page, head);
        updaters.add(() -> {
            long owned = game.outfits.stream().filter(o -> o.owned).count();
            UI.setText(summary, "Гардероб: " + owned + " из " + game.outfits.size()
                + "   (бонус коллекции ×" + Fmt.num(game.collectionMult()) + ")");
        });

        for (Outfit o : game.outfits) {
            UI.Card card = new UI.Card(new BorderLayout(10, 0));
            card.add(new Portrait(o::sakuraLook, () -> !o.owned), BorderLayout.WEST);
            JPanel info = column();
            info.add(UI.label(o.name, Font.BOLD, 15, UI.TEXT));
            info.add(UI.label(o.bonusText, Font.PLAIN, 12, UI.PRIMARY_DARK));
            JLabel status = UI.label("", Font.PLAIN, 11, UI.MUTED);
            info.add(status);
            card.add(info, BorderLayout.CENTER);
            UI.Button wear = new UI.Button("Надеть");
            wear.setPreferredSize(new Dimension(110, 36));
            JPanel bw = new JPanel(new GridLayout(1, 1));
            bw.setOpaque(false);
            bw.setBorder(BorderFactory.createEmptyBorder(14, 0, 14, 0));
            bw.add(wear);
            card.add(bw, BorderLayout.EAST);
            wear.addActionListener(e -> {
                if (game.wear(o)) {
                    onPurchase.run();
                    refresh();
                }
            });
            updaters.add(() -> {
                boolean worn = game.outfitId.equals(o.id);
                if (!o.owned) {
                    UI.setText(wear, "Закрыто");
                    wear.setEnabled(false);
                    UI.setText(status, "Победите босса этапа " + o.unlockStage);
                } else if (worn) {
                    UI.setText(wear, "Надето");
                    wear.setEnabled(false);
                    UI.setText(status, "Сейчас на Сакуре");
                } else {
                    UI.setText(wear, "Надеть");
                    wear.setEnabled(true);
                    UI.setText(status, "В гардеробе");
                }
                card.setColors(worn ? new Color(0xFFF7FA) : UI.CARD, worn ? UI.PRIMARY : UI.CARD_BORDER);
            });
            addRow(page, card);
        }
        return page;
    }

    // ---------- Перерождение ----------

    private JPanel buildRebirthPage() {
        JPanel page = page();
        UI.Card main = new UI.Card(new BorderLayout(0, 8));
        JPanel text = column();
        text.add(UI.label("Перерождение под сакурой", Font.BOLD, 16, UI.TEXT));
        text.add(UI.label("<html>Начните путь заново с этапа 1 и получите <b>души сакуры</b>. Каждая душа навсегда даёт "
            + "+10% ко всему урону. Наряды, талисманы и рекорды сохраняются. Доступно с этапа "
            + Game.REBIRTH_MIN_STAGE + ".</html>", Font.PLAIN, 12, UI.MUTED));
        text.add(Box.createVerticalStrut(6));
        JLabel soulsLabel = UI.label("", Font.BOLD, 14, UI.SOUL);
        JLabel pendingLabel = UI.label("", Font.PLAIN, 12, UI.TEXT);
        text.add(soulsLabel);
        text.add(pendingLabel);
        main.add(text, BorderLayout.CENTER);
        UI.Button rebirth = new UI.Button("");
        rebirth.setColor(UI.SOUL);
        rebirth.setPreferredSize(new Dimension(300, 42));
        main.add(rebirth, BorderLayout.SOUTH);
        main.setPreferredSize(new Dimension(400, 200));
        addRow(page, main);

        rebirth.addActionListener(e -> {
            long gain = game.pendingSouls();
            int answer = JOptionPane.showConfirmDialog(this,
                "Получить " + gain + " душ сакуры и начать путь с этапа 1?\nЗолото и уровни героинь будут сброшены.",
                "Перерождение", JOptionPane.OK_CANCEL_OPTION);
            if (answer == JOptionPane.OK_OPTION) {
                long got = game.rebirth();
                onPurchase.run();
                refresh();
                game.listener.onMessage("Перерождение! +" + got + " душ сакуры");
            }
        });
        updaters.add(() -> {
            UI.setText(soulsLabel, "Души сакуры: " + game.souls + "  (свободно " + game.availableSouls()
                + ")  •  урон ×" + Fmt.num(game.soulMult()));
            long pending = game.pendingSouls();
            UI.setText(pendingLabel, pending > 0
                ? "Сейчас вы получите: +" + pending + " душ (этап " + game.maxStage + ")"
                : "Дойдите до этапа " + Game.REBIRTH_MIN_STAGE + " (сейчас " + game.maxStage + ")");
            UI.setText(rebirth, pending > 0 ? "Переродиться (+" + pending + " душ)" : "Пока недоступно");
            rebirth.setEnabled(pending > 0);
        });

        JLabel talTitle = UI.label("Талисманы (за свободные души)", Font.BOLD, 15, UI.TEXT);
        talTitle.setBorder(BorderFactory.createEmptyBorder(6, 4, 2, 0));
        addRow(page, talTitle);

        for (Talisman t : game.talismans) {
            UI.Card card = new UI.Card(new BorderLayout(10, 0));
            JPanel info = column();
            JLabel name = UI.label(t.name, Font.BOLD, 14, UI.TEXT);
            info.add(name);
            info.add(UI.label(t.description, Font.PLAIN, 12, UI.MUTED));
            card.add(info, BorderLayout.CENTER);
            UI.Button buy = new UI.Button("");
            buy.setColor(UI.SOUL);
            buy.setPreferredSize(new Dimension(120, 40));
            card.add(buy, BorderLayout.EAST);
            buy.addActionListener(e -> {
                if (game.buyTalisman(t)) {
                    onPurchase.run();
                    refresh();
                }
            });
            updaters.add(() -> {
                UI.setText(name, t.name + "   ур. " + t.level + " / " + t.maxLevel);
                if (t.maxed()) {
                    UI.setText(buy, "Максимум");
                    buy.setEnabled(false);
                } else {
                    UI.setText(buy, "<html><center>Улучшить<br>" + t.cost() + " душ</center></html>");
                    buy.setEnabled(game.availableSouls() >= t.cost());
                }
            });
            addRow(page, card);
        }

        UI.Button reset = new UI.Button("Начать игру с нуля");
        reset.setColor(new Color(0xB0A0A8));
        reset.addActionListener(e -> {
            int answer = JOptionPane.showConfirmDialog(this,
                "Удалить весь прогресс, включая души, наряды и талисманы?", "Полный сброс",
                JOptionPane.OK_CANCEL_OPTION, JOptionPane.WARNING_MESSAGE);
            if (answer == JOptionPane.OK_OPTION) {
                game.resetAll();
                onPurchase.run();
                refresh();
            }
        });
        JPanel resetRow = new JPanel(new FlowLayout(FlowLayout.CENTER));
        resetRow.setOpaque(false);
        resetRow.add(reset);
        resetRow.setMaximumSize(new Dimension(Integer.MAX_VALUE, 50));
        addRow(page, resetRow);
        return page;
    }

    // ---------- Статистика ----------

    private JPanel buildStatsPage() {
        JPanel page = page();
        UI.Card card = new UI.Card(new GridLayout(0, 2, 8, 6));
        stat(card, "Золото", () -> Fmt.num(game.gold) + " рё");
        stat(card, "Заработано всего", () -> Fmt.num(game.totalGold) + " рё");
        stat(card, "Урон клика", () -> Fmt.num(game.clickDamage()));
        stat(card, "Урон отряда", () -> Fmt.num(game.dps()) + " / с");
        stat(card, "Шанс крита", () -> Math.round(game.critChance() * 100) + "% (×" + (int) Game.CRIT_MULT + ")");
        stat(card, "Время на босса", () -> Math.round(game.bossTime()) + " с");
        stat(card, "Текущий этап", () -> String.valueOf(game.stage));
        stat(card, "Открыто этапов", () -> String.valueOf(game.maxStage));
        stat(card, "Рекорд этапа", () -> String.valueOf(game.bestStage));
        stat(card, "Ударов", () -> Fmt.num(game.totalClicks));
        stat(card, "Ёкаев повержено", () -> Fmt.num(game.totalKills));
        stat(card, "Боссов повержено", () -> Fmt.num(game.bossesKilled));
        stat(card, "Героинь в отряде", () -> String.valueOf(game.heroines.stream().filter(h -> !h.clicker && h.level > 0).count()));
        stat(card, "Перерождений", () -> String.valueOf(game.rebirths));
        stat(card, "Души сакуры", () -> String.valueOf(game.souls));
        stat(card, "Время в игре", () -> Fmt.time(game.playTime));
        addRow(page, card);

        UI.Card help = new UI.Card(new BorderLayout());
        help.add(UI.label("<html><b>Управление</b><br>"
            + "Клик по полю боя — удар Сакуры<br>"
            + "1 – 4 — навыки<br>"
            + "← / → — предыдущий / следующий этап<br>"
            + "A — авто-продвижение вперёд<br>"
            + "Игра сохраняется сама, а героини сражаются, даже когда игра закрыта (до 8 часов).</html>",
            Font.PLAIN, 12, UI.TEXT), BorderLayout.CENTER);
        help.setPreferredSize(new Dimension(400, 140));
        addRow(page, help);
        return page;
    }

    private void stat(JPanel card, String label, Supplier<String> value) {
        card.add(UI.label(label, Font.PLAIN, 13, UI.MUTED));
        JLabel v = UI.label("", Font.BOLD, 13, UI.TEXT);
        v.setHorizontalAlignment(SwingConstants.RIGHT);
        card.add(v);
        updaters.add(() -> UI.setText(v, value.get()));
    }
}
