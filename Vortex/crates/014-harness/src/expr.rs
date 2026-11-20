//! The grader's own arithmetic evaluator (shunting-yard), deliberately
//! separate from the calculator tool it grades. Used to decide whether two
//! spellings of an expression ("12*7", "12 * 7", "12 x 7") mean the same
//! thing.

pub fn eval(s: &str) -> Option<f64> {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum T {
        N(f64),
        Op(char),
        Neg,
        L,
        R,
    }
    let mut toks = Vec::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_ascii_digit() || c == '.' {
            let st = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.' || (cs[i] == ',' && i + 3 < cs.len() + 1 && cs.get(i + 1..i + 4).is_some_and(|d| d.iter().all(char::is_ascii_digit)))) {
                i += 1;
            }
            let t: String = cs[st..i].iter().filter(|c| **c != ',').collect();
            toks.push(T::N(t.parse().ok()?));
            continue;
        }
        let prev_value = matches!(toks.last(), Some(T::N(_)) | Some(T::R));
        match c {
            ' ' => {}
            '+' | '*' | '/' | '^' | '%' => toks.push(T::Op(c)),
            'x' | 'X' | '×' => toks.push(T::Op('*')),
            '÷' => toks.push(T::Op('/')),
            '-' if prev_value => toks.push(T::Op('-')),
            '-' => toks.push(T::Neg),
            '(' => toks.push(T::L),
            ')' => toks.push(T::R),
            _ => return None,
        }
        i += 1;
    }
    let prec = |t: T| match t {
        T::Op('+') | T::Op('-') => 1,
        T::Op('*') | T::Op('/') | T::Op('%') => 2,
        T::Neg => 3,
        T::Op('^') => 4,
        _ => 0,
    };
    let right = |t: T| matches!(t, T::Op('^') | T::Neg);
    let mut out: Vec<T> = Vec::new();
    let mut ops: Vec<T> = Vec::new();
    for t in toks {
        match t {
            T::N(_) => out.push(t),
            T::Op(_) | T::Neg => {
                while let Some(&top) = ops.last() {
                    if top != T::L && (prec(top) > prec(t) || (prec(top) == prec(t) && !right(t))) {
                        out.push(ops.pop()?);
                    } else {
                        break;
                    }
                }
                ops.push(t);
            }
            T::L => ops.push(t),
            T::R => {
                loop {
                    match ops.pop()? {
                        T::L => break,
                        x => out.push(x),
                    }
                }
            }
        }
    }
    while let Some(t) = ops.pop() {
        if t == T::L {
            return None;
        }
        out.push(t);
    }
    let mut st: Vec<f64> = Vec::new();
    for t in out {
        match t {
            T::N(v) => st.push(v),
            T::Neg => {
                let a = st.pop()?;
                st.push(-a);
            }
            T::Op(o) => {
                let b = st.pop()?;
                let a = st.pop()?;
                st.push(match o {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    '%' => a % b,
                    '^' => a.powf(b),
                    _ => return None,
                });
            }
            _ => return None,
        }
    }
    (st.len() == 1 && st[0].is_finite()).then(|| st[0])
}

#[cfg(test)]
mod tests {
    use super::eval;

    #[test]
    fn agrees_on_spellings() {
        assert_eq!(eval("12*7"), Some(84.0));
        assert_eq!(eval("12 x 7"), Some(84.0));
        assert_eq!(eval("2^3^2"), Some(512.0));
        assert_eq!(eval("-(3+4)*2"), Some(-14.0));
        assert_eq!(eval("1,250/5"), Some(250.0));
        assert_eq!(eval("2+"), None);
        assert_eq!(eval("hello"), None);
    }
}
