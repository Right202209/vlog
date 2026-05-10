# M4 — Microblog (Weibo / X / GNU social style)

> Canonical plan for milestone M4. The exploratory plan file outside the repo is a snapshot; this file is the source future contributors should read.

## Context

`vlog` today is a server-rendered Markdown blog (PRD M1+M2 done): admin-only auth, long-form posts with title/slug/categories/tags, RSS, sitemap, multi-image upload to local disk, CSRF-protected admin forms, Askama templates over `volo-http` and `sqlx`/SQLite. M4 evolves it into a microblog (Weibo / X / GNU social feel) while **keeping the existing blog working side-by-side**.

Decisions baked in:

- **Multi-user, admin-created only.** No public sign-up. Admin creates user accounts; users log in to post and interact.
- **Coexist with blog.** Existing `posts` / `categories` / `tags` and the admin CRUD for them stay. We add a parallel "statuses" world (microblog).
- **Engagement features in this milestone:** threaded replies, likes, reposts (incl. quote-repost), follow + home timeline.
- **Body is Markdown** (same renderer/pipeline as blog posts). Additionally auto-link `@username` and `#hashtag` after Markdown render.
- **Routing:** the microblog timeline is the new front door at `/`; the existing blog moves to `/blog/*`. RSS, sitemap, robots stay at root and continue to point at blog posts. 301 redirects from old `/posts/{slug}`, `/categories/{slug}`, `/tags/{slug}`, `/archive`, `/search`, `/about` to `/blog/...` so existing links don't break.

M4 does not touch M3 launch concerns (RSS already ships, plus structured logs, deployment polish, integration tests stay deferred).

---

## Data model — `migrations/0003_microblog.sql`

Add to `users`:

- `display_name TEXT` (nullable; backfill = username)
- `bio TEXT`
- `avatar_url TEXT`
- `role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user','admin'))` — backfill existing rows to `'admin'` so the bootstrap admin keeps full power.

New tables:

- `statuses(id, user_id, content_md, content_html, parent_id NULL, repost_of_id NULL, reply_count, like_count, repost_count, created_at)` — `parent_id` is the reply target; `repost_of_id` is the original being reposted; counts are denormalized to keep timeline queries cheap. Indexes on `(created_at DESC)`, `(user_id, created_at DESC)`, `(parent_id)`, `(repost_of_id)`.
- `status_assets(status_id, asset_id, sort)` — many-to-many to existing `assets`. Reuses the upload pipeline.
- `likes(user_id, status_id, created_at)` — composite PK; index on `status_id`.
- `follows(follower_id, followee_id, created_at)` — composite PK with `CHECK (follower_id <> followee_id)`; index on `followee_id`.

Triggers (in the same migration) keep `reply_count` / `like_count` / `repost_count` accurate on insert/delete to avoid `COUNT(*)` per timeline row.

We deliberately do **not** add a hashtags table in this milestone; hashtag pages do a `LIKE '%#tag%'` search against `content_md`. A proper `status_hashtags` index is a follow-up.

---

## Code layout

### Domain (`src/domain/`)

