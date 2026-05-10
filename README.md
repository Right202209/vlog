# Volo Blog

[English](README.md) | [简体中文](README.zh-CN.md)

Volo Blog is a lightweight Markdown blog implemented in Rust with CloudWeGo Volo-HTTP, SQLite, SQLx migrations, and Askama server-side templates.

The codebase covers M1 (read-only public surfaces), M2 (admin auth, content CRUD, uploads, settings, render-on-save Markdown), and the M3 launch-prep slice for feeds (RSS / sitemap / robots), SEO meta (Open Graph + canonical), login rate-limiting, and deployment artifacts (Dockerfile, systemd unit, deployment runbook). Integration tests, a custom 500 page, structured request access logs, and bridging dynamic `site_settings` into the public read path remain open.

## Current Scope

Implemented public surfaces:

- `GET /` and `GET /posts` for the latest published posts.
- `GET /posts/{slug}` for post detail (Open Graph + canonical link).
- `GET /categories/{slug}` for category pages.
- `GET /tags/{slug}` for tag pages.
- `GET /archive` for month-grouped archives.
- `GET /search?q=...` for basic title, summary, and tag-name search.
- `GET /about` for the about page.
- `GET /rss.xml` for the RSS 2.0 feed (latest 20 published posts, RFC 2822 dates).
- `GET /sitemap.xml` for the XML sitemap of all published posts plus core pages.
- `GET /robots.txt` (dynamic — references the configured `SITE_URL`).
- `GET /static/css/site.css` for the site stylesheet.
- `GET /static/uploads/{file}` for uploaded media.
- Fallback 404 page.

Implemented admin surfaces (M2):

- `GET/POST /admin/login` and `POST /admin/logout` with argon2 password hashing and a SQLite-backed session cookie (`vlog_session`).
- `GET /admin` dashboard with post / category / tag counts.
- `GET/POST /admin/posts` and friends for full post CRUD, draft/publish toggle, delete, and Markdown render-on-save.
- `GET/POST /admin/categories` and `GET/POST /admin/tags` for CRUD on taxonomy.
- `GET/POST /admin/settings` for editable site settings (rendered into the `site_settings` table).
- `POST /admin/upload` accepts `multipart/form-data` image uploads (PNG/JPEG/GIF/WebP, max 5 MiB, streamed to a temp file with a chunked size cap) and writes them to `storage/uploads/`, exposed under `/static/uploads/`.

All admin write endpoints (including `/admin/logout`) require the session cookie and a per-session `csrf_token` form field. CSRF is checked in constant time. The `vlog_session` cookie is `HttpOnly; SameSite=Lax`, and gains the `Secure` attribute when `SESSION_COOKIE_SECURE=1`. HTML responses set `X-Content-Type-Options: nosniff` and `Referrer-Policy: same-origin`; admin pages also send a strict default-source CSP.

Login is rate-limited per (lower-cased) username: 5 failed attempts within 60 s triggers a 60 s lockout. Lockouts are tracked in process memory and surface as HTTP 429 with `Retry-After`.

Not implemented yet:

- Public read pages do not yet surface dynamic `site_settings` (still read from `config/default.toml`).
- Custom 500 / 5xx error pages (errors return a plain-text body).
- Structured per-request access log middleware (only `tracing` defaults are wired).
- Integration tests.

## Tech Stack

- HTTP: `volo-http`
- Runtime: Tokio through Volo
- Database: SQLite via `sqlx`
- Migrations: embedded `sqlx::migrate!`
- Templates: Askama (HTML + XML)
- Markdown: `pulldown-cmark`
- Password hashing: `argon2`
- Time formatting: `chrono` (RFC 2822 for RSS, ISO date for sitemap)
- Config: `config/default.toml` plus environment overrides
- Logging: `tracing` and `tracing-subscriber`

## Requirements

- Rust toolchain managed by `rustup`.
- The repo pins the toolchain to `nightly` in `rust-toolchain.toml`.

The original PRD minimum was Rust 1.80. During implementation, the latest Volo dependency graph selected crates that require newer Cargo/Rust support (edition-2024 manifests and Rust 2024 let chains in `volo-http`), so the toolchain channel was bumped to `nightly` while keeping this crate on Rust 2021 edition.

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
site_url = "http://localhost:8080"
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
- `SITE_URL` (absolute base URL used by RSS, sitemap, robots.txt, canonical links, and Open Graph; trailing slash is stripped)
- `HOST`
- `PORT`
- `DATABASE_URL`
- `POSTS_PER_PAGE` (clamped to 1..=100)
- `ADMIN_USERNAME` (default: `admin`, only used for the first-run bootstrap of the admin user)
- `ADMIN_PASSWORD` (default: `admin`, only used for the first-run bootstrap; empty values are rejected)
- `SESSION_COOKIE_SECURE` (set to `1` / `true` / `yes` when serving over HTTPS so the session cookie carries the `Secure` attribute)
- `RUST_LOG` (tracing filter; defaults to `vlog=info,volo_http=info`)

