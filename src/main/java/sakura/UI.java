package sakura;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Insets;
import java.awt.LayoutManager;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.awt.geom.RoundRectangle2D;
import javax.swing.BorderFactory;
import javax.swing.JButton;
import javax.swing.JLabel;
import javax.swing.JPanel;

/** Цвета, шрифты и стилизованные компоненты. */
public final class UI {
    private UI() {}

    public static final Color BG = new Color(0xFFF0F5);
    public static final Color CARD = Color.WHITE;
    public static final Color CARD_BORDER = new Color(0xF3D3E0);
    public static final Color PRIMARY = new Color(0xE0598B);
    public static final Color PRIMARY_DARK = new Color(0xB83B6B);
    public static final Color TEXT = new Color(0x4A2C3A);
    public static final Color MUTED = new Color(0x9A7A88);
    public static final Color GOLD = new Color(0xF2B400);
    public static final Color DISABLED = new Color(0xE8D3DC);
    public static final Color GREEN = new Color(0x43A047);
    public static final Color SOUL = new Color(0x8E5CC4);

    public static final String FONT = "SansSerif";

    public static Font font(int style, int size) {
        return new Font(FONT, style, size);
    }

    public static JLabel label(String text, int style, int size, Color color) {
        JLabel l = new JLabel(text);
        l.setFont(font(style, size));
        l.setForeground(color);
        return l;
    }

    /** Меняет текст, только если он изменился (чтобы не дёргать перерисовку). */
    public static void setText(JLabel l, String text) {
        if (!text.equals(l.getText())) l.setText(text);
    }

    public static void setText(JButton b, String text) {
        if (!text.equals(b.getText())) b.setText(text);
    }

    /** Карточка с закруглёнными углами. */
    public static class Card extends JPanel {
        private Color fill = CARD;
        private Color border = CARD_BORDER;

        public Card(LayoutManager layout) {
            super(layout);
            setOpaque(false);
            setBorder(BorderFactory.createEmptyBorder(10, 12, 10, 12));
        }

        public void setColors(Color fill, Color border) {
            if (!fill.equals(this.fill) || !border.equals(this.border)) {
                this.fill = fill;
                this.border = border;
                repaint();
            }
        }

        @Override
        protected void paintComponent(Graphics g0) {
            Graphics2D g = (Graphics2D) g0.create();
            Art.smooth(g);
            RoundRectangle2D r = new RoundRectangle2D.Double(1, 1, getWidth() - 3, getHeight() - 3, 18, 18);
            g.setColor(new Color(0, 0, 0, 14));
            g.fill(new RoundRectangle2D.Double(1, 3, getWidth() - 3, getHeight() - 3, 18, 18));
            g.setColor(fill);
            g.fill(r);
            g.setColor(border);
            g.setStroke(new BasicStroke(1.5f));
            g.draw(r);
            g.dispose();
        }

        @Override
        public Dimension getMaximumSize() {
            return new Dimension(Integer.MAX_VALUE, getPreferredSize().height);
        }
    }

    /** Плоская розовая кнопка. */
    public static class Button extends JButton {
        private boolean hover;
        private Color color = PRIMARY;

        public Button(String text) {
            super(text);
            setFont(font(Font.BOLD, 12));
            setForeground(Color.WHITE);
            setFocusPainted(false);
            setContentAreaFilled(false);
            setBorderPainted(false);
            setOpaque(false);
            setMargin(new Insets(6, 12, 6, 12));
            setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
            addMouseListener(new MouseAdapter() {
                @Override
                public void mouseEntered(MouseEvent e) {
                    hover = true;
                    repaint();
                }

                @Override
                public void mouseExited(MouseEvent e) {
                    hover = false;
                    repaint();
                }
            });
        }

        public void setColor(Color c) {
            if (!c.equals(color)) {
                color = c;
                repaint();
            }
        }

        @Override
        public void setEnabled(boolean b) {
            if (b != isEnabled()) super.setEnabled(b);
        }

        @Override
        protected void paintComponent(Graphics g0) {
            Graphics2D g = (Graphics2D) g0.create();
            Art.smooth(g);
            Color c = !isEnabled() ? DISABLED : getModel().isPressed() ? Art.darker(color, 0.85) : hover ? Art.lighter(color, 0.12) : color;
            g.setColor(c);
            g.fill(new RoundRectangle2D.Double(0, 0, getWidth(), getHeight(), 14, 14));
            g.dispose();
            setForeground(isEnabled() ? Color.WHITE : MUTED);
            super.paintComponent(g0);
        }
    }
}
