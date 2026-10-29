use super::*;
use crate::{GraphConfig, GraphInput, Manifest, extract, is_manifest};
use codelaw_core::{Language, Severity};
use std::path::Path;

fn project(files: &[(&str, &str)]) -> (Graph, BTreeMap<String, FlagFacts>) {
    let mut inputs = Vec::new();
    let mut manifests = Vec::new();
    let mut flag_facts = BTreeMap::new();
    for (p, t) in files {
        let name = p.rsplit('/').next().unwrap();
        if is_manifest(name) {
            manifests.push(Manifest { path: p.to_string(), text: t.to_string() });
        }
        let lang = Language::from_path(Path::new(p));
        inputs.push(GraphInput { path: p.to_string(), facts: lang.and_then(|l| extract(l, t)) });
        if let Some(f) = lang.and_then(|l| extract_flags(l, t)) {
            flag_facts.insert(p.to_string(), f);
        }
    }
    (Graph::build(inputs, &manifests, GraphConfig::default()), flag_facts)
}

fn options(json: &str) -> FlagOptions {
    FlagOptions::from_options(&serde_json::from_str(json).unwrap()).unwrap()
}

fn report(files: &[(&str, &str)], json: &str, as_of: Option<&str>) -> FlagReport {
    let (g, f) = project(files);
    let registry = files.iter().find(|(p, _)| *p == REGISTRY_REL).map(|(_, t)| *t);
    check(&g, &f, registry, &options(json), as_of).unwrap()
}

fn lines(r: &FlagReport) -> Vec<String> {
    r.findings.iter().map(|f| format!("{:?} {} {:?} {}", f.check, f.file, f.severity, f.message)).collect()
}

