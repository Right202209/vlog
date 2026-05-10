# Development Guide

This guide tracks local development commands and the M1 verification flow for Volo Blog.

## Toolchain

The repo uses `rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly"
```

The selected Volo dependency graph (notably `volo-http` 0.5 with edition 2024 and let-chain code) needs a recent compiler. The toolchain is pinned to `nightly` to keep dependency resolution and edition-2024 features happy. The application crate itself still uses Rust 2021 edition.

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

Then verify (note: as of M4 the public blog moved to `/blog/*` and the front door is now the microblog timeline; old `/posts/*` etc. return `301`):

```bash
curl -i http://127.0.0.1:8080/blog
curl -i http://127.0.0.1:8080/blog/posts/hello-world
curl -i http://127.0.0.1:8080/blog/categories/tech
curl -i http://127.0.0.1:8080/blog/tags/rust
curl -i http://127.0.0.1:8080/blog/archive
curl -i "http://127.0.0.1:8080/blog/search?q=hello"
curl -i http://127.0.0.1:8080/blog/about
curl -i http://127.0.0.1:8080/static/css/site.css
curl -i http://127.0.0.1:8080/nope

# Backwards-compat redirects (expect HTTP/1.1 301 + Location header):
curl -i http://127.0.0.1:8080/posts/hello-world
curl -i http://127.0.0.1:8080/categories/tech
```

Expected results:

- Public HTML pages return HTTP 200.
- Unknown paths return HTTP 404.
- HTML responses include `Content-Type: text/html; charset=utf-8`.
- The stylesheet route returns CSS.
- Restarting the server does not duplicate seed data.
- Old blog paths return `301 Moved Permanently` with `Location: /blog/...`.

## M4 Microblog Verification Checklist

End-to-end browser/curl walkthrough after `cargo run`:

1. **Migration** — start once on a fresh DB and once on an existing M2 DB. `users` gets `display_name` / `bio` / `avatar_url` / `role`; the bootstrap admin keeps `role = 'admin'`. No data lost from `posts` / `categories` / `tags`.
2. **Admin creates a user** — sign in at `/admin/login` as `admin`/`admin`, visit `/admin/users`, create `bob` with a temp password. Sign out, sign back in as `bob`. `bob.role` should be `'user'`.
3. **Compose a status** — as `bob`, post from the composer at `/`. Use Markdown plus `@admin` and `#rust` and verify both become links. Attach two images and confirm they appear in the rendered card.
4. **Reply** — as `admin`, open the status, reply. `/s/{id}` should render parent + reply with `reply_count = 1`.
5. **Like / unlike** — like as `admin`, refresh, count → 1, button toggles. Unlike, count → 0.
6. **Repost + quote-repost** — plain repost from `bob`'s status as `admin` shows an "admin reposted" banner above the embedded original. Quote-repost (with extra body text) renders both the comment and the embedded original.
7. **Follow + home timeline** — `bob` visits `/u/admin`, clicks Follow. `/home` as `bob` shows admin's statuses (and bob's own — matches X behaviour).
8. **Profile edit** — `/me/edit` as `bob`: change display name, bio, upload avatar. `/u/bob` reflects the updates and the avatar appears on every card by `bob`.
9. **Hashtag** — `/h/rust` lists all statuses containing `#rust` (LIKE-based; case-insensitive).
10. **Blog still works** — `/blog`, `/blog/posts/hello-world`, `/blog/archive`, `/blog/search?q=hello` all 200. `/posts/hello-world` returns `301`. RSS at `/rss.xml` parses and links into `/blog/posts/...`.
11. **Auth boundaries** — logged-out `POST /compose|/s/{id}/like|/u/{u}/follow` redirects to `/admin/login`. Non-admin `bob` hitting `/admin/posts` or `/admin/users` is forbidden.
12. **CSRF** — every write form (compose, reply, like, repost, follow, profile edit, admin/users/*) carries the per-session token; submitting a tampered token returns 403.

Account creation note: `ADMIN_USERNAME` / `ADMIN_PASSWORD` env vars only seed the very first user (when the `users` table is empty). All additional accounts are created from `/admin/users` after sign-in.

## Current Implementation Notes

- Public route state is currently held in a process-wide `OnceCell<Arc<AppState>>`. This keeps the per-handler boilerplate small while avoiding assumptions about Volo-HTTP state extraction APIs.
- Seed posts store pre-rendered HTML in `content_html`. Markdown rendering on write happens in `admin_post_service::render_html` (blog posts) and `status_service::create_status` (microblog statuses); the read path always reads `content_html` directly.
- RSS, sitemap, and robots.txt are wired (`/rss.xml`, `/sitemap.xml`, `/robots.txt`) and emit `/blog/posts/{slug}` URLs after the M4 routing flip. Structured logs, Docker/systemd packaging, integration tests, and broader SEO meta remain M3 follow-ups.
- Microblog `statuses.reply_count` / `like_count` / `repost_count` are maintained by SQL triggers in `migrations/0003_microblog.sql`. Do not `COUNT(*)` per row at read time and do not write the columns directly from Rust — let the triggers fire on `likes` / `follows` / child `statuses` insert/delete.

