# ImageVue Design Spec

Date: 2026-07-02
Status: Draft v1

## Overview

ImageVue is a cross-platform (macOS + Windows) desktop GUI for browsing OCI image registries and Helm chart repositories. Users can add multiple registry connections, browse catalogs and tags, inspect image manifests/configs, and pull images as docker-save-compatible tarballs or charts as `.tgz` files.

Built with Tauri (Rust + Vue 3 + TypeScript), inspired by the UI patterns of elasticvue.

---

## 1. Architecture

### Layer Stack

**Web UI (Vue 3 + Naive UI + Pinia + Vue Router)**
- ConnectionList / RepoBrowser / TagList / ManifestView
- PullProgress / Settings / LoginDialog

**Tauri Commands Layer (Rust)**
- Thin shell: parameter validation, event emission, streaming progress push

**Domain Services (Rust)**
- RegistryService, RepoService, ManifestService, PullService, AuthService, CredentialStore

**OCI Client (Rust)**
- HttpClient, AuthInterceptor, TokenCache
- DistributionClient (catalog/tags/manifests/blobs)
- ManifestParser, DescriptorParser

**Storage & Output**
- SqliteStore (connections/cache/history), KeyringStore (OS keychain for credentials)

### Design Principles

1. **Unidirectional dependency**: UI → Commands → Services → OCI Client → Storage. OCI Client has zero Tauri dependency, testable independently.
2. **State ownership**: UI state in Pinia, connections/history in SQLite, runtime tokens in memory (TokenCache), credentials in OS Keychain.
3. **Streaming progress**: Large file downloads use `tokio::sync::mpsc` to push byte progression to frontend.
4. **Offline capable**: Cache recently browsed manifests/configs; viewable offline (pull requires online).
5. **Pluggable auth**: AuthInterceptor uses `RegistryKind` enum to dispatch per-registry token strategies (Docker Hub / GHCR / Quay / GCR / Generic).

---

## 2. Module / Crate Structure

### Rust (`src-tauri/`)

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── icons/
└── src/
    ├── main.rs                    # Entry: setup, plugins, command registration
    ├── error.rs                   # AppError / RegistryError unified types
    ├── state.rs                   # AppState: connection pool/cache/SQLite handle
    ├── commands/                  # Tauri command handlers (thin)
    │   ├── mod.rs
    │   ├── connection.rs          # list/add/update/delete/connect
    │   ├── catalog.rs             # list_repos, search_repos
    │   ├── tag.rs                 # list_tags, get_tag
    │   ├── manifest.rs            # get_manifest, get_config
    │   ├── pull.rs                # start_pull, cancel_pull, list_pulls
    │   └── auth.rs                # begin_login, complete_oauth_callback
    ├── services/                  # Business orchestration
    │   ├── mod.rs
    │   ├── registry_service.rs    # Health check, catalog, connection state machine
    │   ├── repo_service.rs        # Repo search, cache
    │   ├── manifest_service.rs    # Manifest/config parsing, referrers
    │   ├── pull_service.rs        # Pull orchestration + progress events + tar assembly
    │   └── auth_service.rs        # OAuth flow orchestration, token refresh
    ├── oci/                       # Protocol layer (no Tauri dependency)
    │   ├── mod.rs
    │   ├── client.rs              # DistributionClient
    │   ├── http.rs                # reqwest wrapper + retry
    │   ├── auth.rs                # AuthInterceptor (per-registry strategy)
    │   ├── token.rs               # TokenCache (in-memory, TTL)
    │   ├── types.rs               # Manifest/Descriptor/Platform structs
    │   └── parser.rs              # OCI manifest JSON parsing + validation
    ├── tarball/                   # docker save / chart tgz output
    │   ├── mod.rs
    │   ├── builder.rs             # TarballBuilder / ChartBuilder
    │   └── repack.rs              # Blob stream repacking
    ├── storage/                   # Persistence
    │   ├── mod.rs
    │   ├── sqlite.rs              # rusqlite + migrations
    │   ├── repo.rs                # ConnectionRepo / HistoryRepo / CacheRepo
    │   └── keyring.rs             # OS keychain wrapper
    └── events.rs                  # Frontend event payload definitions