Example:

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
SITE_URL=https://blog.example.com SESSION_COOKIE_SECURE=1 ADMIN_PASSWORD='change-me' cargo run
```

## Project Layout

```text
config/                 Runtime configuration
deploy/                 systemd unit + env example
docs/                   Development, architecture, and deployment notes
migrations/             Embedded SQL migrations
static/css/site.css     Public stylesheet
storage/uploads/        Local upload storage (created on first run)
templates/              Askama HTML and XML templates
src/bin/server.rs       Server entrypoint
src/config/             Settings loader
src/domain/             Plain domain structs
src/repositories/       SQLite query layer
src/services/           Read-model assembly, auth, admin guard, rate limit
src/handlers/           Volo route handlers (public, feed, admin)
src/templates.rs        Askama template structs and HTML/XML response adapters
src/utils/              Markdown, error, datetime, cookie, password, slug, token helpers
Dockerfile              Multi-stage build (nightly builder → debian-slim runtime)
```

Additional docs:

- `docs/DEVELOPMENT.md` for commands, local runtime files, and the M1/M2 verification checklist.
- `docs/ARCHITECTURE.md` for request flow, layers, data model, and milestone boundaries.
- `docs/DEPLOYMENT.md` for env vars, Docker, systemd, reverse proxy, and backups.

## Database

Migrations applied at startup (in order):

- `0001_initial.sql` creates `posts`, `categories`, `tags`, `post_tags` and seeds two example posts.
- `0002_admin.sql` creates `users`, `sessions`, `site_settings`, `assets` and seeds default `site_settings`.

The server also bootstraps a default admin user on first run if none exists (`admin` / `admin`, override with `ADMIN_USERNAME` / `ADMIN_PASSWORD`). **Change this password before exposing the server.**

Local SQLite runtime files are ignored by Git:

- `vlog.db`
- `vlog.db-shm`
- `vlog.db-wal`

## Verification Status

The M1 read-only implementation, the M2 admin & content management layer, and the M3 launch slice (feeds, SEO meta, login rate-limit, deployment artifacts) are in place. End-to-end build and route verification is the next user-side step.

Suggested verification once the toolchain is set up:

```bash
cargo check
cargo run
# Public surfaces
curl -i http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/posts/hello-world
curl -i http://127.0.0.1:8080/categories/tech
curl -i http://127.0.0.1:8080/tags/rust
curl -i http://127.0.0.1:8080/archive
curl -i "http://127.0.0.1:8080/search?q=hello"
curl -i http://127.0.0.1:8080/about
curl -i http://127.0.0.1:8080/static/css/site.css
curl -i http://127.0.0.1:8080/nope
# M3 feeds and SEO
curl -i http://127.0.0.1:8080/rss.xml
curl -i http://127.0.0.1:8080/sitemap.xml
curl -i http://127.0.0.1:8080/robots.txt
# Login rate-limit (expect HTTP 429 + Retry-After after 5 failures)
for i in 1 2 3 4 5 6; do
  curl -is -X POST -d 'username=admin&password=wrong' http://127.0.0.1:8080/admin/login | head -1
done
```

Expected outcomes:

- HTML routes return `Content-Type: text/html; charset=utf-8` plus `X-Content-Type-Options: nosniff`.
- `/rss.xml` and `/sitemap.xml` return `Content-Type: application/xml; charset=utf-8`.
- `/robots.txt` returns `Content-Type: text/plain; charset=utf-8` and references `SITE_URL`.
- `/static/css/site.css` returns the stylesheet.
- Unknown routes return HTTP 404 with the rendered 404 page.
- Restarting the app keeps the seeded SQLite data without duplicate rows.
- After 5 wrong password submissions for the same username, subsequent attempts return HTTP 429 with a `Retry-After` header for 60 s.

## Deployment

See `docs/DEPLOYMENT.md` for the full guide. Quick start:

```bash
# Docker
docker build -t vlog:latest .
docker run --rm -p 8080:8080 \
    -e SITE_URL=https://blog.example.com \
    -e ADMIN_PASSWORD='change-me' \
    -e SESSION_COOKIE_SECURE=1 \
    -v $(pwd)/storage:/app/storage \
    vlog:latest

# systemd
sudo cp deploy/vlog.service /etc/systemd/system/vlog.service
sudo install -m 600 deploy/vlog.env.example /etc/vlog/vlog.env
sudo systemctl enable --now vlog
```

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

M3: launch prep (in progress).

- RSS, sitemap, robots.txt — done.
- Open Graph + canonical SEO meta — done.
- Login rate-limit, security headers, opt-in `Secure` cookie — done.
- Dockerfile, systemd unit, deployment runbook — done.
- Custom 500 page, structured access-log middleware, integration tests — outstanding.
