# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added (M2 admin and content management)

- New SQL migration `migrations/0002_admin.sql` adding `users`, `sessions`, `site_settings`, and `assets` tables, and seeding default `site_settings` rows.
- Domain models for `User`, `Session`, `SiteSettings`, and `Asset`.
- Admin write methods on the existing repositories and four new repositories: `user_repo`, `session_repo`, `settings_repo`, `asset_repo`.
- Auth service with argon2 password hashing, session creation/revocation, and a default-admin bootstrap that runs at startup (`admin` / `admin` unless overridden by `ADMIN_USERNAME` / `ADMIN_PASSWORD`).
- Admin guard service that resolves the signed-in user from the `vlog_session` cookie and validates a per-session CSRF token on writes.
- Admin handlers under `src/handlers/admin/` for: login (GET/POST), logout, dashboard, post CRUD (list/new/create/edit/update/publish/unpublish/delete), category CRUD, tag CRUD, settings (GET/POST), and multipart image upload.
- Markdown render-on-save (`admin_post_service`) so newly created or edited posts store both `content_md` and `content_html`.
- Slug helper, hex-encoded random token helper, cookie parsing helper, custom `RequestHeaders` extractor, password hashing module, and shared response helpers (redirects, header injection).
- Askama admin templates: `_layout`, `login`, `dashboard`, `posts`, `post_edit`, `categories`, `tags`, `settings`. CSS additions in `static/css/site.css` style the admin shell, forms, and tables.
- Local upload directory (`storage/uploads/`) is now exposed under `/static/uploads/` via a dedicated `ServeDir` mount; the upload handler writes random-named files there and records an `assets` row.

### Added (originally M1)

- Initialized the Rust project for a single-binary Volo-HTTP Markdown blog.
- Added runtime configuration in `config/default.toml` with environment overrides for host, port, database URL, site metadata, and page size.
- Added the first embedded SQLx migration with tables for posts, categories, tags, and post/tag relationships.
- Added idempotent seed content so a fresh local run has published posts, categories, and tags.
- Added read-only public blog modules for domain models, repositories, services, handlers, template rendering, Markdown rendering, and shared errors.
- Added Askama templates for home, post detail, category, tag, archive, search, about, and 404 pages.
- Added a hand-written stylesheet at `static/css/site.css`.
- Added a server entrypoint at `src/bin/server.rs`.
- Added `storage/uploads/.gitkeep` for the upload directory now wired in M2.
- Added `.gitignore` entries for Rust build output and local SQLite runtime files.

### Changed

- Pinned the Rust toolchain channel to `nightly` so the `volo`/`volo-http` 0.5 dependency graph (edition-2024 manifests, Rust 2024 let chains) resolves and compiles. The application crate stays on Rust 2021 edition.
- Refactored response types in shared error handling and the Askama `HtmlTemplate` adapter so handlers can return `Result<HtmlTemplate<T>, AppError>` directly and the adapter sets `Content-Type: text/html; charset=utf-8` on rendered responses.
- Extended `AppError` with `Unauthorized` (redirects to `/admin/login` and clears the session cookie), `Forbidden`, `BadRequest`, `Conflict`, `Password`, and `Io` variants so admin handlers can short-circuit cleanly.
- Server entrypoint now bootstraps the default admin user and purges expired sessions during startup.

### Status

- M1 read-only blog: complete.
- M2 admin and content management: source-complete in this turn — auth, CRUD for posts/categories/tags, settings form, image upload, Markdown render-on-save, CSRF-protected writes, and per-session cookie auth are all wired together. Public read pages still source `site_name`, `site_description`, and `posts_per_page` from the TOML config rather than the `site_settings` table; bridging that into the public read path is a follow-up.
- M3 launch concerns (RSS, sitemap, SEO meta expansion, structured logs, Docker/systemd packaging, integration tests) remain out of scope.
- End-to-end build (`cargo check` / `cargo run`) and route-level verification are intentionally left for the operator; nothing has been executed in this turn.

## [0.1.0-m1-docs] - 2026-05-08

### Added

- Added project documentation for setup, configuration, architecture, current scope, and milestone status.
