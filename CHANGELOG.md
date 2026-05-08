# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Initialized the Rust project for a single-binary Volo-HTTP Markdown blog.
- Added runtime configuration in `config/default.toml` with environment overrides for host, port, database URL, site metadata, and page size.
- Added the first embedded SQLx migration with tables for posts, categories, tags, and post/tag relationships.
- Added idempotent seed content so a fresh local run has published posts, categories, and tags.
- Added read-only public blog modules for domain models, repositories, services, handlers, template rendering, Markdown rendering, and shared errors.
- Added Askama templates for home, post detail, category, tag, archive, search, about, and 404 pages.
- Added a hand-written stylesheet at `static/css/site.css`.
- Added a server entrypoint at `src/bin/server.rs`.
- Added `storage/uploads/.gitkeep` for the upload directory planned in M2.
- Added `.gitignore` entries for Rust build output and local SQLite runtime files.

### Changed

- Pinned the Rust toolchain to `1.86` because the current `volo`/`volo-http` dependency graph requires a Cargo version that can parse edition-2024 manifests and transitive ICU crates requiring Rust 1.86.

### Verification

- Dependency download and `cargo check` were started, but the previous turn was interrupted before project-level diagnostics and route verification completed.
- Manual route checks for `/`, `/posts/hello-world`, `/categories/tech`, `/tags/rust`, `/archive`, `/search?q=hello`, `/about`, `/static/css/site.css`, and a 404 path are still pending.

## [0.1.0-m1-docs] - 2026-05-08

### Added

- Added project documentation for setup, configuration, architecture, current scope, and milestone status.

