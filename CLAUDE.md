# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Volo Blog: a single-binary Markdown blog built on CloudWeGo `volo-http`, SQLite via `sqlx`, and server-rendered Askama templates. The current code covers **M1 (read-only public surfaces)** and **M2 (admin auth + content CRUD + uploads + settings + Markdown render-on-save)** from `Prd.md`. RSS, sitemap, structured logs, deployment files, and integration tests are deferred to M3 and should not be added incidentally.

## Toolchain

- Rust toolchain pinned to `nightly` in `rust-toolchain.toml`. The bump from the PRD's 1.80 minimum is required because the selected `volo`/`volo-http` 0.5 dependency graph uses edition-2024 manifests and Rust 2024 let chains. Do not lower it. The crate itself stays on Rust 2021 edition.

## Common commands

```bash
cargo fmt
cargo check
cargo run                                   # serves on 127.0.0.1:8080
HOST=127.0.0.1 PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
ADMIN_USERNAME=alice ADMIN_PASSWORD=hunter2 cargo run    # bootstrap a different default admin
```

Env overrides recognized by the config loader: `SITE_NAME`, `SITE_DESCRIPTION`, `HOST`, `PORT`, `DATABASE_URL`, `POSTS_PER_PAGE`. Defaults live in `config/default.toml`. `ADMIN_USERNAME` / `ADMIN_PASSWORD` are read by the auth service only when no users exist yet (first-run bootstrap).

There is no test suite yet — integration tests are an M3 deliverable. M1/M2 verification is the curl checklist in `docs/DEVELOPMENT.md`.

## Architecture

Request flow is strictly layered:

```
volo-http Router  →  handler  →  service / repository  →  SQLx (SQLite)  →  Askama template  →  HtmlTemplate IntoResponse
```

- `src/bin/server.rs` — entrypoint: loads `Settings`, opens the SQLite pool, runs `sqlx::migrate!`, calls `auth_service::ensure_default_admin`, purges expired sessions, calls `build_router`, binds the listener.
- `src/lib.rs` — owns `AppState { settings, pool }` and `build_router`. **State is held in a process-wide `OnceCell<Arc<AppState>>`** (`APP_STATE`), accessed by handlers via `app_state()`. This sidesteps assumptions about Volo-HTTP state extraction; do not introduce a parallel state mechanism without removing the OnceCell.
- `src/handlers/` — thin route handlers. Public read handlers live at the module root; admin handlers live in `src/handlers/admin/` (one module per resource: `auth`, `dashboard`, `posts`, `categories`, `tags`, `settings`, `upload`).
- `src/repositories/` — SQLx query functions. M2 added `user_repo`, `session_repo`, `settings_repo`, `asset_repo`, plus admin write helpers (`create`, `update`, `delete`, `set_status`, `set_tags`, `slug_exists`, `find_by_id`, `list_all`) on the existing post/category/tag repos.
- `src/services/` — read-model assembly (`post_service`), auth (`auth_service`), admin guard (`admin_guard`), Markdown render-on-save (`admin_post_service`).
- `src/domain/` — plain structs shared by repos, services, templates. M2 added `user`, `session`, `site_settings`, `asset`.
- `src/templates.rs` — Askama struct definitions plus the `HtmlTemplate<T>` adapter that sets `Content-Type: text/html; charset=utf-8`. Admin templates live under `templates/admin/` and inherit from `templates/admin/_layout.html` which itself extends `templates/base.html`.
- `src/utils/` — Markdown rendering, shared `AppError`, password hashing (argon2), cookie parsing, hex random tokens, slug helper, response helpers (redirects), and a `RequestHeaders` extractor.
- `migrations/` — embedded SQL migrations. `0001_initial.sql` seeds posts/categories/tags. `0002_admin.sql` creates `users`, `sessions`, `site_settings`, `assets` and seeds default `site_settings` rows.

Data model: posts have one category and many tags via `post_tags`. Public routes filter on `status = 'published'`. Admin routes see all statuses.

## Auth & CSRF (M2)

- Login posts username/password to `/admin/login`. The session id is stored in the `vlog_session` cookie (`HttpOnly`, `SameSite=Lax`, 7-day max-age).
- Each session has a per-session `csrf_token` (hex). Every admin write form must echo `csrf_token` as a hidden field; `admin_guard::verify_csrf` rejects requests where it doesn't match.
- The default-admin bootstrap (`auth_service::ensure_default_admin`) only runs when `users` is empty. It seeds `admin` / `admin` unless `ADMIN_USERNAME` / `ADMIN_PASSWORD` are set. Treat the seeded password as a placeholder — change it before exposing the server.

## Conventions to preserve

- Public read pages currently still source `site_name`, `site_description`, and `posts_per_page` from `config/default.toml` rather than the `site_settings` table. Bridging dynamic settings into the public read path is an open M2 follow-up; the admin form already writes to the table.
- Seed posts in migration 0001 store **pre-rendered HTML in `content_html`**. Admin create/update flows now also populate `content_html` via `admin_post_service::render_html`. The read path still reads `content_html` directly — do not start re-rendering Markdown on the read path.
- Local SQLite runtime files (`vlog.db`, `vlog.db-shm`, `vlog.db-wal`) are git-ignored and recreated on run.
- Uploads live in `storage/uploads/` and are exposed under `/static/uploads/`. Files are stored with random hex names + sanitized extension. Mime allow-list currently restricts to `image/*`; size is capped at 5 MiB.
- Milestone discipline: keep M3 features (RSS, sitemap, SEO meta expansion, structured logs, Docker/systemd/release packaging, integration tests) out of M1/M2 changes unless the user explicitly scopes them in.

## Reference docs

- `Prd.md` — full product requirements and milestone breakdown.
- `docs/ARCHITECTURE.md` — layer responsibilities and milestone boundaries.
- `docs/DEVELOPMENT.md` — commands, runtime files, M1 verification checklist.
- `CHANGELOG.md` — current state. Note: as of the last entry, `cargo check` had not been confirmed clean end-to-end; verify before claiming the build works.
