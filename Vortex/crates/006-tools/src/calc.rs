//! Arithmetic evaluator: `+ - * / ^ %`, parentheses, unary minus, decimals,
//! and the common typed operators `x`, `×`, `÷`. Recursive descent over
//! `f64`; `%` is modulo. Used by the `calc` tool and by graders that compare
//! expressions by value rather than spelling.

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    Syntax(String),
    DivideByZero,
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::Syntax(s) => write!(f, "syntax error: {s}"),
            CalcError::DivideByZero => write!(f, "division by zero"),
        }
    }
}

pub fn eval(expr: &str) -> Result<f64, CalcError> {
    let tokens = lex(expr)?;
    let mut p = Parser { t: &tokens, i: 0 };
    let v = p.expr()?;
    if p.i != tokens.len() {
        return Err(CalcError::Syntax(format!("unexpected token at {}", p.i)));
    }
    if !v.is_finite() {
        return Err(CalcError::Syntax("non-finite result".into()));
    }
    Ok(v)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),
    LParen,
    RParen,
}

fn lex(s: &str) -> Result<Vec<Tok>, CalcError> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | ',' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                // A comma followed by exactly three digits is a thousands
                // separator ("1,000"); any other comma separates.
                let thousands = |j: usize| {
                    chars[j] == ','
                        && j + 3 < chars.len() + 1
                        && chars.get(j + 1..j + 4).is_some_and(|d| d.iter().all(char::is_ascii_digit))
                        && !chars.get(j + 4).is_some_and(char::is_ascii_digit)
                };
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || thousands(i)) {
                    i += 1;
                }
                let text: String = chars[start..i].iter().filter(|c| **c != ',').collect();
                let n = text.parse::<f64>().map_err(|_| CalcError::Syntax(format!("bad number {text}")))?;
                out.push(Tok::Num(n));
            }
            '+' | '-' | '*' | '/' | '^' | '%' => {
                out.push(Tok::Op(c));
                i += 1;
            }
            'x' | 'X' | '×' | '·' => {
                out.push(Tok::Op('*'));
                i += 1;
            }
            '÷' => {
                out.push(Tok::Op('/'));
                i += 1;
            }
            '(' | '[' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' | ']' => {
                out.push(Tok::RParen);
                i += 1;
            }
            _ => return Err(CalcError::Syntax(format!("unexpected character {c:?}"))),
        }
    }
    if out.is_empty() {
        return Err(CalcError::Syntax("empty expression".into()));
    }
    Ok(out)
}

struct Parser<'a> {
    t: &'a [Tok],
    i: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }

    fn expr(&mut self) -> Result<f64, CalcError> {
        let mut v = self.term()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.i += 1;
            let r = self.term()?;
            v = if op == '+' { v + r } else { v - r };
        }
        Ok(v)
    }

    fn term(&mut self) -> Result<f64, CalcError> {
        let mut v = self.power()?;
        while let Some(Tok::Op(op @ ('*' | '/' | '%'))) = self.peek().cloned() {
            self.i += 1;
            let r = self.power()?;
            v = match op {
                '*' => v * r,
                '/' if r == 0.0 => return Err(CalcError::DivideByZero),
                '/' => v / r,
                _ if r == 0.0 => return Err(CalcError::DivideByZero),
                _ => v % r,
            };
        }
        Ok(v)
    }

    fn power(&mut self) -> Result<f64, CalcError> {
        let base = self.unary()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.i += 1;
            let exp = self.power()?; // right-associative
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn unary(&mut self) -> Result<f64, CalcError> {
        match self.peek().cloned() {
            Some(Tok::Op('-')) => {
                self.i += 1;
                Ok(-self.unary()?)
            }
            Some(Tok::Op('+')) => {
                self.i += 1;
                self.unary()
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Result<f64, CalcError> {
        match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.i += 1;
                Ok(n)
            }
            Some(Tok::LParen) => {
                self.i += 1;
                let v = self.expr()?;
                match self.peek() {
                    Some(Tok::RParen) => {
                        self.i += 1;
                        Ok(v)
                    }
                    _ => Err(CalcError::Syntax("missing )".into())),
                }
            }
            other => Err(CalcError::Syntax(format!("expected a number, found {other:?}"))),
        }
    }
}

/// Format a result without float noise: integers print as integers.
pub fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 && v.abs() < 1e15 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.10}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_and_forms() {
        assert_eq!(eval("2+3*4").unwrap(), 14.0);
        assert_eq!(eval("(2+3)*4").unwrap(), 20.0);
        assert_eq!(eval("2^3^2").unwrap(), 512.0);
        assert_eq!(eval("-3 + 5").unwrap(), 2.0);
        assert_eq!(eval("12 x 7").unwrap(), 84.0);
        assert_eq!(eval("100 ÷ 8").unwrap(), 12.5);
        assert_eq!(eval("1,000 * 3").unwrap(), 3000.0);
        assert_eq!(eval("17 % 5").unwrap(), 2.0);
        assert_eq!(eval("1/0"), Err(CalcError::DivideByZero));
        assert!(eval("2 +").is_err());
        assert_eq!(format_number(84.0), "84");
        assert_eq!(format_number(12.5), "12.5");
    }
}
