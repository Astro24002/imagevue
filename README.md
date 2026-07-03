# ImageVue

**Version 0.1.0** · OCI image & chart registry viewer

A cross-platform desktop GUI for browsing OCI image registries and Helm chart repositories. Browse tags, inspect manifests, and pull images as docker-save tarballs or charts as `.tgz` files.

Built with [Tauri 2](https://v2.tauri.app) (Rust backend) + [Vue 3](https://vuejs.org) + [Naive UI](https://www.naiveui.com) (TypeScript frontend).

## Architecture

```
┌─ Web UI (Vue 3 + Naive UI + Pinia + Vue Router) ─────────────────┐
│  ConnectionList / RepoBrowser / TagList / ManifestView            │
│  PullProgress / Settings / LoginDialog                            │
├─ Tauri Commands Layer (Rust ─ thin shell) ────────────────────────┤
│  parameter validation, event emission, streaming progress push    │
├─ Domain Services (Rust) ──────────────────────────────────────────┤
│  RegistryService / RepoService / ManifestService / PullService    │
│  AuthService / CredentialStore                                    │
├─ OCI Client (Rust ─ no Tauri dependency, independently testable) ─┤
│  HttpClient / AuthInterceptor / TokenCache / DistributionClient   │
│  ManifestParser / DescriptorParser                                │
├─ Storage & Output (Rust) ─────────────────────────────────────────┤
│  SqliteStore (connections / cache / history)                      │
│  KeyringStore (OS keychain)                                       │
│  TarballBuilder (docker-save tar) / Rezip (compression normalize) │
└───────────────────────────────────────────────────────────────────┘
```

## Features

| Feature | Status |
|---|---|
| Browse OCI image registries (Docker Hub, GHCR, Quay, GCR, Generic) | ✅ |
| Search repositories within registry | ✅ |
| List tags per repository | ✅ |
| View manifest JSON + image config + layers | ✅ |
| Pull image as docker-save-compatible tarball | ✅ |
| Browse OCI helm chart registries | ✅ |
| Pull helm chart as `.tgz` | ✅ |
| Multi-registry connection management | ✅ |
| OAuth authentication (system browser flow) | ✅ |
| OS keychain credential storage | ✅ |
| SQLite-based caching (catalog, tags, manifests) | ✅ |
| Dark/light theme | ✅ |
| i18n (en-US, zh-CN) | ✅ |
| Offline cache browse | ✅ |
| Windows (.msi) + macOS (.dmg) + Linux (.AppImage, .deb) | ✅ |

## Project Structure

```
imagevue/
├── src/                          # Vue 3 frontend
│   ├── main.ts                   # App entry point
│   ├── App.vue                   # Root component (providers)
│   ├── router/index.ts           # Vue Router (9 routes)
│   ├── stores/                   # Pinia stores
│   │   ├── connections.ts        # Registry connection CRUD
│   │   └── pull.ts               # Pull progress events
│   ├── views/
│   │   ├── ConnectionListView.vue # Registry card grid
│   │   ├── ConnectionEditView.vue # Add/edit form
│   │   ├── RegistryView.vue      # Catalog browser
│   │   ├── RepositoryView.vue    # Tag list
│   │   ├── TagDetailView.vue     # Manifest + config + layers
│   │   ├── SettingsView.vue      # Global preferences
│   │   └── AboutView.vue         # Version info
│   ├── components/
│   │   ├── AppShell.vue          # Layout shell (sidebar + header)
│   │   ├── ConnectionCard.vue    # Connection card widget
│   │   └── PullProgressDrawer.vue # Pull progress drawer
│   ├── i18n/                     # vue-i18n (en-US, zh-CN)
│   └── theme/index.ts            # Naive UI component registration
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml                # 27+ Rust dependencies
│   ├── tauri.conf.json           # Tauri app config
│   └── src/
│       ├── main.rs               # Entry: logging init + app run
│       ├── lib.rs                # Module tree + Tauri builder
│       ├── error.rs              # AppError (tagged serialization)
│       ├── registry_error.rs     # RegistryError
│       ├── auth_error.rs         # AuthError
│       ├── storage_error.rs      # StorageError
│       ├── pull_error.rs         # PullError
│       ├── logging.rs            # File-based tracing (daily rotation)
│       ├── oci/                  # Protocol layer (no Tauri dep)
│       │   ├── mod.rs
│       │   ├── types.rs          # ManifestSummary, Tag, ImageConfig, etc.
│       │   ├── http.rs           # HttpClient (reqwest + retry)
│       │   ├── auth.rs           # AuthInterceptor, TokenCache
│       │   ├── parser.rs         # Manifest + image config parser
│       │   └── client.rs         # DistributionClient
│       ├── storage/              # Persistence
│       │   ├── sqlite.rs         # SqliteHandle (WAL, migrations)
│       │   ├── repo.rs           # ConnectionRepo, CacheRepo
│       │   └── keyring.rs        # KeyringStore
│       ├── services/             # Business orchestration
│       │   ├── registry_service.rs
│       │   ├── auth_service.rs   # OAuth initiation + completion
│       │   ├── manifest_service.rs
│       │   └── pull_service.rs   # Pull orchestration + progress
│       ├── commands/             # Tauri command handlers
│       │   ├── connection.rs     # 5 commands
│       │   ├── catalog.rs        # 1 command
│       │   ├── tag.rs            # 1 command
│       │   ├── manifest.rs       # 2 commands
│       │   ├── pull.rs           # 1 command
│       │   └── auth.rs           # 2 commands
│       └── tarball/              # Docker save output
│           ├── repack.rs         # Rezip pipeline (gzip/zstd → gzip)
│           └── builder.rs        # TarballBuilder (docker-save format)
├── .github/workflows/
│   ├.yml                    # Matrix test + build
│   └── release.yml               # Tag → bundle
├── docs/superpowers/             # Design spec + implementation plan
├── CHANGELOG.md
├── README.md
└── LICENSE (MIT)
```

## Prerequisites

- **Rust** 1.78+ (`rustup install stable`)
- **Node.js** 20+ (`nvm use 20`)
- **Tauri CLI**: `cargo install tauri-cli --version "^2"`

### Platform-specific

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft Visual Studio C++ Build Tools, WebView2 (pre-installed on Win 10+)
- **Linux**: `sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libappindicator3-dev librsvg2-dev`

## Quick Start

```bash
# Install frontend dependencies
npm install

# Run in development mode (hot-reload)
npm run tauri dev

# Build production bundle
npm run tauri build
```

The dev server starts at `http://localhost:5173` and Tauri opens a native window.

## Testing

```bash
# Rust unit tests (26 tests covering all backend modules)
cargo test --manifest-path src-tauri/Cargo.toml --lib

# Rust tests with output
cargo test --manifest-path src-tauri/Cargo.toml --lib -- --nocapture

# Run specific module tests
cargo test --manifest-path src-tauri/Cargo.toml --lib -- oci::

# Frontend type-check (TS)
npx vue-tsc --noEmit

# Full frontend build
npm run build
```

## API (Tauri Commands)

| Command | Input | Returns | Description |
|---|---|---|---|
| `list_connections` | — | `Vec<RegistryConnection>` | List all saved connections |
| `get_connection` | `id: Uuid` | `RegistryConnection` | Get single connection |
| `create_connection` | `input: CreateConnectionInput` | `RegistryConnection` | Add new registry connection |
| `delete_connection` | `id: Uuid` | — | Delete connection + credentials |
| `test_connection` | `id: Uuid` | — | Ping registry, update last_connected_at |
| `list_repositories` | `connectionId, query, limit` | `Vec<Repository>` | Search catalog |
| `list_tags` | `connectionId, repository` | `Vec<Tag>` | List tags for repository |
| `get_manifest` | `connectionId, repository, reference` | `ManifestSummary` | Fetch + parse manifest |
| `get_image_config` | `connectionId, repository, digest` | `ImageConfig` | Fetch + parse image config |
| `start_pull` | `input: StartPullInput` | `Uuid` (job ID) | Start async pull, emits `pull://progress` |
| `begin_oauth` | `input: BeginOAuthInput` | `OAuthSession` | Initiate OAuth browser flow |
| `complete_oauth` | `input: CompleteOAuthInput` | — | Exchange auth code for tokens |

## License

MIT
