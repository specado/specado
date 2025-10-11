// Reference corpus lives at ../../specado-newest-legacy/golden-corpus (one level above repo root).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specado_core::hot_reload::ProviderCache;
use specado_core::translate;
use specado_core::types::{LossinessReport, PromptSpec};

#[derive(Debug, Deserialize)]
struct GoldenCase {
    provider: String,
    prompt: PromptSpec,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct GoldenSnapshot {
    provider_payload: Value,
    lossiness: LossinessReport,
}

#[test]
fn golden_cases_match_snapshots() {
    let base = cases_dir();
    let case_dirs = collect_case_dirs(&base);
    assert!(
        !case_dirs.is_empty(),
        "No golden test cases found under {}",
        base.display()
    );

    for dir in case_dirs {
        run_case(&dir);
    }
}

fn run_case(dir: &Path) {
    let case_path = dir.join("case.json");
    let snapshot_path = dir.join("snapshot.json");

    let case_data = fs::read_to_string(&case_path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {}", case_path.display(), err));
    let case: GoldenCase = serde_json::from_str(&case_data)
        .unwrap_or_else(|err| panic!("Failed to parse {}: {}", case_path.display(), err));

    let updating = std::env::var("UPDATE_GOLDEN").is_ok();
    if updating {
        if let Some(note) = &case.description {
            eprintln!("Updating snapshot for {} ({})", dir.display(), note);
        } else {
            eprintln!("Updating snapshot for {}", dir.display());
        }
    }

    let provider = load_provider(&case.provider);
    let (payload, lossiness) = translate(&case.prompt, &provider)
        .unwrap_or_else(|err| panic!("Translation failed for {}: {}", case_path.display(), err));

    let snapshot = GoldenSnapshot {
        provider_payload: payload,
        lossiness,
    };

    if updating {
        write_snapshot(&snapshot_path, &snapshot);
        return;
    }

    let expected = read_snapshot(&snapshot_path);
    assert_eq!(
        snapshot,
        expected,
        "Golden snapshot mismatch for {}",
        dir.display()
    );
}

fn load_provider(relative: &str) -> specado_core::ProviderSpec {
    let path = provider_root().join(relative);
    ProviderCache::new()
        .load_or_read(&path)
        .unwrap_or_else(|err| panic!("Failed to load provider spec {}: {}", path.display(), err))
}

fn read_snapshot(path: &Path) -> GoldenSnapshot {
    let data = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "Failed to read snapshot {}: {} (hint: run with UPDATE_GOLDEN=1)",
            path.display(),
            err
        )
    });
    serde_json::from_str(&data)
        .unwrap_or_else(|err| panic!("Failed to parse snapshot {}: {}", path.display(), err))
}

fn write_snapshot(path: &Path, snapshot: &GoldenSnapshot) {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            panic!(
                "Failed to create snapshot directory {}: {}",
                parent.display(),
                err
            );
        }
    }

    let content = serde_json::to_string_pretty(snapshot)
        .unwrap_or_else(|err| panic!("Failed to serialize snapshot: {}", err));
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("Failed to write snapshot {}: {}", path.display(), err));
}

fn collect_case_dirs(base: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if !base.exists() {
        return dirs;
    }

    fn visit(current: &Path, acc: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("case.json").is_file() {
                    acc.push(path);
                } else {
                    visit(&path, acc);
                }
            }
        }
    }

    visit(base, &mut dirs);
    dirs.sort();
    dirs
}

fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/cases")
}

fn provider_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../specado-providers/providers")
}
