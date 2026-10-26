//! String similarity for "did you mean ...?" suggestions (L11: no crate).
//!
//! Both functions work on `char`s, so they agree with the `strsim` functions
//! they replaced on every input the suggestions use.

/// The edit distance between two strings: the fewest single-character
/// inserts, deletes and substitutions that turn `a` into `b`.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diag = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diag + usize::from(ca != cb)).min(above + 1).min(row[j] + 1);
            diag = above;
        }
    }
    row[b.len()]
}

/// The Jaro similarity, 0.0 (nothing alike) to 1.0 (equal).
fn jaro(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let window = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut a_hit = vec![false; a.len()];
    let mut b_hit = vec![false; b.len()];
    let mut matches = 0usize;
    for (i, ca) in a.iter().enumerate() {
        let lo = i.saturating_sub(window);
        let hi = (i + window + 1).min(b.len());
        for j in lo..hi {
            if !b_hit[j] && b[j] == *ca {
                a_hit[i] = true;
                b_hit[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }
    let mut transposed = 0usize;
    let mut j = 0;
    for (i, ca) in a.iter().enumerate() {
        if a_hit[i] {
            while !b_hit[j] {
                j += 1;
            }
            if *ca != b[j] {
                transposed += 1;
            }
            j += 1;
        }
    }
    let m = matches as f64;
    (m / a.len() as f64 + m / b.len() as f64 + (m - (transposed / 2) as f64) / m) / 3.0
}

/// The Jaro-Winkler similarity, 0.0 to 1.0: Jaro, raised for a shared prefix
/// of up to four characters when the strings are already close.
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let j = jaro(a, b);
    if j <= 0.7 {
        return j;
    }
    let prefix = a.chars().zip(b.chars()).take(4).take_while(|(x, y)| x == y).count();
    j + 0.1 * prefix as f64 * (1.0 - j)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_known_pairs() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("same", "same"), 0);
        assert_eq!(levenshtein("héllo", "hello"), 1);
    }

    #[test]
    fn jaro_winkler_known_pairs() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-4;
        assert_eq!(jaro_winkler("", ""), 1.0);
        assert_eq!(jaro_winkler("abc", ""), 0.0);
        assert_eq!(jaro_winkler("same", "same"), 1.0);
        assert!(close(jaro_winkler("martha", "marhta"), 0.9611));
        assert!(close(jaro_winkler("dixon", "dicksonx"), 0.8133));
        assert!(close(jaro_winkler("dwayne", "duane"), 0.84));
        assert_eq!(jaro_winkler("abc", "xyz"), 0.0);
    }
}
