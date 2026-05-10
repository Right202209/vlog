# Architecture Notes

Volo Blog is organized around a small server-rendered Rust application. It hosts two coexisting surfaces: a Weibo / X style **microblog timeline** at the root, and the original long-form **blog** under `/blog/*`.

## Request Flow

```text
Volo Router
  -> handler
  -> repository/service
  -> SQLite via SQLx
  -> Askama template
  -> HtmlTemplate IntoResponse adapter
```

Handlers are intentionally thin. They read query/path parameters, load application state, call repository/service functions, and return typed Askama templates wrapped in `HtmlTemplate`.

## Layers

`src/handlers/`

Route handlers, organized by surface:

- `microblog/` — timeline (`/`, `/home`), status detail and writes (`/s/{id}`, `/compose`, `/s/{id}/(reply|like|...)`), profile (`/u/{username}`, follow/unfollow, follower/following lists), hashtag (`/h/{tag}`), and self-service (`/me/edit`, `/me/avatar`).
- `blog/` — the original public read pages, now mounted under `/blog/*` (`post`, `search`, `about`).
- `admin/` — admin console (`auth`, `dashboard`, `posts`, `categories`, `tags`, `settings`, `upload`, `users`).
- `feed.rs` — RSS, sitemap, robots.
- `not_found.rs` — 404 fallback.

`src/repositories/`

SQLx query functions. The repository layer owns SQL shape and pagination queries. M4 added `status_repo`, `like_repo`, `follow_repo`, `status_asset_repo`, plus extensions to `user_repo` (`find_by_username`, `list_all`, `create_with_password`, `update_profile`, `update_password`, `set_role`).

`src/services/`

- `post_service` — read-model assembly for blog posts (attach category + tags).
- `auth_service` — login, logout, default-admin bootstrap.
- `auth_guard` (renamed from `admin_guard`) — `require_user` for any logged-in write, `require_admin` for `/admin/*`. Both produce an `AuthContext { user, session }`. `verify_csrf` validates the per-session token.
- `admin_post_service` — Markdown render + summary excerpt for blog posts.
- `status_service` — Markdown render + `@username` / `#hashtag` auto-link for microblog statuses, plus the compose transaction (status row + status_assets attachments).
- `mention_service` — `@username` resolution (used by `status_service` and the auto-linker).
- `upload_service` — common multipart-to-asset helper used by `admin/upload`, `me/avatar`, and the compose-with-images flow.
- `rate_limit` — login-failure throttle.

`src/domain/`

Plain structs used by repositories, services, and templates. M2 added `user`, `session`, `site_settings`, `asset`. M4 added `status`, `like`, `follow` and extended `user` with `display_name`, `bio`, `avatar_url`, `role`.

`src/templates.rs`

Askama template structs plus the `HtmlTemplate<T>` response adapter that sets the HTML content type. Each microblog template carries a `viewer: Option<ViewerContext>` so partials know whether to show like/repost/follow buttons.

`templates/`

Server-rendered HTML files using Askama inheritance:

- `base.html` — shared header (Feed / Home / Blog / Search nav + signed-in user widget) and footer.
- Microblog templates at the root: `timeline.html`, `home.html`, `status_detail.html`, `profile.html`, `followers.html`, `following.html`, `hashtag.html`, `me_edit.html`, plus `_status_card.html` and `_composer.html` partials (Askama `{% include %}`).
- `blog/` — moved from the root: `index.html`, `post_detail.html`, `category.html`, `tag.html`, `archive.html`, `search.html`, `about.html`, `404.html`.
- `admin/` — admin console (`_layout.html` extends `base.html`; per-resource pages: `login`, `dashboard`, `posts`, `post_edit`, `categories`, `tags`, `settings`, `users`).

`migrations/`

Embedded SQL migrations run at startup through `sqlx::migrate!`. `0001_initial.sql` seeds posts/categories/tags. `0002_admin.sql` adds users/sessions/site_settings/assets. `0003_microblog.sql` adds the role/profile columns to `users`, the `statuses` / `status_assets` / `likes` / `follows` tables, and the SQL triggers that maintain `statuses.reply_count` / `like_count` / `repost_count`.

