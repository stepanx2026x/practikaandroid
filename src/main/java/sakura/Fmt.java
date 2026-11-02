package sakura;

/** Форматирование чисел и времени. */
public final class Fmt {
    private static final String[] SUFFIXES = {
        "", " тыс.", " млн", " млрд", " трлн", " квдрлн", " квнтлн", " скстлн", " сптлн", " октлн", " нонлн", " дцлн",
    };

    private Fmt() {}

    public static String num(double v) {
        if (Double.isNaN(v) || Double.isInfinite(v)) return "∞";
        if (v < 0) return "-" + num(-v);
        if (v < 1000) {
            if (v < 10 && v != Math.floor(v)) return String.format("%.1f", v);
            return String.format("%.0f", Math.floor(v));
        }
        int i = 0;
        double x = v;
        while (x >= 1000 && i < SUFFIXES.length - 1) {
            x /= 1000;
            i++;
        }
        if (x >= 1000) return String.format("%.2e", v);
        String s = String.format("%.2f", x);
        while (s.endsWith("0")) s = s.substring(0, s.length() - 1);
        if (s.endsWith(",") || s.endsWith(".")) s = s.substring(0, s.length() - 1);
        return s + SUFFIXES[i];
    }

    public static String time(double seconds) {
        long s = (long) seconds;
        long h = s / 3600, m = (s % 3600) / 60, sec = s % 60;
        if (h > 0) return h + " ч " + m + " мин";
        if (m > 0) return m + " мин " + sec + " с";
        return sec + " с";
    }
}
