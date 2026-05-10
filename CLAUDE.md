# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Volo Blog: a single-binary site built on CloudWeGo `volo-http`, SQLite via `sqlx`, and server-rendered Askama templates. The code covers **M1 (read-only public surfaces)**, **M2 (admin auth + content CRUD + uploads + settings + Markdown render-on-save)**, and is in progress on **M4 (Weibo / X style microblog timeline + multi-user accounts + replies/likes/reposts/follows)**. The launch-prep M3 work (RSS, sitemap, structured logs, Docker/systemd packaging, integration tests) shipped partially (RSS / sitemap / robots.txt are wired) but the rest is still deferred and should not be added incidentally.

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

Env overrides recognized by the config loader: `SITE_NAME`, `SITE_DESCRIPTION`, `HOST`, `PORT`, `DATABASE_URL`, `POSTS_PER_PAGE`. Defaults live in `config/default.toml`. `ADMIN_USERNAME` / `ADMIN_PASSWORD` are read by the auth service only when no users exist yet (first-run bootstrap). Additional accounts after bootstrap are created from `/admin/users`, not from env.

There is no test suite yet — integration tests are an M3 deliverable. M1/M2/M4 verification is the curl checklist in `docs/DEVELOPMENT.md`.

## Architecture

Request flow is strictly layered:

```
volo-http Router  →  handler  →  service / repository  →  SQLx (SQLite)  →  Askama template  →  HtmlTemplate IntoResponse
```

- `src/bin/server.rs` — entrypoint: loads `Settings`, opens the SQLite pool, runs `sqlx::migrate!`, calls `auth_service::ensure_default_admin`, purges expired sessions, calls `build_router`, binds the listener.
- `src/lib.rs` — owns `AppState { settings, pool }` and `build_router`. **State is held in a process-wide `OnceCell<Arc<AppState>>`** (`APP_STATE`), accessed by handlers via `app_state()`. This sidesteps assumptions about Volo-HTTP state extraction; do not introduce a parallel state mechanism without removing the OnceCell.
- `src/handlers/` — thin route handlers, organized by surface:
  - `microblog/` — timeline, status detail, profile, hashtag, me (`/`, `/home`, `/s/{id}`, `/u/{username}`, `/h/{tag}`, `/me/...`).
  - `blog/` — the original public read pages, now mounted under `/blog/*` (`post`, `search`, `about`).
  - `admin/` — admin console (`auth`, `dashboard`, `posts`, `categories`, `tags`, `settings`, `upload`, `users`).
  - `feed.rs` — RSS, sitemap, robots.
  - `not_found.rs` — fallback.
- `src/repositories/` — SQLx query functions. Microblog adds `status_repo`, `like_repo`, `follow_repo`, `status_asset_repo`, plus extensions on `user_repo` (`find_by_username`, `list_all`, `create_with_password`, `update_profile`, `update_password`, `set_role`).
- `src/services/` — read-model assembly (`post_service`), auth (`auth_service`), session/CSRF guard (`auth_guard`, formerly `admin_guard`), Markdown render-on-save for blog posts (`admin_post_service`), microblog compose orchestration (`status_service`), `@username` lookup (`mention_service`), shared multipart upload (`upload_service`), login rate limiting (`rate_limit`).
- `src/domain/` — plain structs shared by repos, services, templates. M2 added `user`, `session`, `site_settings`, `asset`. M4 added `status`, `like`, `follow` and extended `user` with `display_name` / `bio` / `avatar_url` / `role`.
- `src/templates.rs` — Askama struct definitions plus the `HtmlTemplate<T>` adapter that sets `Content-Type: text/html; charset=utf-8`. Admin templates live under `templates/admin/` and inherit from `templates/admin/_layout.html` which itself extends `templates/base.html`. Blog templates live under `templates/blog/`. Microblog templates live at `templates/` root (`timeline.html`, `home.html`, `status_detail.html`, `profile.html`, `hashtag.html`, `me_edit.html`, `_status_card.html`, `_composer.html`).
- `src/utils/` — Markdown rendering, shared `AppError`, password hashing (argon2), cookie parsing, hex random tokens, slug helper, response helpers (redirects + 301 redirects), and a `RequestHeaders` extractor.
- `migrations/` — embedded SQL migrations. `0001_initial.sql` seeds posts/categories/tags. `0002_admin.sql` creates `users`, `sessions`, `site_settings`, `assets` and seeds default `site_settings`. `0003_microblog.sql` adds the role/profile columns to `users`, the `statuses` / `status_assets` / `likes` / `follows` tables, and the count-maintenance triggers.

Data model:

