//! Форматирование чисел и времени по-русски.

const SUFFIXES: [&str; 12] = [
    "",
    " тыс.",
    " млн",
    " млрд",
    " трлн",
    " квдрлн",
    " квнтлн",
    " скстлн",
    " сптлн",
    " октлн",
    " нонлн",
    " дцлн",
];

/// 1234567 → «1,23 млн».
pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "∞".into();
    }
    if v < 0.0 {
        return format!("-{}", num(-v));
    }
    if v < 1000.0 {
        if v < 10.0 && v.fract() != 0.0 {
            return format!("{:.1}", v).replace('.', ",");
        }
        return format!("{}", v.floor() as u64);
    }
    let mut x = v;
    let mut i = 0;
    while x >= 1000.0 && i < SUFFIXES.len() - 1 {
        x /= 1000.0;
        i += 1;
    }
    if x >= 1000.0 {
        return format!("{:.2e}", v);
    }
    let mut s = if x >= 100.0 {
        format!("{:.1}", x)
    } else {
        format!("{:.2}", x)
    };
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s.replace('.', ",") + SUFFIXES[i]
}

/// Целое число с разделителями разрядов: 12 345.
pub fn int(v: u64) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{a0}');
        }
        out.push(ch);
    }
    out
}

/// 3725 → «1 ч 2 мин».
pub fn time(seconds: f64) -> String {
    let s = seconds.max(0.0) as u64;
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{} ч {} мин", h, m)
    } else if m > 0 {
        format!("{} мин {} с", m, sec)
    } else {
        format!("{} с", sec)
    }
}

/// Множитель: 1.5 → «×1,5».
pub fn mult(m: f64) -> String {
    format!("×{}", num(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(num(0.0), "0");
        assert_eq!(num(5.5), "5,5");
        assert_eq!(num(999.0), "999");
        assert_eq!(num(1000.0), "1 тыс.");
        assert_eq!(num(1500.0), "1,5 тыс.");
        assert_eq!(num(123_456.0), "123,5 тыс.");
        assert_eq!(num(2_500_000.0), "2,5 млн");
        assert_eq!(int(1234567), "1\u{a0}234\u{a0}567");
        assert_eq!(time(3725.0), "1 ч 2 мин");
        assert_eq!(time(65.0), "1 мин 5 с");
    }
}