#[test]
fn a_two_level_wrapper_across_files_is_an_accessor() {
    let files = [
        ("src/flags.ts", "import { client } from \"./ld\";\nexport function isOn(name: string) { return client.variation(name, false); }\n"),
        ("src/feature.ts", "import { isOn } from \"./flags\";\nexport const enabled = (flag: string) => isOn(flag);\n"),
        ("src/app.ts", "import { enabled as on } from \"./feature\";\nif (on(\"new-checkout\")) { go(); }\n"),
        ("src/ld.ts", "export const client = init();\n"),
    ];
    let r = report(&files, r#"{"presets":["launchdarkly"]}"#, None);
    let wrappers: Vec<&str> = r.accessors.iter().filter(|a| a.at.is_some()).map(|a| a.pattern.as_str()).collect();
    assert_eq!(wrappers, vec!["enabled($NAME)", "isOn($NAME)"]);
    assert_eq!(lines(&r), vec!["Reads src/app.ts Error Flag `new-checkout` is read here but is not registered: add it to `.codelaw/flags.toml` with an owner and an expiry date, or mark it `codelaw-flag new-checkout expires YYYY-MM-DD owner …`"]);
}

#[test]
fn python_wrappers_methods_and_constants() {
    let files = [
        ("app/flags.py", "from ldclient import get\nNEW = \"new-checkout\"\nclass Flags:\n    def on(self, key, default=False):\n        return get().variation(key, None, default)\n"),
        ("app/views.py", "from app.flags import Flags, NEW\nflags = Flags()\nif flags.on(NEW):\n    go()\nflags.on(compute())\n"),
        (REGISTRY_REL, "[[flag]]\nname = \"new-checkout\"\nowner = \"@pay\"\nexpires = \"2026-11-05\"\n"),
    ];
    let r = report(&files, r#"{"presets":["launchdarkly"]}"#, Some("2026-10-30"));
    assert_eq!(r.accessors.iter().filter(|a| a.at.is_some()).map(|a| a.pattern.clone()).collect::<Vec<_>>(), vec!["$_.on($NAME, $$$)"]);
    assert_eq!(
        lines(&r),
        vec![
            "Reads app/views.py Warn Flag `new-checkout` expires on 2026-11-05, in 6 days (owner @pay)",
            "Reads app/views.py Info A flag is read here, but its name is not known, so its expiry cannot be checked",
        ]
    );
    assert!(r.findings[1].uncertain.is_some());
}

#[test]
fn expired_with_keep_collapses_and_unused_entries_go() {
    let files = [
        ("src/a.ts", "if (unleash.isEnabled(\"old\")) {\n  a();\n} else {\n  b();\n}\n"),
        (REGISTRY_REL, "[[flag]]\nname = \"old\"\nexpires = \"2026-01-01\"\nkeep = \"off\"\n\n[[flag]]\nname = \"gone\"\nexpires = \"2027-01-01\"\n"),
    ];
    let r = report(&files, r#"{"presets":["unleash"]}"#, Some("2026-10-30"));
    let l = lines(&r);
    assert_eq!(l[0], "Reads src/a.ts Error Flag `old` expired on 2026-01-01 (302 days before the as-of date): remove it; it is decided (`keep = \"off\"`): keep that branch");
    assert_eq!(r.findings[0].edits[0].text, "b();");
    assert_eq!(l[1], "Registry .codelaw/flags.toml Info Flag `gone` has no read left in the project: remove its entry");
    assert_eq!(r.findings[1].edits[0].text, "");
}

#[test]
fn markers_count_and_disagree() {
    let files = [
        ("src/a.ts", "// codelaw-flag beta expires 2027-02-28 owner @team\nif (isFeatureOn(\"beta\")) go();\n// codelaw-flag reg expires 2030-01-01\nisFeatureOn(\"reg\");\n"),
        ("src/b.ts", "// codelaw-flag beta expires 2027-03-01 owner @team\n"),
        (REGISTRY_REL, "[[flag]]\nname = \"reg\"\nexpires = \"2027-01-01\"\n"),
    ];
    let r = report(&files, r#"{"accessors":["isFeatureOn($NAME)"]}"#, Some("2026-10-30"));
    assert_eq!(
        lines(&r),
        vec![
            "Markers src/a.ts Error Flag `beta` is marked in 2 places with different values; register it once in `.codelaw/flags.toml`",
            "Markers src/a.ts Warn This marker of `reg` disagrees with `.codelaw/flags.toml`, which wins",
            "Markers src/b.ts Error Flag `beta` is marked in 2 places with different values; register it once in `.codelaw/flags.toml`",
        ]
    );
    assert_eq!(r.findings[1].edits[0].text, "codelaw-flag reg expires 2027-01-01");
    // Registry-only: a marker is an error, removed when the registry has it.
    let r = report(&files, r#"{"accessors":["isFeatureOn($NAME)"],"inline":"registry-only"}"#, None);
    assert_eq!(r.findings.iter().filter(|f| f.check == FlagCheck::Markers).count(), 3);
    assert!(r.findings.iter().any(|f| f.check == FlagCheck::Reads && f.message.starts_with("Flag `beta` is read here but is not registered")));
    assert_eq!(r.findings.iter().filter(|f| f.check == FlagCheck::Markers && !f.edits.is_empty()).count(), 1);
}

#[test]
fn object_keys_namespace_imports_and_enums() {
    let files = [
        ("src/names.ts", "export enum Flag { Beta = \"beta\" }\n"),
        ("src/use.tsx", "import * as names from \"./names\";\nconst a = useFeature({ key: names.Flag.Beta });\n"),
    ];
    let r = report(&files, r#"{"accessors":["useFeature({ key: $NAME })"]}"#, None);
    assert_eq!(lines(&r), vec!["Reads src/use.tsx Error Flag `beta` is read here but is not registered: add it to `.codelaw/flags.toml` with an owner and an expiry date, or mark it `codelaw-flag beta expires YYYY-MM-DD owner …`"]);
}

#[test]
fn never_needs_a_reason_and_is_never_due() {
    let files = [("src/a.js", "growthbook.isOn(\"kill\");\n"), (REGISTRY_REL, "[[flag]]\nname = \"kill\"\nexpires = \"never\"\n")];
    let r = report(&files, r#"{"presets":["growthbook"]}"#, Some("2030-01-01"));
    assert_eq!(lines(&r), vec!["Registry .codelaw/flags.toml Error Flag `kill` expires `never` without a `reason`"]);
    let files = [("src/a.js", "growthbook.isOn(\"kill\");\n"), (REGISTRY_REL, "[[flag]]\nname = \"kill\"\nexpires = \"never\"\nreason = \"kill switch\"\n")];
    assert!(report(&files, r#"{"presets":["growthbook"]}"#, Some("2030-01-01")).findings.is_empty());
    assert_eq!(Severity::Error, Severity::Error);
}