- **Blog**: posts have one category and many tags via `post_tags`. Public routes (under `/blog`) filter on `status = 'published'`. Admin routes see all statuses.
- **Microblog**: `statuses.user_id` links to `users`; `parent_id` chains replies; `repost_of_id` links a repost (or quote-repost when `content_md` is non-empty) to its original. `reply_count` / `like_count` / `repost_count` are denormalized columns kept current by SQL triggers — do not `COUNT(*)` per timeline row.

## Auth & CSRF (M2)

- Login posts username/password to `/admin/login`. The session id is stored in the `vlog_session` cookie (`HttpOnly`, `SameSite=Lax`, 7-day max-age).
- Each session has a per-session `csrf_token` (hex). Every write form must echo `csrf_token` as a hidden field; `auth_guard::verify_csrf` rejects requests where it doesn't match.
- The default-admin bootstrap (`auth_service::ensure_default_admin`) only runs when `users` is empty. It seeds `admin` / `admin` unless `ADMIN_USERNAME` / `ADMIN_PASSWORD` are set. Treat the seeded password as a placeholder — change it before exposing the server.

## Microblog & multi-user (M4)

- `users.role` is `'user'` or `'admin'`. Bootstrap admin is `'admin'`; everyone created from `/admin/users` is `'user'` unless toggled.
- Use `auth_guard::require_user` for any logged-in write (compose, reply, like, repost, follow, profile edit). Use `auth_guard::require_admin` for `/admin/*`. `require_admin` calls `require_user` then asserts the role.
- Microblog statuses share the existing CSRF model — every form posts `csrf_token`.
- The site front door is the global timeline at `/`. The blog moved to `/blog/*`. Old paths (`/posts/{slug}`, `/categories/{slug}`, `/tags/{slug}`, `/archive`, `/search`, `/about`) return 301 to `/blog/...` via `utils::response::redirect_permanent`.
- Compose flow: `status_service::create_status` renders Markdown via `utils::markdown::render`, then post-processes the HTML to wrap `@username` and `#hashtag` in `<a>` (text-node walker that skips inside existing tags / code spans). `mention_service` resolves `@username` to a user id; unresolved mentions stay as plain text but still get a `/u/{username}` link.
- Counts (`reply_count`, `like_count`, `repost_count`) are maintained by triggers in `0003_microblog.sql`. Reads should use the column; writes should never touch the column directly — let the trigger fire.
- `/h/{tag}` uses `LIKE '%#tag%'` against `statuses.content_md`. There is intentionally no `status_hashtags` index in M4; revisit if status volume makes this slow.
- Timeline visibility: `/` shows top-level statuses (no `parent_id`) ordered by `created_at DESC`. `/home` shows top-level statuses where `user_id` is in `following_ids(viewer)` ∪ `{viewer.id}` (X-style; viewer's own posts appear).

## Conventions to preserve

- The site front door is the microblog timeline. The blog lives at `/blog/*`. Do not regress this routing without updating the redirects, RSS links, and template path annotations together.
- Public blog read pages still source `site_name`, `site_description`, and `posts_per_page` from `config/default.toml` rather than the `site_settings` table. Bridging dynamic settings into the public read path is an open M2 follow-up; the admin form already writes to the table. The same applies to microblog templates (they currently read settings from config, not the DB).
- Seed posts in migration 0001 store **pre-rendered HTML in `content_html`**. Admin create/update flows for blog posts populate `content_html` via `admin_post_service::render_html`; microblog statuses populate `content_html` via `status_service::create_status`. The read path always reads `content_html` directly — do not start re-rendering Markdown on the read path.
- Local SQLite runtime files (`vlog.db`, `vlog.db-shm`, `vlog.db-wal`) are git-ignored and recreated on run.
- Uploads live in `storage/uploads/` and are exposed under `/static/uploads/`. Files are stored with random hex names + sanitized extension. Mime allow-list currently restricts to `image/*`; size is capped at 5 MiB. Both blog cover images, microblog status attachments, and user avatars share this pipeline via `upload_service`.
- Milestone discipline: keep deferred M3 work (structured logs, Docker/systemd/release packaging, integration tests, SEO meta expansion) out of M1/M2/M4 changes unless the user explicitly scopes them in.

## Reference docs

- `Prd.md` — full product requirements and milestone breakdown (M4 is §17).
- `docs/M4_MICROBLOG.md` — the canonical microblog plan: schema, routes, templates, files-touched, verification.
- `docs/ARCHITECTURE.md` — layer responsibilities and milestone boundaries.
- `docs/DEVELOPMENT.md` — commands, runtime files, M1/M2/M4 verification checklist.
- `CHANGELOG.md` — current state. Note: as of the last entry, `cargo check` had not been confirmed clean end-to-end after M4; verify before claiming the build works.