```

### Frontend (`src/`)

```
src/
├── main.ts
├── App.vue
├── router/index.ts
├── stores/                       # Pinia
│   ├── connections.ts
│   ├── repositories.ts
│   ├── manifest.ts
│   └── pull.ts
├── views/
│   ├── ConnectionListView.vue
│   ├── RegistryView.vue
│   ├── RepositoryView.vue
│   ├── TagDetailView.vue
│   └── SettingsView.vue
├── components/
│   ├── ConnectionCard.vue
│   ├── RepoTree.vue
│   ├── TagTable.vue
│   ├── ManifestViewer.vue
│   ├── LayerList.vue
│   ├── PullProgressDrawer.vue
│   └── LoginDialog.vue
├── composables/
│   ├── useTauriCommand.ts
│   └── usePullProgress.ts
├── i18n/
│   ├── zh-CN.ts
│   └── en-US.ts
├── theme/naive-ui.ts
└── lib/
    ├── api.ts
    └── types.ts                   # From specta auto-generation
```

### Type Synchronization

Use **specta** + **tauri-specta** to auto-generate TypeScript types from Rust structs:
- `#[derive(Specta)]` on shared types
- Build output: `src/lib/bindings.ts`
- Commands use `#[tauri::command]`, specta exports typed invoke wrappers
- Eliminates hand-written type duplication

### Core Dependencies

```toml
# Cargo.toml
tauri = { version = "2", features = ["protocol-asset"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls", "stream", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
keyring = "3"
thiserror = "1"
anyhow = "1"
tar = "0.4"
flate2 = "1"
sha2 = "0.10"
hex = "0.4"
url = "2"
chrono = { version = "0.4", features = ["serde"] }
specta = { version = "=2.0.0-rc.20", features = ["derive"] }
specta-typescript = "0.0.9"
tauri-specta = { version = "=2.0.0-rc.20", features = ["derive", "typescript"] }
```

```json
{
  "dependencies": {
    "vue": "^3.5",
    "vue-router": "^4.4",
    "pinia": "^2.2",
    "naive-ui": "^2.40",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-deep-link": "^2",
    "@vicons/ionicons5": "^0.13"
  },
  "devDependencies": {
    "vite": "^5",
    "@vitejs/plugin-vue": "^5",
    "typescript": "^5.5",
    "naive-ui": "^2.40",
    "@tauri-apps/cli": "^2"
  }
}
```

---

## 3. Data Model

### Core Rust Types

