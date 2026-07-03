# Changelog

## 0.1.0 (2026-07-02)

Initial release.

- Multi-registry connection management (Docker Hub, GHCR, Quay, GCR, Generic)
- Browse catalog, search repositories within a registry
- View tags with metadata (size, OS/arch)
- Inspect manifest JSON + image config + layer history
- Pull images as docker-save tarballs (compatible with `docker load`)
- Pull OCI Helm charts as `.tgz`
- OAuth via system browser (GitHub/Quay/Google)
- Local caching via SQLite, credentials in OS keychain
- Dark/light theme
- English / Simplified Chinese UI
