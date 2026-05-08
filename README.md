# Volo Blog

Volo Blog is a lightweight Markdown blog implemented in Rust with CloudWeGo Volo-HTTP, SQLite, SQLx migrations, and Askama server-side templates.

The current codebase targets M1 from `Prd.md`: project initialization plus a read-only public blog. Admin login, content CRUD, uploads, RSS, sitemap, SEO expansion, and deployment hardening are planned for later milestones.

## Current Scope

Implemented public surfaces:

- `GET /` and `GET /posts` for the latest published posts.
- `GET /posts/{slug}` for post detail.
- `GET /categories/{slug}` for category pages.
- `GET /tags/{slug}` for tag pages.
- `GET /archive` for month-grouped archives.
- `GET /search?q=...` for basic title, summary, and tag-name search.
- `GET /about` for the about page.
- `GET /static/css/site.css` for the site stylesheet.
- Fallback 404 page.

Not implemented yet:

- Admin authentication and CRUD.
- Upload handling.
- RSS and sitemap.
- Production deployment files.
- Integration tests.

## Tech Stack

- HTTP: `volo-http`
- Runtime: Tokio through Volo
- Database: SQLite via `sqlx`
- Migrations: embedded `sqlx::migrate!`
- Templates: Askama
- Markdown: `pulldown-cmark`
- Config: `config/default.toml` plus environment overrides
- Logging: `tracing` and `tracing-subscriber`

## Requirements

- Rust toolchain managed by `rustup`.
- The repo pins Rust `1.86` in `rust-toolchain.toml`.

The original PRD minimum was Rust 1.80. During implementation, the latest Volo dependency graph selected crates that require newer Cargo/Rust support, so the pin was raised to 1.86 while keeping this crate on Rust 2021 edition.

## Run Locally

From the repository root:

```bash
cargo run
```

The server reads `config/default.toml`, creates `storage/uploads/` if needed, opens `vlog.db`, runs embedded migrations, seeds sample content idempotently, and listens on:

```text
http://127.0.0.1:8080
```

## Configuration

Default configuration lives in `config/default.toml`:

```toml
site_name = "Volo Blog"
site_description = "A lightweight Markdown blog powered by Volo-HTTP."
host = "127.0.0.1"
port = 8080
database_url = "sqlite://vlog.db"
static_dir = "static"
upload_dir = "storage/uploads"
posts_per_page = 10
```

Supported environment overrides:

- `SITE_NAME`
- `SITE_DESCRIPTION`
- `HOST`
- `PORT`
- `DATABASE_URL`
- `POSTS_PER_PAGE`

Example:

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
```

## Project Layout

```text
config/                 Runtime configuration
docs/                   Development and architecture notes
migrations/             Embedded SQL migrations
static/css/site.css     Public stylesheet
storage/uploads/        Future local upload storage
templates/              Askama HTML templates
src/bin/server.rs       Server entrypoint
src/config/             Settings loader
src/domain/             Plain domain structs
src/repositories/       SQLite query layer
src/services/           Read-model assembly
src/handlers/           Volo route handlers
src/templates.rs        Askama template structs and HTML response adapter
src/utils/              Markdown and error helpers
```

Additional docs:

- `docs/DEVELOPMENT.md` for commands, local runtime files, and the M1 verification checklist.
- `docs/ARCHITECTURE.md` for request flow, layers, data model, and milestone boundaries.

## Database

The first migration creates:

- `posts`
- `categories`
- `tags`
- `post_tags`

It also inserts sample categories, tags, and two published posts with `ON CONFLICT`/`INSERT OR IGNORE`, so restarting the app does not duplicate seed data.

Local SQLite runtime files are ignored by Git:

- `vlog.db`
- `vlog.db-shm`
- `vlog.db-wal`

## Verification Status

`cargo check` was started after dependency setup, but the previous work turn was interrupted before it completed. Route-level verification is still pending.

Expected M1 verification once compilation is clean:

```bash
cargo check
cargo run
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

Expected outcomes:

- HTML routes return `Content-Type: text/html; charset=utf-8`.
- `/static/css/site.css` returns the stylesheet.
- Unknown routes return HTTP 404 with the rendered 404 page.
- Restarting the app keeps the seeded SQLite data without duplicate rows.

## Milestones

M1: read-only blog.

- Project scaffold.
- SQLite schema and sample data.
- Public route handlers.
- Askama templates.
- Static CSS.

M2: admin and content management.

- Login/session authentication.
- CSRF protection.
- Post/category/tag CRUD.
- Markdown render-on-save.
- Upload endpoint and local asset records.
- Site settings form.

M3: launch prep.

- RSS and sitemap.
- SEO meta fields.
- Structured request logging.
- Production error pages.
- Docker/systemd/release packaging.
- Integration tests and runbook hardening.