```rust
// Registry Connection
#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConnection {
    pub id: Uuid,
    pub name: String,
    pub kind: RegistryKind,
    pub endpoint: String,
    pub insecure: bool,
    pub credential_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Specta)]
#[serde(rename_all = "kebab-case")]
pub enum RegistryKind {
    DockerHub,
    Ghcr,
    Quay,
    Gcr,
    Generic,
}

// Domain types
#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct Repository { pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub name: String,
    pub digest: String,
    pub size: u64,
    pub updated_at: Option<DateTime<Utc>>,
    pub os: Option<String>,
    pub architecture: Option<String>,
    pub artifact_kind: ArtifactKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactKind { Image, HelmChart }

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSummary {
    pub digest: String,
    pub media_type: String,
    pub schema_version: u32,
    pub kind: ManifestKind,
    pub total_size: u64,
    pub layer_count: u32,
    pub config_descriptor: Option<Descriptor>,
    pub layer_descriptors: Vec<Descriptor>,
    pub platforms: Vec<Platform>,
    pub raw_json: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Specta)]
pub enum ManifestKind { DockerV2, DockerV1, OciIndex, OciManifest, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    pub urls: Option<Vec<String>>,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct Platform { pub architecture: String, pub os: String, pub variant: Option<String>, pub os_version: Option<String>, pub os_features: Option<Vec<String>> }

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub digest: String, pub architecture: String, pub os: String,
    pub created: Option<DateTime<Utc>>, pub author: Option<String>,
    pub env: Vec<String>, pub cmd: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>, pub working_dir: Option<String>,
    pub exposed_ports: HashMap<String, serde_json::Value>,
    pub labels: HashMap<String, String>,
    pub history: Vec<HistoryEntry>, pub rootfs: RootFs, pub raw_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub created: Option<DateTime<Utc>>, pub author: Option<String>,
    pub created_by: String, pub empty_layer: Option<bool>, pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct RootFs { pub r#type: String, pub diff_ids: Vec<String> }

// Pull / Save
#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum PullJob {
    Queued { id: Uuid, connection_id: Uuid, repo: String, tag: String },
    Running { id: Uuid, progress: PullProgress },
    Completed { id: Uuid, output_path: String, size_bytes: u64, duration_ms: u64 },
    Failed { id: Uuid, error: String },
    Cancelled { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct PullProgress {
    pub phase: PullPhase, pub bytes_downloaded: u64, pub bytes_total: u64,
    pub current_layer: Option<String>, pub layer_index: u32, pub layer_count: u32,
    pub speed_bps: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Specta)]
pub enum PullPhase { ResolvingManifest, FetchingLayers, AssemblingTar, WritingFile }

#[derive(Debug, Clone, Serialize, Deserialize, Specta)]
#[serde(rename_all = "camelCase")]
pub struct PullOptions {
    pub connection_id: Uuid, pub repo: String, pub tag: String,
    pub platform: Option<Platform>, pub output_dir: String,
    pub output_filename: Option<String>,
}
```

### SQLite Schema

```sql
CREATE TABLE connections (
    id TEXT PRIMARY KEY, name TEXT NOT NULL,
    kind TEXT NOT NULL, endpoint TEXT NOT NULL,
    insecure INTEGER NOT NULL, credential_ref TEXT,
    created_at TEXT NOT NULL, last_connected_at TEXT
);

CREATE TABLE repo_cache (
    connection_id TEXT NOT NULL, query TEXT NOT NULL,
    payload_json TEXT NOT NULL, fetched_at TEXT NOT NULL,
    PRIMARY KEY (connection_id, query)
);

CREATE TABLE tag_cache (
    connection_id TEXT NOT NULL, repository TEXT NOT NULL,
    payload_json TEXT NOT NULL, fetched_at TEXT NOT NULL,
    PRIMARY KEY (connection_id, repository)
);

CREATE TABLE manifest_cache (
    digest TEXT PRIMARY KEY, payload_json TEXT NOT NULL,
    size_bytes INTEGER NOT NULL, fetched_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL
);

CREATE TABLE pull_history (
    id TEXT PRIMARY KEY, connection_id TEXT NOT NULL,
    repo TEXT NOT NULL, tag TEXT NOT NULL, digest TEXT NOT NULL,
    output_path TEXT NOT NULL, size_bytes INTEGER NOT NULL,
    started_at TEXT NOT NULL, finished_at TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
```

### Keyring Storage

```
service:  "imagevue"
account:  "cred:{connection_id}"    # username / PAT
account:  "oauth:{connection_id}"   # OAuth refresh token + scope + expiry
```

Access tokens stored in memory only (TokenCache with TTL). Refresh tokens persisted to keychain.

### Artifact Kind Detection

- `MediaType = application/vnd.docker.distribution.manifest.v2+json` or `application/vnd.oci.image.manifest.v1+json` with `config.mediaType = application/vnd.docker.container.image.v1+json` → `ArtifactKind::Image`
- `MediaType = application/vnd.oci.image.manifest.v1+json` with `config.mediaType = application/vnd.cncf.helm.config.v1+json` → `ArtifactKind::HelmChart`
- Otherwise → storage-specific display (generic OCI artifact, shown but not pullable in V1)

---

## 4. OCI Client Design

### DistributionClient

