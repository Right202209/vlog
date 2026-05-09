# Development Guide

This guide tracks local development commands and the M1 verification flow for Volo Blog.

## Toolchain

The repo uses `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.88"
```

Rust 1.88 is currently required by the selected Volo dependency graph because `volo-http` uses Rust 2024 let chains. The application crate itself still uses Rust 2021 edition.

## Common Commands

Format:

```bash
cargo fmt
```

Check:

```bash
cargo check
```

Run:

```bash
cargo run
```

Run with overrides:

```bash
HOST=127.0.0.1 PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
```

## Local Runtime Files

The application creates these local files during normal development:

```text
vlog.db
vlog.db-shm
vlog.db-wal
target/
```

They are intentionally ignored by Git.

## M1 Verification Checklist

After `cargo check` succeeds, start the server:

```bash
cargo run
```

Then verify:

```bash
curl -i http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/posts/hello-world
curl -i http://127.0.0.1:8080/categories/tech
curl -i http://127.0.0.1:8080/tags/rust
curl -i http://127.0.0.1:8080/archive
curl -i "http://127.0.0.1:8080/search?q=hello"
curl -i http://127.0.0.1:8080/about
curl -i http://127.0.0.1:8080/static/css/site.css
curl -i http://127.0.0.1:8080/nope
```

Expected results:

- Public HTML pages return HTTP 200.
- Unknown paths return HTTP 404.
- HTML responses include `Content-Type: text/html; charset=utf-8`.
- The stylesheet route returns CSS.
- Restarting the server does not duplicate seed data.

## Current Implementation Notes

- Public route state is currently held in a process-wide `OnceCell<Arc<AppState>>`. This keeps M1 simple while avoiding assumptions about Volo-HTTP state extraction APIs.
- Seed posts store pre-rendered HTML in `content_html`. Markdown rendering on write belongs to M2.
- RSS and sitemap belong to M3 and are intentionally not wired yet.
