package sakura;

import java.awt.BorderLayout;
import java.awt.Dimension;
import java.awt.Graphics2D;
import java.awt.event.ActionEvent;
import java.awt.event.KeyEvent;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;
import java.awt.image.BufferedImage;
import java.util.ArrayList;
import java.util.List;
import javax.swing.AbstractAction;
import javax.swing.JComponent;
import javax.swing.JFrame;
import javax.swing.JOptionPane;
import javax.swing.JRootPane;
import javax.swing.KeyStroke;
import javax.swing.SwingUtilities;
import javax.swing.Timer;
import javax.swing.UIManager;
import javax.swing.WindowConstants;

/** Точка входа: окно игры «Сакура и ёкаи». */
public final class Main {

    private Main() {}

    public static void main(String[] args) {
        System.setProperty("awt.useSystemAAFontSettings", "on");
        System.setProperty("sun.java2d.opengl", "false");
        SwingUtilities.invokeLater(Main::start);
    }

    private static void start() {
        try {
            UIManager.setLookAndFeel(UIManager.getCrossPlatformLookAndFeelClassName());
            UIManager.put("Button.disabledText", UI.MUTED);
        } catch (Exception ignored) {
            // стандартная тема подойдёт
        }

        Game game = new Game();
        long away = Save.load(game);

        JFrame frame = new JFrame("Сакура и ёкаи");
        frame.setDefaultCloseOperation(WindowConstants.DO_NOTHING_ON_CLOSE);
        frame.setIconImages(icons());

        BattlePanel battle = new BattlePanel(game);
        ShopPanel shop = new ShopPanel(game);
        shop.onPurchase = () -> Save.save(game);
        battle.onChange = shop::refresh;

        frame.getContentPane().setLayout(new BorderLayout());
        frame.getContentPane().add(battle, BorderLayout.CENTER);
        frame.getContentPane().add(shop, BorderLayout.EAST);
        frame.setMinimumSize(new Dimension(1050, 640));
        frame.pack();
        frame.setLocationRelativeTo(null);

        bindKeys(frame.getRootPane(), game, battle, shop);

        // Сохранение и при аварийном завершении (например, Ctrl+C в терминале)
        Runtime.getRuntime().addShutdownHook(new Thread(() -> Save.save(game)));

        frame.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosing(WindowEvent e) {
                frame.dispose();
                System.exit(0); // сохранение выполнит shutdown hook
            }
        });

        // Игровой цикл ~60 кадров в секунду
        final long[] last = {System.nanoTime()};
        final double[] sinceSave = {0};
        final int[] frameNo = {0};
        Timer loop = new Timer(16, e -> {
            long now = System.nanoTime();
            double dt = Math.min(0.25, (now - last[0]) / 1e9);
            last[0] = now;
            game.update(dt);
            battle.tick(dt);
            battle.repaint();
            if (++frameNo[0] % 6 == 0) shop.refresh();
            sinceSave[0] += dt;
            if (sinceSave[0] > 20) {
                sinceSave[0] = 0;
                Save.save(game);
            }
        });

        shop.refresh();
        frame.setVisible(true);
        loop.start();

        if (away > 60_000) {
            double gold = game.offlineGold(away / 1000.0);
            if (gold >= 1) {
                game.earn(gold);
                JOptionPane.showMessageDialog(frame,
                    "С возвращением!\nПока вас не было (" + Fmt.time(Math.min(away / 1000.0, Game.OFFLINE_LIMIT_SEC))
                        + "), отряд добыл " + Fmt.num(gold) + " рё.",
                    "Сакура и ёкаи", JOptionPane.INFORMATION_MESSAGE);
            }
        } else if (away < 0) {
            battle.onMessage("Кликайте по ёкаю, чтобы Сакура нанесла удар!");
            battle.onMessage("Нанимайте героинь справа — они сражаются сами");
        }
    }

    private static void bindKeys(JRootPane root, Game game, BattlePanel battle, ShopPanel shop) {
        for (int i = 0; i < 4; i++) {
            final int idx = i;
            bind(root, KeyStroke.getKeyStroke(KeyEvent.VK_1 + i, 0), "skill" + i, () -> battle.useSkill(idx));
            bind(root, KeyStroke.getKeyStroke(KeyEvent.VK_NUMPAD1 + i, 0), "skillN" + i, () -> battle.useSkill(idx));
        }
        bind(root, KeyStroke.getKeyStroke(KeyEvent.VK_LEFT, 0), "prev", () -> {
            game.setStage(game.stage - 1);
            game.autoAdvance = false;
            shop.refresh();
        });
        bind(root, KeyStroke.getKeyStroke(KeyEvent.VK_RIGHT, 0), "next", () -> {
            game.setStage(game.stage + 1);
            shop.refresh();
        });
        bind(root, KeyStroke.getKeyStroke(KeyEvent.VK_A, 0), "auto", () -> {
            game.autoAdvance = !game.autoAdvance;
            battle.onMessage(game.autoAdvance ? "Авто-продвижение включено" : "Авто-продвижение выключено");
        });
    }

    private static void bind(JRootPane root, KeyStroke key, String name, Runnable action) {
        root.getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(key, name);
        root.getActionMap().put(name, new AbstractAction() {
            @Override
            public void actionPerformed(ActionEvent e) {
                action.run();
            }
        });
    }

    /** Иконка окна — цветок сакуры. */
    private static List<BufferedImage> icons() {
        List<BufferedImage> list = new ArrayList<>();
        for (int size : new int[] {16, 32, 64, 128}) {
            BufferedImage img = new BufferedImage(size, size, BufferedImage.TYPE_INT_ARGB);
            Graphics2D g = img.createGraphics();
            Art.smooth(g);
            Art.drawBlossom(g, size / 2.0, size / 2.0 + size * 0.05, size * 0.45, new java.awt.Color(0xF8BBD0), new java.awt.Color(0xE0598B));
            g.dispose();
            list.add(img);
        }
        return list;
    }
}
