//! Criterion benches over the Phase R confirmed hot lanes. Measurement
//! only: every lane calls the production function unchanged.

// The lib's regular deps ride along as externs of this target; none are
// referenced directly, so mark them used for `unused_crate_dependencies`.
use memchr as _;
use serde as _;
use thiserror as _;
use toml_edit as _;
use yaml_serde as _;

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use serde_json::Value;
use superai_config::document::{self, DocumentKind};
use superai_config::json;
use superai_config::jsonc;
use superai_config::raw_editor;
use superai_config::snapshot;
use superai_config::span_codec::SpanCodec;
use superai_config::transaction;

/// Scratch dir for bench files; removed on drop so runs leave nothing behind.
#[derive(Debug)]
struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "superai-bench-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("bench scratch dir must be creatable");
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(err) = fs::remove_dir_all(&self.dir) {
            eprintln!(
                "superai bench: scratch cleanup failed for {}: {err}",
                self.dir.display()
            );
        }
    }
}

/// Deterministic filler; `index` seeds the byte choices, `len` the length.
fn filler(index: usize, len: usize) -> String {
    let chars: Vec<char> = "abcdefghij".chars().collect();
    let mut out = String::with_capacity(len);
    let mut seed = index;
    for _ in 0..len {
        seed = seed.wrapping_mul(31).wrapping_add(7);
        let pick = seed % chars.len();
        let ch = chars.get(pick).copied().unwrap_or('a');
        out.push(ch);
    }
    out
}

/// Pretty strict JSON of at least `target` bytes; no comments, no trailing commas.
fn clean_json(target: usize) -> String {
    let mut text = String::from("{\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("  \"key");
        text.push_str(&idx.to_string());
        text.push_str("\": \"");
        text.push_str(&filler(idx, 40));
        text.push_str("\",\n");
        idx += 1;
    }
    text.push_str("  \"last\": 0\n}\n");
    text
}

/// JSONC of at least `target` bytes carrying a line comment on every entry.
fn commented_jsonc(target: usize) -> String {
    let mut text = String::from("{\n  /* header comment */\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("  \"key");
        text.push_str(&idx.to_string());
        text.push_str("\": \"");
        text.push_str(&filler(idx, 40));
        text.push_str("\", // note ");
        text.push_str(&idx.to_string());
        text.push('\n');
        idx += 1;
    }
    text.push_str("  \"last\": 0 // tail\n}\n");
    text
}

/// JSONC of at least `target` bytes where every entry line ends in a comma,
/// including the last one before the closing brace.
fn trailing_comma_jsonc(target: usize) -> String {
    let mut text = String::from("{\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("  \"key");
        text.push_str(&idx.to_string());
        text.push_str("\": \"");
        text.push_str(&filler(idx, 40));
        text.push_str("\",\n");
        idx += 1;
    }
    text.push_str("  \"last\": 0,\n}\n");
    text
}

/// Flat TOML document of at least `target` bytes.
fn toml_doc(target: usize) -> String {
    let mut text = String::from("[bench]\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("key");
        text.push_str(&idx.to_string());
        text.push_str(" = \"");
        text.push_str(&filler(idx, 40));
        text.push_str("\"\n");
        idx += 1;
    }
    text
}

/// Flat YAML mapping of at least `target` bytes.
fn yaml_doc(target: usize) -> String {
    let mut text = String::new();
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("key");
        text.push_str(&idx.to_string());
        text.push_str(": ");
        text.push_str(&filler(idx, 40));
        text.push('\n');
        idx += 1;
    }
    text
}

/// Text fragment of at least `target` bytes made of well-formed managed spans.
fn span_fragment(target: usize) -> String {
    let mut text = String::from("# fragment prelude\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("# superai:begin:s");
        text.push_str(&idx.to_string());
        text.push('\n');
        for line in 0..3usize {
            text.push_str("body ");
            text.push_str(&idx.to_string());
            text.push(' ');
            text.push_str(&filler(line.wrapping_add(idx), 18));
            text.push('\n');
        }
        text.push_str("# superai:end:s");
        text.push_str(&idx.to_string());
        text.push('\n');
        idx += 1;
    }
    text
}

/// Line-oriented buffer pair of at least `target` bytes; line `edit_line` differs.
fn lexical_pair(target: usize, edit_line: usize) -> (Vec<u8>, Vec<u8>) {
    let push_line = |out: &mut String, idx: usize, edited: bool| {
        out.push_str("line ");
        out.push_str(&idx.to_string());
        out.push_str(if edited { ": EDITED " } else { ": " });
        out.push_str(&filler(idx, 40));
        out.push('\n');
    };
    let mut old = String::new();
    let mut new = String::with_capacity(target);
    let mut idx = 0usize;
    while old.len() < target {
        push_line(&mut old, idx, false);
        push_line(&mut new, idx, idx == edit_line);
        idx += 1;
    }
    (old.into_bytes(), new.into_bytes())
}