## Data Model

**Blog tables** (M1):

- `posts` (with `status IN ('draft','published','archived')`)
- `categories`
- `tags`
- `post_tags`

Posts belong to one category and many tags. Only `status = 'published'` is exposed by `/blog/*`.

**Microblog tables** (M4):

- `statuses(id, user_id, content_md, content_html, parent_id NULL, repost_of_id NULL, reply_count, like_count, repost_count, created_at)` — `parent_id` non-NULL = reply; `repost_of_id` non-NULL = repost (or quote-repost when `content_md` is non-empty).
- `status_assets(status_id, asset_id, sort)` — many-to-many to existing `assets`.
- `likes(user_id, status_id, created_at)` — composite PK.
- `follows(follower_id, followee_id, created_at)` — composite PK with `CHECK (follower_id <> followee_id)`.

The three `*_count` columns on `statuses` are kept current by triggers in `0003_microblog.sql`. Reads use the column directly; nothing in Rust should `COUNT(*)` per timeline row, and writes should never touch the column directly — let the trigger fire when child rows insert/delete.

Hashtag pages (`/h/{tag}`) use `LIKE '%#tag%'` against `statuses.content_md`. There is no `status_hashtags` index in M4; revisit if status volume makes this slow.

## Milestone Boundaries

- **M1** is read-only and intentionally keeps authentication, writes, uploads, RSS, sitemap, deployment files, and integration tests out of scope.
- **M2** introduced the admin console: auth, sessions, CSRF, post/category/tag forms, upload handling, settings, render-on-save.
- **M3** is partially shipped (RSS, sitemap, robots.txt are wired) and partially deferred (structured logs, Docker/systemd polish, integration tests, expanded SEO meta) — keep the deferred slice out of M4 changes.
- **M4** introduces the microblog: multi-user (admin-created only), statuses with replies / likes / reposts / quote-reposts / follows, profiles, hashtag pages, and the routing flip that puts the timeline at `/` and the blog at `/blog/*`.

## M4 Layer Additions

- **Compose path**: `POST /compose` → `handlers::microblog::status::compose` → `auth_guard::require_user` → `status_service::create_status` (renders Markdown via `utils::markdown::render`, post-processes the rendered HTML to wrap `@username` and `#hashtag` outside existing tags / code spans, persists the status + status_assets in one transaction) → `status_repo::create` → triggers update parent counts where applicable. Reply/repost/quote-repost flow through the same service with `parent_id` / `repost_of_id` / non-empty `content_md` set accordingly.
- **Engagement path**: like/unlike, follow/unfollow, repost/unrepost POST handlers call `like_repo::like` / `follow_repo::follow` / `status_repo::create` (with `repost_of_id` set). The triggers update the denormalized counts; handlers redirect back so the no-JS form-POST UX works.
- **Read path**: timeline/profile/hashtag handlers call `status_repo::list_*`, then `like_repo::liked_status_ids_for(viewer, ids)` and `follow_repo::following_ids(viewer)` to batch-decorate the rows for the partial.
- **Auth split**: `auth_guard::require_user` is the new shared session check; `auth_guard::require_admin` is `require_user` plus a role assertion. Every existing admin handler call site moved from the old `admin_guard::require_admin` to `auth_guard::require_admin`.
- **Routing flip**: `/` is now the microblog timeline. The blog templates moved to `templates/blog/` and the public blog handlers to `src/handlers/blog/`. Old paths (`/posts/{slug}`, `/categories/{slug}`, `/tags/{slug}`, `/archive`, `/search`, `/about`) return 301 to `/blog/...` via the new `utils::response::redirect_permanent` helper. RSS, sitemap, and robots.txt continue to live at root and emit `/blog/posts/{slug}` URLs.
- **Uploads**: status attachments and user avatars share the multipart pipeline with admin uploads via the extracted `services::upload_service`. Files still land in `storage/uploads/` and serve from `/static/uploads/`.
