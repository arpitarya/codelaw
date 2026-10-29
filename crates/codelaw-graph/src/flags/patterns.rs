//! Accessor patterns (SR-CONFIG `[flags] accessors`): a call with a `$NAME`
//! slot — `isFeatureOn($NAME)`, `flags.get($NAME, $$$)`,
//! `useFeature({ key: $NAME })`, `$_.variation($NAME, $$$)` — and the
//! presets, the same patterns shipped for six flag SDKs.

use super::extract::{Arg, Call};

/// One argument of a pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgPat {
    /// `$NAME`: the flag's name.
    Name,
    /// `$_`: any one argument.
    Any,
    /// `$$$`: any number of arguments, last.
    Rest,
    /// `{ key: $NAME }`: an object literal whose `key` is the name (JS).
    Key(String),
    /// `key=$NAME`: a keyword argument (Python).
    Keyword(String),
}

/// A parsed accessor pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub text: String,
    /// `$_.x(…)`: called on any receiver.
    pub any_receiver: bool,
    /// The callee's path; with `any_receiver`, its last segments.
    pub path: Vec<String>,
    pub args: Vec<ArgPat>,
    /// Applies to JS and TS files, to Python files.
    pub js: bool,
    pub py: bool,
}

fn ident(s: &str) -> bool {
    !s.is_empty() && !s.starts_with(|c: char| c.is_ascii_digit()) && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Split at top-level commas.
fn split_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in s.char_indices() {
        match c {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = s[start..].trim();
    if !last.is_empty() || !out.is_empty() {
        out.push(last);
    }
    out
}

impl Pattern {
    pub fn parse(text: &str) -> Result<Pattern, String> {
        let t = text.trim();
        let err = |why: &str| format!("accessor `{text}`: {why}");
        let open = t.find('(').ok_or_else(|| err("is not a call (`name($NAME)`)"))?;
        if !t.ends_with(')') {
            return Err(err("does not end with `)`"));
        }
        let callee = t[..open].trim();
        let mut path: Vec<String> = callee.split('.').map(|s| s.trim().to_owned()).collect();
        let any_receiver = path.first().is_some_and(|s| s == "$_");
        if any_receiver {
            path.remove(0);
        }
        if path.is_empty() || !path.iter().all(|s| ident(s) && s != "$_") {
            return Err(err("its callee is not a dotted name (`$_.` may start it)"));
        }
        let mut args = Vec::new();
        let mut names = 0;
        let parts = split_args(&t[open + 1..t.len() - 1]);
        for (i, a) in parts.iter().enumerate() {
            let p = match *a {
                "$NAME" => ArgPat::Name,
                "$_" => ArgPat::Any,
                "$$$" if i + 1 == parts.len() => ArgPat::Rest,
                "$$$" => return Err(err("`$$$` must be the last argument")),
                obj if obj.starts_with('{') && obj.ends_with('}') => {
                    let inner: Vec<&str> = split_args(&obj[1..obj.len() - 1]).into_iter().filter(|p| *p != "$$$").collect();
                    let [pair] = inner[..] else { return Err(err("an object argument holds one `key: $NAME`")) };
                    let (k, v) = pair.split_once(':').ok_or_else(|| err("an object argument holds one `key: $NAME`"))?;
                    let k = k.trim().trim_matches(['"', '\'']);
                    if v.trim() != "$NAME" || !ident(k) {
                        return Err(err("an object argument holds one `key: $NAME`"));
                    }
                    ArgPat::Key(k.to_owned())
                }
                kw if kw.contains('=') => {
                    let (k, v) = kw.split_once('=').expect("contains `=`");
                    if v.trim() != "$NAME" || !ident(k.trim()) {
                        return Err(err("a keyword argument is `key=$NAME`"));
                    }
                    ArgPat::Keyword(k.trim().to_owned())
                }
                other => return Err(err(&format!("`{other}` is not `$NAME`, `$_`, `$$$`, `{{ key: $NAME }}` or `key=$NAME`"))),
            };
            if matches!(p, ArgPat::Name | ArgPat::Key(_) | ArgPat::Keyword(_)) {
                names += 1;
            }
            args.push(p);
        }
        if names != 1 {
            return Err(err("needs exactly one `$NAME`"));
        }
        let js = !args.iter().any(|a| matches!(a, ArgPat::Keyword(_)));
        let py = !args.iter().any(|a| matches!(a, ArgPat::Key(_)));
        Ok(Pattern { text: t.to_owned(), any_receiver, path, args, js, py })
    }

    /// The call's argument in the `$NAME` slot, when the call matches.
    pub fn name_arg<'c>(&self, call: &'c Call) -> Option<&'c Arg> {
        let c = &call.callee;
        let callee_ok = if self.any_receiver { c.len() > self.path.len() && c[c.len() - self.path.len()..] == self.path[..] } else { *c == self.path };
        if !callee_ok {
            return None;
        }
        let positional: Vec<&Arg> = call.args.iter().filter(|a| !matches!(a, Arg::Keyword(..))).collect();
        let mut i = 0;
        let mut rest = false;
        let mut found = None;
        for p in &self.args {
            match p {
                ArgPat::Rest => {
                    rest = true;
                    break;
                }
                ArgPat::Any | ArgPat::Name | ArgPat::Key(_) => {
                    let a = *positional.get(i)?;
                    if matches!(a, Arg::Spread(_)) {
                        return None;
                    }
                    match p {
                        ArgPat::Name => found = Some(a),
                        ArgPat::Key(k) => {
                            let Arg::Object(keys, _) = a else { return None };
                            found = Some(keys.iter().find(|(key, _)| key == k).map(|(_, v)| v)?);
                        }
                        _ => {}
                    }
                    i += 1;
                }
                ArgPat::Keyword(k) => {
                    found = Some(call.args.iter().find_map(|a| match a {
                        Arg::Keyword(n, v, _) if n == k => Some(&**v),
                        _ => None,
                    })?);
                }
            }
        }
        if !rest && i != positional.len() {
            return None;
        }
        found
    }
}