/// Restore the seed bytes and drop accumulated backups so every iteration
/// starts from the same directory state.
fn reset_doc(path: &Path, original: &[u8], file_name: &str) {
    fs::write(path, original).expect("bench doc reset must write");
    let Some(parent) = path.parent() else {
        return;
    };
    let prefix = format!("{file_name}.bak.");
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                drop(fs::remove_file(entry.path()));
            }
        }
    }
}

/// Changing-write round trip through `json::edit`: a toggled value makes
/// every iteration a real commit (backup, temp, fsyncs, verify).
fn bench_json_edit(c: &mut Criterion) {
    let scratch = Scratch::new("json-edit");
    let file = "bench-doc.json";
    let path = scratch.path(file);
    let original = clean_json(16_384).into_bytes();
    fs::write(&path, &original).expect("bench doc seed must write");

    let flip = Cell::new(false);
    let mut group = c.benchmark_group("edit_roundtrip");
    group.bench_function("json_16k_changing_write", |b| {
        b.iter_batched(
            || reset_doc(&path, &original, file),
            |()| {
                let next = !flip.get();
                flip.set(next);
                json::edit(&path, |m| {
                    m.insert("toggled".into(), Value::Bool(next));
                })
                .expect("changing edit must commit");
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

/// Same changing-write round trip through `jsonc::edit` on a comment-free
/// document, so the lossy-write gate passes and the store lane runs.
fn bench_jsonc_edit(c: &mut Criterion) {
    let scratch = Scratch::new("jsonc-edit");
    let file = "bench-doc.jsonc";
    let path = scratch.path(file);
    let original = clean_json(16_384).into_bytes();
    fs::write(&path, &original).expect("bench doc seed must write");

    let flip = Cell::new(false);
    let mut group = c.benchmark_group("edit_roundtrip");
    group.bench_function("jsonc_16k_changing_write", |b| {
        b.iter_batched(
            || reset_doc(&path, &original, file),
            |()| {
                let next = !flip.get();
                flip.set(next);
                jsonc::edit(&path, |m| {
                    m.insert("toggled".into(), Value::Bool(next));
                })
                .expect("changing edit must commit");
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

fn urls_json(target: usize) -> String {
    let mut text = String::from("{\n");
    let mut idx = 0usize;
    while text.len() < target {
        text.push_str("  \"key");
        text.push_str(&idx.to_string());
        text.push_str("\": \"https://host.example/");
        text.push_str(&idx.to_string());
        text.push('/');
        text.push_str(&filler(idx, 20));
        text.push_str("\",\n");
        idx += 1;
    }
    text.push_str("  \"last\": 0\n}\n");
    text
}

fn bench_strip_jsonc(c: &mut Criterion) {
    let cases = [
        ("clean_32k", clean_json(32_768)),
        ("commented_32k", commented_jsonc(32_768)),
        ("trailing_comma_32k", trailing_comma_jsonc(32_768)),
        ("urls_32k", urls_json(32_768)),
    ];
    let mut group = c.benchmark_group("strip_jsonc");
    for (name, text) in &cases {
        let stripped = jsonc::strip_jsonc(text);
        serde_json::from_str::<Value>(&stripped).expect("strip corpus must be strict JSON");
        group.bench_function(*name, |b| {
            b.iter(|| jsonc::strip_jsonc(std::hint::black_box(text)));
        });
    }
    group.finish();
}

/// One full `SpanCodec::validate` pass over a ~64 KiB multi-span fragment.
fn bench_span_validate(c: &mut Criterion) {
    let text = span_fragment(65_536);
    let codec = SpanCodec::default();
    codec
        .validate(&text)
        .expect("span corpus must validate cleanly");
    let mut group = c.benchmark_group("span_codec");
    group.bench_function("validate_64k_fragment", |b| {
        b.iter(|| codec.validate(std::hint::black_box(&text)));
    });
    group.finish();
}

/// `lexical_diff` over ~900 KiB buffers with one locally edited line near
/// the top; the 8 KiB output cap truncates after ~140 compared lines.
fn bench_lexical_diff(c: &mut Criterion) {
    let (old, new) = lexical_pair(921_600, 200);
    let mut group = c.benchmark_group("lexical_diff");
    group.bench_function("900k_local_edit", |b| {
        b.iter(|| raw_editor::lexical_diff(std::hint::black_box(&old), std::hint::black_box(&new)));
    });
    group.finish();
}

/// The staging gate `validate_bytes_for_kind` on ~16 KiB documents of each
/// parseable kind.
fn bench_validate_bytes(c: &mut Criterion) {
    let strict_body = clean_json(16_384).into_bytes();
    let jsonc_body = commented_jsonc(16_384).into_bytes();
    let jsonc_urls_body = urls_json(16_384).into_bytes();
    let toml_body = toml_doc(16_384).into_bytes();
    let yaml_body = yaml_doc(16_384).into_bytes();
    document::validate_bytes_for_kind(&strict_body, DocumentKind::StrictJson, Path::new("b.json"))
        .expect("json corpus must pass the gate");
    document::validate_bytes_for_kind(&jsonc_body, DocumentKind::JsonC, Path::new("b.jsonc"))
        .expect("jsonc corpus must pass the gate");
    document::validate_bytes_for_kind(
        &jsonc_urls_body,
        DocumentKind::JsonC,
        Path::new("b.urls.jsonc"),
    )
    .expect("jsonc urls corpus must pass the gate");
    document::validate_bytes_for_kind(&toml_body, DocumentKind::Toml, Path::new("b.toml"))
        .expect("toml corpus must pass the gate");
    document::validate_bytes_for_kind(&yaml_body, DocumentKind::Yaml, Path::new("b.yaml"))
        .expect("yaml corpus must pass the gate");

    let mut group = c.benchmark_group("validate_bytes_for_kind");
    group.bench_function("strict_json_16k", |b| {
        b.iter(|| {
            document::validate_bytes_for_kind(
                std::hint::black_box(&strict_body),
                DocumentKind::StrictJson,
                Path::new("b.json"),
            )
        });
    });
    group.bench_function("jsonc_16k", |b| {
        b.iter(|| {
            document::validate_bytes_for_kind(
                std::hint::black_box(&jsonc_body),
                DocumentKind::JsonC,
                Path::new("b.jsonc"),
            )
        });
    });
    group.bench_function("jsonc_urls_16k", |b| {
        b.iter(|| {
            document::validate_bytes_for_kind(
                std::hint::black_box(&jsonc_urls_body),
                DocumentKind::JsonC,
                Path::new("b.urls.jsonc"),
            )
        });
    });
    group.bench_function("toml_16k", |b| {
        b.iter(|| {
            document::validate_bytes_for_kind(
                std::hint::black_box(&toml_body),
                DocumentKind::Toml,
                Path::new("b.toml"),
            )
        });
    });
    group.bench_function("yaml_16k", |b| {
        b.iter(|| {
            document::validate_bytes_for_kind(
                std::hint::black_box(&yaml_body),
                DocumentKind::Yaml,
                Path::new("b.yaml"),
            )
        });
    });
    group.finish();
}

/// The per-commit case-fold scan over a populated directory with no
/// case variant present (the common case: full scan, no collision).
fn bench_collision_scan(c: &mut Criterion) {
    let scratch = Scratch::new("collision");
    for i in 0..200usize {
        let name = format!("entry-{i:03}.conf");
        fs::write(scratch.path(&name), b"bench\n").expect("collision dir entry must write");
    }
    let target = scratch.path("settings.json");
    assert!(
        transaction::case_fold_collision_in_dir(&target).is_none(),
        "no case variant was seeded, so the scan must find none"
    );
    let mut group = c.benchmark_group("collision_scan");
    group.bench_function("case_fold_200_entries", |b| {
        b.iter(|| transaction::case_fold_collision_in_dir(std::hint::black_box(&target)));
    });
    group.finish();
}

/// Fresh `snapshot()` of a ~100 KiB file: stat, full read, `SipHash` digest.
fn bench_snapshot_digest(c: &mut Criterion) {
    let scratch = Scratch::new("snapshot");
    let path = scratch.path("snapshot-target.json");
    fs::write(&path, clean_json(102_400)).expect("snapshot target must write");
    let first = snapshot::snapshot(&path);
    assert!(
        first.digest.is_some(),
        "snapshot must digest a readable file"
    );
    let mut group = c.benchmark_group("snapshot");
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("digest_100k_file", |b| {
        b.iter(|| snapshot::snapshot(std::hint::black_box(&path)));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_strip_jsonc,
    bench_json_edit,
    bench_jsonc_edit,
    bench_span_validate,
    bench_lexical_diff,
    bench_validate_bytes,
    bench_collision_scan,
    bench_snapshot_digest,
);
criterion_main!(benches);