```rust
impl DistributionClient {
    pub fn new(connection: &RegistryConnection, http: HttpClient) -> Self;
    pub async fn ping(&self) -> Result<(), RegistryError>;
    pub async fn list_repositories(&self, opts: ListOptions) -> Result<Page<Repository>>;
    pub async fn search_repositories(&self, query: &str, limit: u32) -> Result<Vec<Repository>>;
    pub async fn list_tags(&self, repo: &str, opts: ListOptions) -> Result<Page<Tag>>;
    pub async fn get_manifest(&self, repo: &str, reference: &str) -> Result<(ManifestSummary, Vec<u8>)>;
    pub async fn get_blob(&self, repo: &str, digest: &str) -> Result<Bytes>;
    pub async fn stream_blob(&self, repo: &str, digest: &str) -> Result<BlobStream>;
    pub async fn blob_exists(&self, repo: &str, digest: &str) -> Result<bool>;
}

pub struct Page<T> { pub items: Vec<T>, pub next_cursor: Option<String> }
pub struct ListOptions { pub limit: u32, pub last: Option<String> }
```

### HTTP Layer

```rust
pub struct HttpClient {
    inner: reqwest::Client, user_agent: String,
    max_retries: u32, retry_backoff: ExponentialBackoff,
    timeout_connect: Duration, timeout_read: Duration,
}
```

- Retry: network errors, 5xx, 429 only (401 handled by AuthInterceptor)
- Backoff: 200ms → 400ms → 800ms → max 5s
- `Accept` header includes all manifest media types

### Auth Flow

```rust
pub struct AuthInterceptor {
    kind: RegistryKind, token_cache: Arc<TokenCache>,
    credential_provider: Arc<dyn CredentialProvider>,
    http_for_token: HttpClient,
}
```

**Per-registry token strategies:**

| RegistryKind | Token Endpoint | Method | Credential |
|---|---|---|---|
| DockerHub | `auth.docker.io/token?service=registry.docker.io` | Bearer challenge | PAT / refresh_token |
| Ghcr | `ghcr.io/token` | Bearer challenge | OAuth refresh_token |
| Quay | `quay.io/v2/auth` | Bearer challenge | OAuth refresh_token |
| Gcr | `gcr.io/v2/token` | Bearer challenge | OAuth refresh_token |
| Generic | `{endpoint}/token` or WWW-Authenticate challenge | Per response | username/password or PAT |

**OAuth Desktop Flow:**

```
[UI: "Login with GitHub"] → auth_service::begin_oauth
  1. Generate state + code_verifier (PKCE)
  2. Open system browser to authorization endpoint
  3. Register deep-link scheme (imagevue://oauth-callback) or localhost loopback

[System browser: user authorizes] → redirects to imagevue://oauth-callback?code=...&state=...

[tauri-plugin-deep-link triggers event] → auth_service::complete_oauth
  1. Validate state
  2. POST to token endpoint (code + code_verifier → access_token + refresh_token)
  3. Store refresh_token in keyring
  4. Ping registry to verify connectivity
  5. Mark connection credential_ref = "oauth:{uuid}"
```

System browser approach preferred over embedded webview (better 2FA support, no cookie isolation issues).

### TokenCache

In-memory TTL cache scoped by `(endpoint, repository, scope)`. Invalidated on 401 response, triggers one reauth retry.

### Manifest Parsing & Validation

- Size validation: sum(layer.size) == config.size
- Digest validation: re-sha256 raw bytes, compare with declared digest
- Media type sniffing: server Content-Type takes precedence
- Index manifest: enumerate platforms without downloading child manifests
- Error: structured ManifestError returned to frontend

### Error Model

```rust
#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    Http(#[from] HttpError), Auth(#[from] AuthError),
    NotFound(String), InvalidManifest(String),
    UnsupportedMediaType(String),
    RateLimited { retry_after_ms: u64 },
    ServerError { status: u16, body: String },
}
```

---

## 5. Pull & Save

### Docker Tarball Format (Images)

Compatible with `docker load` / `podman load`:

```
{repo}-{tag}.tar
├── manifest.json
├── repositories
├── {config_digest_stripped}.json
├── {layer_digest_with_prefix}/layer.tar
└── ...
```

