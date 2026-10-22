//! `codelaw bench` and L8's gate (SR-LAW-8 decisions 3, 6 and 7;
//! SR-CLI-MCP decision 10), end to end on a small workload.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-bench-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn packs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs")
}

fn bench(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .arg("bench")
        .args(["--files", "120", "--runs", "2", "--packs"])
        .arg(packs())
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

fn budgets(dir: &Path, startup: &str) -> PathBuf {
    let mut text = String::from("[reference]\nmachine = \"test\"\n");
    for (op, unit) in [
        ("startup", "ms"),
        ("check_warm_10k", "ms"),
        ("check_cold_10k", "ms"),
        ("format_warm_10k", "ms"),
        ("fix_one_file", "ms"),
        ("lsp_after_edit", "ms"),
        ("mcp_rules_for_path", "ms"),
        ("peak_memory_10k", "MiB"),
    ] {
        let budget = if op == "startup" { startup } else { "budget = \"unset\"\nproposed = 1" };
        text.push_str(&format!("\n[operation.{op}]\nunit = \"{unit}\"\n{budget}\n"));
    }
    let p = dir.join("budgets.toml");
    fs::write(&p, text).unwrap();
    p
}

/// Every operation runs and reports; with only proposals the gate fails
/// nothing, however far over a proposal the p95 is (L8 decision 6).
#[test]
fn every_operation_is_measured_and_a_proposal_never_fails() {
    let d = temp("all");
    let b = budgets(&d, "budget = \"unset\"\nproposed = 0.001");
    let out = d.join("report.json");
    let (code, stdout, stderr) =
        bench(&d, &["--workload", d.join("w").to_str().unwrap(), "--budgets", b.to_str().unwrap(), "--output", out.to_str().unwrap()]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    let report: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    let ops: Vec<&str> = report["operations"].as_array().unwrap().iter().map(|o| o["name"].as_str().unwrap()).collect();
    for op in ["startup", "check_warm_10k", "check_cold_10k", "format_warm_10k", "fix_one_file", "lsp_after_edit", "mcp_rules_for_path", "peak_memory_10k"] {
        assert!(ops.contains(&op), "{op} not measured: {stdout}");
    }
    for o in report["operations"].as_array().unwrap() {
        assert_eq!(o["samples"].as_array().unwrap().len(), 1, "the first of 2 runs is discarded");
        if o["budgeted"] == true {
            assert_eq!(o["verdict"], "unset");
        }
    }
    assert!(report["warm_runs_served_by_daemon"].as_u64().unwrap() >= 6, "{report}");
    assert!(stdout.contains("proposed 0.001 ms, awaiting the decider"), "{stdout}");
    // The one-file operations put the edited file back.
    assert!(!fs::read_to_string(d.join("w/packages/ts/ts0/m5.ts")).unwrap().contains("unformatted"));
}

/// A budget the decider set fails a p95 over it, and only then.
#[test]
fn a_set_budget_fails_a_p95_over_it() {
    let d = temp("set");
    let w = d.join("w");
    let over = budgets(&d, "budget = 0.001\nset_by = \"test\"\nset_on = 2026-10-22\nmeasured_p95 = 0.001");
    let (code, stdout, stderr) = bench(&d, &["--workload", w.to_str().unwrap(), "--only", "startup", "--budgets", over.to_str().unwrap()]);
    assert_eq!(code, 1, "{stdout}{stderr}");
    assert!(stdout.contains("OVER BUDGET"), "{stdout}");
    let within = budgets(&d, "budget = 60000\nset_by = \"test\"\nset_on = 2026-10-22\nmeasured_p95 = 1");
    let (code, stdout, stderr) = bench(&d, &["--workload", w.to_str().unwrap(), "--only", "startup", "--budgets", within.to_str().unwrap()]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(stdout.contains("within"), "{stdout}");
    // A number without the ruling's provenance is a usage error, not a pass.
    let bare = budgets(&d, "budget = 60000");
    let (code, _, stderr) = bench(&d, &["--workload", w.to_str().unwrap(), "--only", "startup", "--budgets", bare.to_str().unwrap()]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("set_by"), "{stderr}");
}

/// A folder that is not a workload is never overwritten.
#[test]
fn a_foreign_folder_is_left_alone() {
    let d = temp("foreign");
    fs::write(d.join("keep.txt"), "mine").unwrap();
    let (code, _, stderr) = bench(&d, &["--workload", d.to_str().unwrap(), "--generate-only"]);
    assert_eq!(code, 2, "{stderr}");
    assert_eq!(fs::read_to_string(d.join("keep.txt")).unwrap(), "mine");
}