New: `status.rs` (`Status`, `StatusView` = status + author + media + viewer's like/repost flags + (for reposts) embedded original), `follow.rs`, `like.rs`. Extend `user.rs` with the new profile + role columns.

### Repositories (`src/repositories/`)

- `status_repo.rs` — `create`, `find_by_id`, `delete_own`, `list_global_timeline`, `list_user_timeline`, `list_home_timeline` (join through `follows`), `list_replies`.
- `like_repo.rs` — `like`, `unlike`, `has_liked`, `liked_status_ids_for(viewer_id, status_ids)` for batch decoration.
- `follow_repo.rs` — `follow`, `unfollow`, `is_following`, `followers`, `following`, `following_ids` for batch decoration.
- `status_asset_repo.rs` — attach/list assets for a status.
- Extend `user_repo.rs` with `find_by_username`, `list_all`, `create_with_password`, `update_profile`, `update_password`, `set_role`.

### Services (`src/services/`)

- Rename `admin_guard.rs` → `auth_guard.rs`. Keep `verify_csrf`. Replace `require_admin` with two functions:
  - `require_user(pool, headers) -> AuthContext` — any active session.
  - `require_admin(pool, headers) -> AuthContext` — calls `require_user`, then asserts `user.role == "admin"`.
- New `status_service.rs` — orchestrates compose: render Markdown via `utils::markdown::render`, then post-process the rendered HTML to wrap `@username` / `#hashtag` tokens in `<a>` (only outside existing tags / code spans). Persists status + status_assets in one transaction.
- New `mention_service.rs` — lookup helper for `@username`; if username doesn't resolve, leave the literal text.
- Extracted `upload_service.rs` — common multipart-to-asset helper used by `handlers/admin/upload.rs`, `me::upload_avatar`, and the compose-with-images flow.

### Handlers (`src/handlers/`)

```
src/handlers/
  feed.rs                 (existing — RSS/sitemap/robots; unchanged)
  microblog/
    mod.rs
    timeline.rs           GET / and GET /home
    status.rs             GET /s/{id}, POST /compose, POST /s/{id}/(reply|like|unlike|repost|unrepost|delete)
    profile.rs            GET /u/{username}, /u/{username}/(followers|following), POST follow/unfollow
    hashtag.rs            GET /h/{tag}
    me.rs                 GET/POST /me/edit, POST /me/avatar
  admin/
    users.rs              NEW — list/create/reset-password/delete/role-toggle
    (existing modules unchanged in behaviour)
  blog/                   (move existing public read handlers under here)
    mod.rs
    post.rs               (formerly src/handlers/post.rs)
    search.rs             (formerly src/handlers/search.rs)
    about.rs              (formerly src/handlers/about.rs)
  not_found.rs            (unchanged)
```

### Routes (`src/lib.rs`)

```text
# Microblog (front door)
GET   /                           timeline::global
GET   /home                       timeline::home              (require_user)
GET   /s/{id}                     status::detail
POST  /compose                    status::compose             (require_user)
POST  /s/{id}/reply               status::reply               (require_user)
POST  /s/{id}/like                status::like                (require_user)
POST  /s/{id}/unlike              status::unlike              (require_user)
POST  /s/{id}/repost              status::repost              (require_user)
POST  /s/{id}/unrepost            status::unrepost            (require_user)
POST  /s/{id}/delete              status::delete              (require_user; own only)
GET   /u/{username}               profile::show
GET   /u/{username}/followers     profile::followers
GET   /u/{username}/following     profile::following
POST  /u/{username}/follow        profile::follow             (require_user)
POST  /u/{username}/unfollow      profile::unfollow           (require_user)
GET   /h/{tag}                    hashtag::show
GET   /me/edit                    me::edit_form               (require_user)
POST  /me/edit                    me::save                    (require_user)
POST  /me/avatar                  me::upload_avatar           (require_user)

# Auth (unchanged)
GET/POST /admin/login, POST /admin/logout

# Admin (unchanged routes; new admin/users)
GET  /admin                       dashboard
GET  /admin/users                 admin::users::list
POST /admin/users                 admin::users::create
POST /admin/users/{id}/reset      admin::users::reset_password
POST /admin/users/{id}/role       admin::users::set_role
POST /admin/users/{id}/delete     admin::users::delete
... (existing /admin/posts, /admin/categories, /admin/tags, /admin/settings, /admin/upload unchanged)

# Blog (moved under /blog)
GET  /blog                        blog::post::index
GET  /blog/posts/{slug}           blog::post::detail
GET  /blog/categories/{slug}      blog::post::category
GET  /blog/tags/{slug}            blog::post::tag
GET  /blog/archive                blog::post::archive
GET  /blog/search                 blog::search::search
GET  /blog/about                  blog::about::about

# Backwards-compat 301 redirects
GET  /posts                       -> /blog
GET  /posts/{slug}                -> /blog/posts/{slug}
GET  /categories/{slug}           -> /blog/categories/{slug}
GET  /tags/{slug}                 -> /blog/tags/{slug}
GET  /archive                     -> /blog/archive
GET  /search                      -> /blog/search
GET  /about                       -> /blog/about

# Feeds & static (unchanged)
GET  /rss.xml, /sitemap.xml, /robots.txt
nest /static/uploads/, /static/
```

A small `redirect_permanent(path)` helper lives in `utils/response.rs` (mirrors the existing `redirect`).

### Templates

New microblog templates (extend `base.html`):

- `timeline.html` — composer (if logged in) + `_status_card.html` partials + pagination.
- `home.html` — same shell, different data source.
- `status_detail.html` — original (or repost chain) + thread of replies + reply composer (if logged in).
- `profile.html` — header (avatar, display name, bio, follower/following counts, follow button if viewer ≠ owner) + that user's statuses.
- `followers.html`, `following.html` — simple user lists.
- `hashtag.html` — header + matching statuses (paged).
- `me_edit.html` — profile edit form.
- `_status_card.html`, `_composer.html` — partials.
- `admin/users.html` — admin user management.

`base.html` nav: "Feed" → `/`, "Home" → `/home` (only when logged in), "Blog" → `/blog`, "Search" → `/blog/search`. Right side: when logged in, show `@{username}` linking to `/u/{username}` + edit/sign-out; when logged out, "Sign in" → `/admin/login`.

Existing blog templates move into `templates/blog/`; the `#[template(path=...)]` annotations in `src/templates.rs` are updated to match.

Each template carries a `viewer: Option<ViewerContext>` so partials can render the like/repost/follow buttons or hide them.

### CSS (`static/css/site.css`)

Append a `===== Microblog (M4) =====` section: `.composer`, `.status-card` (+ author / body / actions / repost-banner), `.thread`, `.profile-header`, `.avatar`, `.follow-button(.following)`, `.hashtag-pill`. Stays within the existing palette (`--accent`, `--accent-2`, `--soft`, `--surface`, `--line`). No new dependencies.

---

## Files touched

- New: `migrations/0003_microblog.sql`
- New: `src/domain/{status,like,follow}.rs`
- Modified: `src/domain/{user,mod}.rs`
- New repos: `src/repositories/{status_repo,like_repo,follow_repo,status_asset_repo}.rs`
- Modified: `src/repositories/{user_repo,mod}.rs`
- New services: `src/services/{status_service,mention_service,upload_service}.rs`
- Renamed: `src/services/admin_guard.rs` → `src/services/auth_guard.rs`
- Modified: every file in `src/handlers/admin/`
- New: `src/handlers/microblog/{mod,timeline,status,profile,hashtag,me}.rs`, `src/handlers/admin/users.rs`
- Moved: `src/handlers/{post,search,about}.rs` → `src/handlers/blog/{post,search,about}.rs`, plus `src/handlers/blog/mod.rs`
- Modified: `src/lib.rs`, `src/handlers/mod.rs`, `src/templates.rs`, `templates/base.html`
- Moved: `templates/{index,post_detail,category,tag,archive,search,about,404}.html` → `templates/blog/`
- New templates: as listed above
- Modified: `static/css/site.css`, `src/utils/response.rs`
- Reused as-is: `utils::markdown`, `utils::token`, `utils::password`, `utils::cookie`, `utils::slug`

---

## Verification (manual end-to-end)

After `cargo check` is clean and `cargo run` starts:

1. **Migration runs cleanly** — fresh DB and an existing M2 DB; `users` gets the new columns, the bootstrap admin keeps role `admin`.
2. **Admin creates a second user** — `/admin/users`, create `bob`, sign in as `bob`. `bob.role == 'user'`.
3. **Compose** — as `bob`, post from `/`. Markdown renders. `@admin` and `#rust` become links. Two attached images appear in the card.
4. **Reply** — as `admin`, reply. `/s/{id}` shows parent + reply, `reply_count` = 1.
5. **Like / unlike** — as `admin`. Count flips 0 ↔ 1.
6. **Repost + quote** — plain repost from `bob`'s status as `admin` shows "admin reposted" row. Quote-repost renders both comment and embedded original.
7. **Follow + home timeline** — `bob` follows `admin`; `/home` as `bob` shows admin's statuses (and bob's own — design choice, matches X).
8. **Profile edit** — `/me/edit` as `bob`. Display name, bio, avatar persist; cards reflect avatar.
9. **Hashtag** — `/h/rust` lists matching statuses.
10. **Blog still works** — `/blog`, `/blog/posts/hello-world`, `/blog/archive`, `/blog/search?q=hello`. `/posts/hello-world` returns `301`. RSS at `/rss.xml` links into `/blog/posts/...`.
11. **Auth boundaries** — logged-out `POST /compose|/s/{id}/like|/u/{u}/follow` redirects to `/admin/login`. Non-admin hitting `/admin/posts|/admin/users` is forbidden.
12. **CSRF** — every write form carries the per-session token; tampered token → 403.

Automated tests still belong to M3.

---

## Out of scope (explicit)

- Notifications (in-app or email) for mentions / replies / likes / follows.
- Full-text search across statuses; hashtag page uses LIKE.
- A proper hashtag index table.
- Real-time updates (websockets / SSE / polling JS).
- Federation (ActivityPub / Diaspora).
- Public sign-up flow, email verification, password reset by email.
- Image processing (resize / EXIF strip / thumbnails).
- Status edit (X-style edit window) — only delete in this pass.
- Drafts / scheduled statuses.
- Direct messages, lists, bookmarks.
- Rate limiting on compose / like / follow (`rate_limit` only covers login today).
- Admin moderation tools (hide / lock / shadow-ban) beyond plain delete.

These are good follow-ups; each is its own scoped piece of work.