- `manifest.json`: `[{Config, Repos, Layers}]` with digest hex (no `sha256:` prefix for Config, full digest path for Layers)
- `repositories`: legacy format for old docker versions

### Chart `.tgz` Format (Helm Charts)

For OCI helm charts, save the chart layer blob directly as `{repo}-{version}.tgz` (equivalent to `helm pull`).

### TarballBuilder

```rust
pub struct TarballBuilder {
    writer: BufWriter<File>, repo: String, tags: Vec<String>,
    manifest_json: serde_json::Value, config_descriptor: Descriptor,
    layer_descriptors: Vec<Descriptor>, progress_tx: Option<mpsc::Sender<TarProgress>>,
    artifact_kind: ArtifactKind,
}

impl TarballBuilder {
    pub async fn new(output_path: &Path, opts: BuildOptions) -> Result<Self>;
    pub async fn finalize(self) -> Result<TarballResult>;
}
```

### Known Hazards

1. **Empty layers** (size=0): must be included in Layers array to preserve history
2. **Multi-platform index**: user selects one platform before pull (V1 single-arch only)
3. **OCI → docker format**: OCI config is docker-load-compatible in docker 24+
4. **Uncompressed OCI layers**: detect + regzip to gzip
5. **zstd layers**: decompress + regzip to gzip (docker load does not accept zstd)
6. **diff_id vs digest**: downloaded blob digest ≠ tar content sha256; must decompress and verify against config.rootfs.diff_ids

### Rezip Pipeline

```rust
fn rezip_layer(data: &[u8]) -> Result<Vec<u8>> {
    let decompressed = match sniff_compression(data) {
        Gzip => decompress_gzip(data),
        Zstd => decompress_zstd(data),
        None => data.to_vec(),
    };
    // Gzip output (level 6)
    let mut out = Vec::new();
    let mut encoder = GzEncoder::new(&mut out, Compression::new(6));
    encoder.write_all(&decompressed)?;
    encoder.finish()?;
    Ok(out)
}
```

### Pull Orchestration

```
[User clicks "Pull"]
  → start_pull command → create PullJob::Queued → spawn background task
    Step 1: get_manifest (resolve tag)
    Step 2: pick platform (for index manifests)
    Step 3: pre-fetch config + list layers
    Step 4: concurrent download layers (semaphore=4), sha256 verify
    Step 5: assemble tarball/chart
    Step 6: move to output directory
    Step 7: emit Completed event
```

Progress events pushed via Tauri events every 256 KiB.

### Concurrency & Caching

- Layer download: `tokio::sync::Semaphore(4)`
- Cancellation: `tokio_util::CancellationToken`
- Blob cache: `~/.cache/imagevue/blobs/{digest}` (LRU 2 GB limit)

---

## 6. UI Structure

### Global Layout

Top bar (56px, fixed) + left sidebar (240px, collapsible to 56px) + main content area.

Left sidebar shows registry connections (green dot = authenticated, gray circle = logged out). App header has global search + settings button.

### Route Table

| Path | View | Description |
|---|---|---|
| `/` | redirect | → `/connections` |
| `/connections` | ConnectionListView | Default home, card grid |
| `/connections/new` | ConnectionEditView | Add new registry |
| `/connections/:id/edit` | ConnectionEditView | Edit connection |
| `/r/:id` | RegistryView | Browse catalog, search repos |
| `/r/:id/repo/:repo(*)/tags` | RepositoryView | Tag list with details |
| `/r/:id/repo/:repo(*)/tag/:tag` | TagDetailView | Manifest viewer + config + pull |
| `/settings` | SettingsView | Global preferences |
| `/about` | AboutView | Version, credits |

### Key Screens

**ConnectionListView**: Card grid (3/2/1 responsive columns). Cards show name, kind, status, last seen. "Browse" navigates to RegistryView.

**ConnectionEditView**: Form with dynamic fields per RegistryKind. "Test Connection" button. OAuth flow launched on save.

