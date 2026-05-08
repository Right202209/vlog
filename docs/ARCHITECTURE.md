# Architecture Notes

Volo Blog is organized around a small server-rendered Rust application.

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

Route handlers for public pages. M2 will add `src/handlers/admin/`.

`src/repositories/`

SQLx query functions. The repository layer owns SQL shape and pagination queries.

`src/services/`

Read-model assembly. M1 uses this to attach category and tag data to post list items.

`src/domain/`

Plain structs used by repositories, services, and templates.

`src/templates.rs`

Askama template structs plus the `HtmlTemplate<T>` response adapter that sets the HTML content type.

`templates/`

Server-rendered HTML files using Askama inheritance from `base.html`.

`migrations/`

Embedded SQL migrations run at startup through `sqlx::migrate!`.

## Data Model

M1 uses four tables:

- `posts`
- `categories`
- `tags`
- `post_tags`

Posts can belong to one category and many tags. Only rows with `status = 'published'` are exposed by public routes.

## Milestone Boundaries

M1 is read-only and intentionally keeps authentication, writes, uploads, RSS, sitemap, deployment files, and integration tests out of scope.

M2 should introduce admin-specific modules rather than expanding public handlers. Expected additions include auth services, session storage, CSRF middleware, post/category/tag forms, and upload handling.

M3 should add feed/SEO/deployment concerns after the public and admin flows are stable.

