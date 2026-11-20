//! Word-level tokenisation: lowercase, Unicode-alphanumeric runs (with inner
//! apostrophes and hyphens), numbers kept as tokens, punctuation dropped.
//! Simple and deterministic; the n-gram model's job is statistics, not
//! segmentation.

pub fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let inner_join = (c == '\'' || c == '\u{2019}' || c == '-')
            && !cur.is_empty()
            && chars.get(i + 1).is_some_and(|n| n.is_alphanumeric());
        if c.is_alphanumeric() {
            cur.extend(c.to_lowercase());
        } else if inner_join {
            cur.push(if c == '-' { '-' } else { '\'' });
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_and_lowercases() {
        assert_eq!(words("The cat's well-known 3.5 hats!"), vec!["the", "cat's", "well-known", "3", "5", "hats"]);
        assert!(words("  ...  ").is_empty());
    }
}
