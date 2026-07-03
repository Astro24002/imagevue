# ImageVue

A cross-platform desktop GUI for browsing OCI image registries and Helm chart repositories. Browse tags, inspect manifests, and pull images as docker-save tarballs or charts as .tgz files.

## Features

- Browse Docker Hub, GHCR, Quay, GCR, and any OCI Distribution v2 registry
- View manifest JSON, image config, and layer details
- Pull images as docker-save tarballs (loadable by `docker load` / `podman load`)
- Pull Helm charts as `.tgz` files
- Multi-registry connection management
- OAuth authentication (system browser flow)
- Offline cache for recently viewed manifests
- Dark/light theme, English/Chinese

## Download

Grab the latest installer for your platform from the [Releases](https://github.com/bkverse/imagevue/releases) page:
- macOS (Intel / Apple Silicon): `.dmg`
- Windows (x64): `.msi`
- Linux (best-effort): `.AppImage` / `.deb`

## Development

Requires Rust 1.78+, Node 20+, and the Tauri CLI.

```bash
npm install
npm run tauri dev
```

### Test

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
npx vitest run
```

## License

MIT
