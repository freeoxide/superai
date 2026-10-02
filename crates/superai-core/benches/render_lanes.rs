//! Criterion benches over the provider-render/template hot lanes.
//! Measurement only: every lane calls the production function unchanged.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// The lib's regular deps ride along as externs of this target; none are
// referenced directly, so mark them used for `unused_crate_dependencies`.
use duct as _;
use hex as _;
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use superai_config as _;
use thiserror as _;
use toml_edit as _;
use ureq as _;
use yaml_serde as _;

use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::BTreeMap;
use superai_core::Instance;
use superai_core::adapters::codex_cli::CodexCliAdapter;
use superai_core::ids::{HarnessId, InstanceId, InstanceName, ProviderId, TemplateId};
use superai_core::paths::AbsolutePath;
use superai_core::provider::{self, BUNDLED_PROVIDERS_JSON};
use superai_core::provider_render::{self, ProviderChange};
use superai_core::state::{InstanceOrigin, Isolation, Ownership};
use superai_core::template::{OwnedPatch, TEMPLATE_SCHEMA_VERSION, Template, TemplateStatus};

struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "superai-render-bench-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("bench scratch dir must be creatable");
        Self { dir }
    }

    fn instance(&self, name: &str) -> Instance {
        let config_root = self.dir.join(name);
        fs::create_dir_all(&config_root).expect("bench instance root must be creatable");
        Instance {
            id: InstanceId::new(&format!("id-{name}")).expect("bench instance id is valid"),
            name: InstanceName::new(name).expect("bench instance name is valid"),
            harness: HarnessId::new("codex-cli").expect("bench harness id is valid"),
            config_root: AbsolutePath::from_path(&config_root)
                .expect("bench scratch path is absolute"),
            binary: None,
            wrapper: None,
            isolation: Isolation::RelocatedRoot,
            origin: InstanceOrigin::Created,
            ownership: Ownership::SuperaiCreated,
            template: None,
            created_at: "2026-08-26T12:00:00Z".to_owned(),
            adapter_revision: "0.1.0".to_owned(),
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.dir) {
            eprintln!("bench scratch cleanup failed: {e}");
        }
    }
}

const CODEX_TOML: &str = "model_provider = \"glm\"\nmodel = \"glm-4.5\"\n\n\
[model_providers.glm]\nname = \"GLM\"\nbase_url = \"https://open.bigmodel.cn/api/paas/v4\"\n\
env_key = \"GLM_API_KEY\"\nwire_api = \"chat\"\n\n[foreign_root]\nkeep = true\n";

fn bench_bundled_providers(c: &mut Criterion) {
    let mut group = c.benchmark_group("bundled_providers");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("uncached_parse_and_validate", |b| {
        b.iter(|| {
            let providers: Vec<provider::ProviderDefinition> =
                serde_json::from_str(BUNDLED_PROVIDERS_JSON).expect("bundled asset must parse");
            for p in &providers {
                p.validate().expect("bundled provider must validate");
            }
            providers.len()
        });
    });
    group.bench_function("load_bundled_providers_cached", |b| {
        b.iter(|| {
            provider::load_bundled_providers()
                .expect("bundled providers must load")
                .len()
        });
    });
    group.finish();
}

fn bench_inspect_codex_toml(c: &mut Criterion) {
    let scratch = Scratch::new("inspect");
    let instance = scratch.instance("inspect");
    fs::write(
        instance.config_root.as_path().join("config.toml"),
        CODEX_TOML,
    )
    .expect("bench config seed must write");
    let adapter = CodexCliAdapter::new().expect("codex adapter constructs");
    let providers = provider::load_bundled_providers().expect("bundled providers must load");
    let mut group = c.benchmark_group("provider_render");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("inspect_effective_provider_codex_toml", |b| {
        b.iter(|| {
            provider_render::inspect_effective_provider(&instance, &adapter, &providers)
                .expect("inspect must succeed on the seeded config")
                .model_roles
                .len()
        });
    });
    let bundled_glm = providers
        .iter()
        .find(|p| p.id.as_str() == "glm")
        .expect("bundled glm provider exists");
    let change = ProviderChange::AddOrUpdate {
        provider: bundled_glm,
    };
    group.bench_function("preview_provider_change_codex", |b| {
        b.iter(|| {
            provider_render::preview_provider_change(&instance, &adapter, &change)
                .edits
                .len()
        });
    });
    group.finish();
}

fn bench_validate_against_adapter(c: &mut Criterion) {
    let adapter = CodexCliAdapter::new().expect("codex adapter constructs");
    let template = Template {
        schema_version: TEMPLATE_SCHEMA_VERSION,
        id: TemplateId::new("codex-glm").expect("bench template id is valid"),
        version: "1.0.0".to_owned(),
        harness: HarnessId::new("codex-cli").expect("bench harness id is valid"),
        provider: ProviderId::new("glm").expect("bench provider id is valid"),
        label: "Codex on GLM".to_owned(),
        status: TemplateStatus::Active,
        inputs: Vec::new(),
        patches: (0..12)
            .map(|i| OwnedPatch {
                selector: format!("model_providers.glm.field_{i}"),
                value: serde_json::json!(i),
            })
            .collect(),
        wrapper_env: BTreeMap::new(),
        wrapper_args: Vec::new(),
        assets: Vec::new(),
        capability_map: BTreeMap::new(),
        migration_notes: Vec::new(),
        digest: "a".repeat(64),
        harness_version_req: None,
        provider_protocol: None,
        replacement: None,
    };
    let mut group = c.benchmark_group("template");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("validate_against_adapter_codex_12_patches", |b| {
        b.iter(|| {
            template
                .validate_against_adapter(&adapter)
                .expect("bench template must validate");
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_bundled_providers,
    bench_inspect_codex_toml,
    bench_validate_against_adapter,
);
criterion_main!(benches);
