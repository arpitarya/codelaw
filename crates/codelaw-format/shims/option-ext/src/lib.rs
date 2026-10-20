//! `Option::contains` as an extension trait: the one item `dirs-sys` uses
//! from `option-ext`. Written for codelaw under MIT (SR-FORMATTER
//! decision 2, L2).

/// Extension methods for `Option`.
pub trait OptionExt<T> {
    /// Whether the option is `Some` and holds a value equal to `x`.
    #[must_use]
    fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn contains<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            Some(v) => x == v,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OptionExt;

    #[test]
    fn contains() {
        assert!(Some(2).contains(&2));
        assert!(!Some(3).contains(&2));
        assert!(!None::<u32>.contains(&2));
    }
}