**RegistryView**: Catalog browser with search (debounce 300ms). Data table with pagination. Filter options per registry kind (Docker Hub: official/verified/private).

**RepositoryView**: Tag list sorted by update time. OS/Arch extracted from manifest config (async load with spinner).

**TagDetailView**: Three sections:
- Manifest JSON tree (collapsible)
- Config field table (architecture, cmd, env, labels, history)
- Layers table (index, digest short, size, media type, created_by)
- "Pull" button → PullProgressDrawer

**PullProgressDrawer**: Global drawer (bottom-right) with tabs for concurrent pulls. Phase progress, speed, ETA. Cancel button.

### Pinia Stores

- `connections`: CRUD, active connection, test connection
- `repositories`: catalog by registry, pagination cursor, search
- `tags`: tag list by (registry, repo), loading state
- `manifest`: summary + config + raw JSON by (registry, repo, ref)
- `pull`: active jobs, progress events, cancellation

### Settings

- defaultDownloadDir
- maxConcurrentLayers (default 4)
- blobCacheSizeMb (default 2048)
- theme (auto/light/dark)
- language (zh-CN/en-US)

### Theme & i18n

- Naive UI: default blue accent (`#2080f0`), dark/light
- vue-i18n: V1 ships `zh-CN` and `en-US`

---

## 7. Error Handling

### Error Layers

```
Frontend (UI) ← AppError JSON (tagged union)
  Tauri commands (thin) ← AppError
    Domain services ← RegistryError / AuthError / HttpError / StorageError
      OCI client / storage
```

### AppError Serialization

Tagged union sent to frontend. Each variant has `kind` discriminator:

- **network**: { message, retryable, retryAfterMs } — toast + retry button
- **unauthorized**: { message, registryKind, connectionId } — trigger reauth
- **notFound**: { resource, identifier } — error toast, stay on page
- **rateLimited**: { retryAfterMs, scope } — countdown toast, auto-retry
- **invalidManifest**: { digest, reason } — error toast with details
- **unsupportedMediaType**: { mediaType } — warning toast
- **digestMismatch**: { expected, actual } — error toast, auto-retry layer
- **diskFull**: { path, neededBytes, availableBytes } — blocking dialog
- **cancelled**: silent (user-initiated)
- **serverError**: { status, body, registryKind } — error toast
- **storage**: { message, key } — error toast
- **invalidInput**: { message } — warning toast (inline form validation)
- **internal**: { message } — error toast + "View Logs" link

### Pull-Specific Error Handling

Pull job errors include phase context. Drawer shows which step failed. "Retry" restarts from manifest fetch. "View Logs" opens system log file.

### 401 Auto-Recovery

For OAuth connections with valid refresh token: auto-refresh token and retry (once). If second 401 fails → emit unauthorized to UI.

### Logging

- `tracing` + `tracing-subscriber` (env filter, default `info`)
- Log file: `{app_data_dir}/imagevue/imagevue.log` (rotating daily, 7 days retention)
- Frontend "View Logs" opens with system text editor via `tauri-plugin-shell`

---

## 8. Testing Strategy

### Test Pyramid

**E2E (Playwright + tauri-driver)**: 5-10 key user journeys (add connection → browse, browse → tag → pull → verify, pull → docker load, OAuth mock, cancel pull, dark mode toggle, offline cache)

**Integration (Rust)**: Real registry subset (local `registry:2` container in CI). Pull alpine:3.19, verify docker load. Pull helm chart, verify .tgz.

**Unit (Rust + Vitest)**: Manifest parser, auth interceptor, tarball builder, rezip, error serialization (snapshot), stores, composables

### Fixtures

- Image fixtures: `tests/fixtures/images/` (crane-exported small alpine tarballs)
- HTTP recordings: `tests/fixtures/http/` (wiremock format for Docker Hub / GHCR / Quay)
- Offline by default in CI; integration tests behind feature flag

### CI Matrix

