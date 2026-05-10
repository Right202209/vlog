# Volo Blog

[English](README.md) | [简体中文](README.zh-CN.md)

A single-binary Rust site that pairs a Weibo / X style microblog timeline with a long-form Markdown blog. Built on CloudWeGo Volo-HTTP, SQLite via SQLx, Askama server-side templates, and Argon2 sessions. Runs from one binary with embedded migrations.

The microblog (replies, likes, reposts, quote-reposts, follows, profiles, hashtags) lives at `/`; the blog (CRUD, RSS, sitemap, search, archive) lives at `/blog/*`. Admin-only multi-user accounts — no public sign-up.

## Run

```bash
cargo run
# http://127.0.0.1:8080
```

The server reads `config/default.toml`, creates `storage/uploads/`, opens `vlog.db`, runs embedded migrations, and bootstraps a default admin (`admin` / `admin`) if the `users` table is empty. **Change this password before exposing the server.**

## Configuration

Defaults in `config/default.toml`. Environment overrides: `SITE_NAME`, `SITE_DESCRIPTION`, `SITE_URL`, `HOST`, `PORT`, `DATABASE_URL`, `POSTS_PER_PAGE`, `ADMIN_USERNAME`, `ADMIN_PASSWORD` (first-run bootstrap only), `SESSION_COOKIE_SECURE` (set when serving over HTTPS), `RUST_LOG`.

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
SITE_URL=https://blog.example.com SESSION_COOKIE_SECURE=1 ADMIN_PASSWORD='change-me' cargo run
```

## Toolchain

`rust-toolchain.toml` pins the channel to `nightly` because the selected `volo` / `volo-http` 0.5 dependency graph requires edition-2024 manifests and Rust 2024 let chains. The crate itself stays on Rust 2021 edition.

## Docs

- `docs/DEVELOPMENT.md` — commands, runtime files, verification checklist.
- `docs/ARCHITECTURE.md` — request flow, layers, data model, milestone boundaries.
- `docs/DEPLOYMENT.md` — Docker, systemd, reverse proxy, env vars, backups.
- `docs/M4_MICROBLOG.md` — microblog spec.
