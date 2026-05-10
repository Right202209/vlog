# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added (M4 microblog)

- New SQL migration `migrations/0003_microblog.sql`: extends `users` with `display_name`, `bio`, `avatar_url`, and a `role` column (`user` / `admin`, default `user`; existing rows backfilled to `admin`); creates `statuses`, `status_assets`, `likes`, `follows` tables with appropriate indexes and `CHECK` constraints; installs SQL triggers that maintain `statuses.reply_count` / `like_count` / `repost_count` on insert/delete of children.
- Domain models for `Status`, `StatusView`, `Like`, `Follow`. Extended `User` with the four new profile/role fields.
- New repositories: `status_repo` (create, find_by_id, list_global_timeline, list_user_timeline, list_home_timeline, list_replies, delete_own), `like_repo`, `follow_repo`, `status_asset_repo`. Extended `user_repo` with `find_by_username`, `list_all`, `create_with_password`, `update_profile`, `update_password`, `set_role`.
- Renamed `services/admin_guard` → `services/auth_guard`. Split into `require_user(pool, headers)` (any active session) and `require_admin(pool, headers)` (calls `require_user`, then asserts `role == "admin"`). Every admin handler call site updated.
- New services: `status_service` (compose orchestration: render Markdown via `utils::markdown::render`, then auto-link `@username` and `#hashtag` outside existing tags / code spans, persist status + status_assets in one transaction), `mention_service` (`@username` → user id lookup), `upload_service` (multipart-to-asset helper extracted from the admin upload handler so avatar uploads and status image attachments can share it).
- New handler tree under `src/handlers/microblog/`: `timeline` (`/`, `/home`), `status` (`/s/{id}`, `/compose`, `/s/{id}/(reply|like|unlike|repost|unrepost|delete)`), `profile` (`/u/{username}`, `/followers`, `/following`, follow/unfollow), `hashtag` (`/h/{tag}`), `me` (`/me/edit`, `/me/avatar`).
- Moved public blog read handlers under `src/handlers/blog/` (`post`, `search`, `about`). Old top-level paths (`/posts`, `/posts/{slug}`, `/categories/{slug}`, `/tags/{slug}`, `/archive`, `/search`, `/about`) now return 301 to `/blog/...` via `utils::response::redirect_permanent`. RSS, sitemap, and robots.txt continue to live at root and link to `/blog/posts/{slug}`.
- New admin handler `admin/users` with list, create, reset-password, set-role, and delete operations (CSRF-protected).
- New microblog templates under `templates/`: `timeline.html`, `home.html`, `status_detail.html`, `profile.html`, `followers.html`, `following.html`, `hashtag.html`, `me_edit.html`, plus `_status_card.html` and `_composer.html` partials. New admin template `templates/admin/users.html`. Existing public blog templates moved into `templates/blog/` and the `#[template(path=...)]` annotations in `src/templates.rs` updated.
- Header navigation in `templates/base.html` updated: "Feed" → `/`, "Home" → `/home` (only when logged in), "Blog" → `/blog`, "Search" → `/blog/search`. Right side shows the signed-in user as `@{username}` linking to their profile, with edit + sign-out controls; logged-out visitors see a "Sign in" link.
- New CSS section `===== Microblog (M4) =====` appended to `static/css/site.css` covering `.composer`, `.status-card` (+ author / body / actions / repost-banner), `.thread`, `.profile-header`, `.avatar`, `.follow-button(.following)`, `.hashtag-pill`. Stays within the existing palette; no new dependencies or fonts.
- `utils/response.rs` gains a `redirect_permanent(path)` helper (`301 Moved Permanently`) used by the blog backwards-compat redirects.

### Status

- M4 microblog: source-complete in this turn. Multi-user (admin-created only), threaded replies, likes, reposts (incl. quote), follow + home timeline, profile pages, hashtag pages, admin user management, blog coexistence at `/blog/*` with 301 redirects from old paths. Public sign-up, notifications, full-text search across statuses, real-time updates, federation, image processing, status edit, drafts, DMs, and moderation tools beyond delete remain out of scope.
- The `users.role` column defaults to `'user'`; existing users from M2 are migrated to `'admin'` so the bootstrap account keeps full access.
- Verification of the new flows (compose, reply, like, repost, follow, profile, hashtag, blog redirects, CSRF) is the curl/browser checklist in `docs/DEVELOPMENT.md`. Automated tests are still M3.
- End-to-end build (`cargo check` / `cargo run`) and route-level verification are intentionally left for the operator; nothing has been executed in this turn.

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