| OS | Arch | Task |
|---|---|---|
| macos-14 | x86_64 | cargo test + vitest + build |
| macos-14-arm | aarch64 | cargo test + vitest + build |
| windows-2022 | x86_64 | cargo test + vitest + build |
| ubuntu-22.04 | x86_64 | cargo test + vitest + build |

### Coverage Targets

| Module | Line | Branch |
|---|---|---|
| oci::parser | ≥95% | ≥90% |
| oci::auth | ≥90% | ≥85% |
| tarball::* | ≥90% | ≥85% |
| services::* | ≥80% | ≥75% |
| commands::* | ≥70% | ≥65% |
| Vue stores | ≥85% | — |

`cargo-llvm-cov` + `vitest --coverage`. PR blocking threshold.

### Performance Benchmarks

- Manifest parsing < 5ms (typical 5-layer image)
- Layer download ≈ physical bandwidth
- 1 GB image assembly < 30s (M1 MacBook)
- Cold start → interactive < 1.5s

---

## 9. Packaging & Distribution

### Tauri Bundle Config

```json
{
  "productName": "ImageVue",
  "version": "0.1.0",
  "identifier": "space.bkverse.imagevue",
  "bundle": {
    "targets": ["msi", "dmg", "appimage", "deb"],
    "category": "DeveloperTool",
    "shortDescription": "OCI image & chart registry viewer",
    "longDescription": "ImageVue is a cross-platform desktop GUI for browsing OCI image and Helm chart registries.",
    "publisher": "bkverse",
    "homepage": "https://github.com/bkverse/imagevue",
    "createUpdaterArtifacts": true,
    "macOS": { "minimumSystemVersion": "11.0" },
    "windows": { "wix": { "language": "en-US" } }
  }
}
```

### Artifacts

| Platform | Format | Size Target | Channel |
|---|---|---|---|
| macOS Intel | .dmg (x86_64) | ≤15 MB | Releases + brew cask |
| macOS Apple Silicon | .dmg (aarch64) | ≤15 MB | Releases + brew cask |
| Windows x64 | .msi | ≤20 MB | Releases + winget |
| Linux x64 | .AppImage, .deb | ≤20 MB | Releases (best-effort) |

### Auto-Update

`tauri-plugin-updater` with GitHub Releases source. Manual check + prompt (no force-update during pull). Backs up SQLite before upgrade.

### Code Signing

V1: ad-hoc signing (macOS), unsigned (Windows). Noted in README limitations. Future: Apple Developer ID + Authenticode EV.

### CI/CD

GitHub Actions: `ci.yml` (lint+test+build per PR), `release.yml` (tag → bundle). macOS arm64 via native runner.

### First-Run Experience

Welcome dialog on first launch: "Use Sample (Docker Hub)" or "Add Your Own". Sample pre-configures an anonymous Docker Hub connection.

---

## 10. V1 Scope Summary

| Feature | Status |
|---|---|
| Browse OCI image registries (Docker Hub, GHCR, Quay, GCR, Generic) | ✅ |
| Search repositories within registry | ✅ |
| List tags per repository | ✅ |
| View manifest JSON + image config + layers | ✅ |
| Pull image as docker-save-compatible tarball | ✅ |
| Browse OCI helm chart registries | ✅ |
| Pull helm chart as .tgz | ✅ |
| Multi-registry connection management | ✅ |
| OAuth authentication (system browser flow) | ✅ |
| OS keychain credential storage | ✅ |
| SQLite-based caching (catalog, tags, manifests) | ✅ |
| Dark/light theme | ✅ |
| i18n (en-US, zh-CN) | ✅ |
| Offline cache browse | ✅ |
| Multi-platform image index (user selects arch) | ✅ |
| Auto-update (manual check) | ✅ |
| Windows (.msi) + macOS (.dmg) installer | ✅ |
| Linux (.AppImage, .deb) | ✅ (best-effort) |
| Generic OCI artifact pull (sig, SBOM, etc.) | ❌ V2 |
| Push to registry | ❌ V2 |
| Tag delete | ❌ V2 |
| Image diff / comparison | ❌ V2 |
| Build / commit / tag | ❌ V2 |