/// The presets' names, in the order `[flags] presets` lists them.
pub const PRESETS: &[&str] = &["launchdarkly", "unleash", "split", "openfeature", "growthbook", "flagsmith"];

/// A preset's patterns: (JS and TS, Python). Each is the SDK's documented
/// read of one flag by its key; a read that returns every flag at once
/// (`useFlags()`, `allFlags()`) names none and is not a pattern.
pub fn preset(name: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    Some(match name {
        // LaunchDarkly: server and client SDKs' `variation` family.
        "launchdarkly" => (
            &[
                "$_.variation($NAME, $$$)",
                "$_.variationDetail($NAME, $$$)",
                "$_.boolVariation($NAME, $$$)",
                "$_.boolVariationDetail($NAME, $$$)",
                "$_.stringVariation($NAME, $$$)",
                "$_.numberVariation($NAME, $$$)",
                "$_.jsonVariation($NAME, $$$)",
            ],
            &["$_.variation($NAME, $$$)", "$_.variation_detail($NAME, $$$)"],
        ),
        // Unleash: `isEnabled` / `getVariant`, and the React proxy's hooks.
        "unleash" => (
            &["$_.isEnabled($NAME, $$$)", "$_.getVariant($NAME, $$$)", "useFlag($NAME)", "useVariant($NAME)"],
            &["$_.is_enabled($NAME, $$$)", "$_.get_variant($NAME, $$$)"],
        ),
        // Split: the server SDK takes the key first, the browser SDK not.
        "split" => (
            &["$_.getTreatment($NAME)", "$_.getTreatment($_, $NAME, $$$)", "$_.getTreatmentWithConfig($NAME)", "$_.getTreatmentWithConfig($_, $NAME, $$$)"],
            &["$_.get_treatment($_, $NAME, $$$)", "$_.get_treatment_with_config($_, $NAME, $$$)"],
        ),
        // OpenFeature: the typed evaluation methods and the React hooks.
        "openfeature" => (
            &[
                "$_.getBooleanValue($NAME, $$$)",
                "$_.getStringValue($NAME, $$$)",
                "$_.getNumberValue($NAME, $$$)",
                "$_.getObjectValue($NAME, $$$)",
                "$_.getBooleanDetails($NAME, $$$)",
                "$_.getStringDetails($NAME, $$$)",
                "$_.getNumberDetails($NAME, $$$)",
                "$_.getObjectDetails($NAME, $$$)",
                "useBooleanFlagValue($NAME, $$$)",
                "useStringFlagValue($NAME, $$$)",
                "useNumberFlagValue($NAME, $$$)",
                "useObjectFlagValue($NAME, $$$)",
                "useFlag($NAME, $$$)",
            ],
            &[
                "$_.get_boolean_value($NAME, $$$)",
                "$_.get_string_value($NAME, $$$)",
                "$_.get_integer_value($NAME, $$$)",
                "$_.get_float_value($NAME, $$$)",
                "$_.get_object_value($NAME, $$$)",
                "$_.get_boolean_details($NAME, $$$)",
                "$_.get_string_details($NAME, $$$)",
                "$_.get_integer_details($NAME, $$$)",
                "$_.get_float_details($NAME, $$$)",
                "$_.get_object_details($NAME, $$$)",
            ],
        ),
        // GrowthBook: `isOn`, `getFeatureValue`, and the React hooks.
        "growthbook" => (
            &["$_.isOn($NAME)", "$_.isOff($NAME)", "$_.getFeatureValue($NAME, $$$)", "$_.evalFeature($NAME)", "useFeatureIsOn($NAME)", "useFeatureValue($NAME, $$$)", "useFeature($NAME)"],
            &["$_.is_on($NAME)", "$_.is_off($NAME)", "$_.get_feature_value($NAME, $$$)", "$_.eval_feature($NAME)"],
        ),
        // Flagsmith: the JS client and the Python `Flags` object.
        "flagsmith" => (
            &["$_.hasFeature($NAME)", "$_.getValue($NAME, $$$)"],
            &["$_.is_feature_enabled($NAME)", "$_.get_feature_value($NAME)"],
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_parses_and_fits_its_language() {
        for p in PRESETS {
            let (js, py) = preset(p).unwrap();
            for t in js.iter() {
                assert!(Pattern::parse(t).unwrap().js, "{t}");
            }
            for t in py.iter() {
                assert!(Pattern::parse(t).unwrap().py, "{t}");
            }
        }
        assert!(preset("nope").is_none());
    }

    #[test]
    fn patterns_parse_and_refuse() {
        let p = Pattern::parse("useFeature({ key: $NAME })").unwrap();
        assert_eq!(p.args, vec![ArgPat::Key("key".into())]);
        assert!(p.js && !p.py);
        let p = Pattern::parse("flags.get(key=$NAME)").unwrap();
        assert!(p.py && !p.js);
        assert!(Pattern::parse("flags.get($_)").unwrap_err().contains("exactly one"));
        assert!(Pattern::parse("flags.get($$$, $NAME)").unwrap_err().contains("last"));
        assert!(Pattern::parse("flags").unwrap_err().contains("not a call"));
    }
}
