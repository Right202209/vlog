# Volo Blog

[English](README.md) | [简体中文](README.zh-CN.md)

Volo Blog 是一个单二进制 Rust 站点，把 **Weibo / X 风格的微博时间线**与原来的长文 **Markdown 博客**并列在一个进程中，使用 Rust、CloudWeGo Volo-HTTP、SQLite、SQLx 迁移和 Askama 服务端模板实现。

代码目前已覆盖 M1（公开只读博客）、M2（后台认证、内容 CRUD、上传、设置、保存时渲染 Markdown）、M3 上线准备的一部分（信息流 RSS / sitemap / robots、SEO 元信息 Open Graph + canonical、登录限流、部署产物 Dockerfile / systemd / 部署手册），以及 **M4（微博：管理员创建多用户账号，状态支持回复 / 点赞 / 转发 / 引用转发 / 关注，个人主页与话题页，与博客在 `/blog/*` 共存）**。集成测试、自定义 500 页、结构化访问日志、状态通知、以及把动态 `site_settings` 桥接到公开页路径仍未完成。

## 当前范围

已实现的微博页面（M4）：

- `GET /`：全站时间线（顶层状态；登录后顶部显示发帖框）。
- `GET /home`：关注流（关注的人 + 自己；需登录）。
- `GET /s/{id}`：状态详情与回复线程。
- `POST /compose`、`POST /s/{id}/(reply|like|unlike|repost|unrepost|delete)`：写接口（需登录 + CSRF）。
- `GET /u/{username}`、`GET /u/{username}/(followers|following)`、`POST /u/{username}/(follow|unfollow)`：个人主页与关注图。
- `GET /h/{tag}`：话题聚合页（基于 `LIKE`）。
- `GET /me/edit`、`POST /me/edit`、`POST /me/avatar`：个人资料与头像自助。
- `GET/POST /admin/users`、`POST /admin/users/{id}/(reset|role|delete)`：后台用户管理（不开放公开注册）。

已实现的博客页面（M1，于 M4 整体迁移到 `/blog/*`）：

- `GET /blog`、`GET /blog/posts/{slug}`：最新已发布文章列表与详情（含 Open Graph 与 canonical 链接）。
- `GET /blog/categories/{slug}`、`GET /blog/tags/{slug}`：分类与标签页。
- `GET /blog/archive`：按月份分组的归档页。
- `GET /blog/search?q=...`：基于标题、摘要和标签名的基础搜索。
- `GET /blog/about`：关于页面。
- `GET /rss.xml`、`GET /sitemap.xml`、`GET /robots.txt`：RSS / 站点地图链接到 `/blog/posts/{slug}`。
- `GET /static/css/site.css`、`GET /static/uploads/{file}`。
- 旧路径 `/posts`、`/posts/{slug}`、`/categories/{slug}`、`/tags/{slug}`、`/archive`、`/search`、`/about` 返回 `301` 到对应 `/blog/...`。
- 兜底 404 页面。

已实现的后台页面（M2 + M4）：

- `GET/POST /admin/login` 与 `POST /admin/logout`：argon2 密码哈希 + SQLite 存储的会话 cookie（`vlog_session`）。
- `GET /admin`：控制台，展示文章 / 分类 / 标签计数。
- `GET/POST /admin/posts` 等：完整文章 CRUD、草稿/发布切换、删除、保存时渲染 Markdown。
- `GET/POST /admin/categories` 与 `GET/POST /admin/tags`：分类与标签 CRUD。
- `GET/POST /admin/settings`：可编辑的站点设置（写入 `site_settings` 表）。
- `POST /admin/upload`：接收 `multipart/form-data` 图片上传（PNG / JPEG / GIF / WebP，最大 5 MiB，按块流式写入临时文件并校验大小）。同一份多部件管线由 `services::upload_service` 抽出，状态附图与头像上传都复用它。
- `GET/POST /admin/users` 等：仅管理员可见的用户管理（创建账号、重置密码、调整角色、删除账号）。

所有后台写接口（含 `/admin/logout`）都需要会话 cookie 与每会话一份的 `csrf_token` 表单字段，CSRF 校验采用常时比较。微博写接口共用同一套 CSRF 机制。`vlog_session` cookie 默认 `HttpOnly; SameSite=Lax`，当设置 `SESSION_COOKIE_SECURE=1` 时会附加 `Secure`。HTML 响应附带 `X-Content-Type-Options: nosniff` 与 `Referrer-Policy: same-origin`，后台页面还会设置严格的默认源 CSP。

登录按用户名（小写）做限流：60 秒内 5 次失败将触发 60 秒锁定，状态保存在进程内存中，超限返回 HTTP 429 并附带 `Retry-After`。

尚未实现：

- 公开页面尚未读取动态 `site_settings`，仍来自 `config/default.toml`。
- 自定义 500 / 5xx 错误页（出错时返回纯文本）。
- 结构化的请求访问日志中间件（仅 `tracing` 默认输出）。
- 通知、状态全文检索、实时刷新、联邦化、公开注册、图片处理、状态编辑、草稿、私信。
- 集成测试。

## 技术栈

- HTTP：`volo-http`
- 运行时：Volo 使用的 Tokio
- 数据库：SQLite，通过 `sqlx`
- 迁移：嵌入式 `sqlx::migrate!`
- 模板：Askama（HTML + XML）
- Markdown：`pulldown-cmark`
- 密码哈希：`argon2`
- 时间格式化：`chrono`（RSS 用 RFC 2822、站点地图用 ISO date）
- 配置：`config/default.toml` 加环境变量覆盖
- 日志：`tracing` 和 `tracing-subscriber`

## 环境要求

- 通过 `rustup` 管理的 Rust 工具链。
- 仓库在 `rust-toolchain.toml` 中将工具链通道固定为 `nightly`。

原始 PRD 的最低要求是 Rust 1.80。实现过程中，最新的 Volo 依赖图选择了一些需要更新 Cargo/Rust 支持的 crate（包括 edition-2024 manifest 和 `volo-http` 中的 Rust 2024 let chains），因此将工具链通道升至 `nightly`，同时本 crate 仍保持 Rust 2021 edition。

## 本地运行

在仓库根目录执行：

```bash
cargo run
```

服务会读取 `config/default.toml`，按需创建 `storage/uploads/`，打开 `vlog.db`，运行嵌入式迁移，以幂等方式写入示例内容，并监听：

```text
http://127.0.0.1:8080
```

## 配置

默认配置位于 `config/default.toml`：

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

支持的环境变量覆盖：

- `SITE_NAME`
- `SITE_DESCRIPTION`
- `SITE_URL`（用于 RSS、站点地图、robots.txt、canonical 链接和 Open Graph 的绝对基础 URL；尾部斜杠会被去掉）
- `HOST`
- `PORT`
- `DATABASE_URL`
- `POSTS_PER_PAGE`（夹紧到 1..=100）
- `ADMIN_USERNAME`（默认 `admin`，仅在首次启动初始化管理员时使用）
- `ADMIN_PASSWORD`（默认 `admin`，仅在首次启动初始化管理员时使用；空值会被拒绝）
- `SESSION_COOKIE_SECURE`（设为 `1` / `true` / `yes` 时，会话 cookie 会附加 `Secure` 属性，需在 HTTPS 下使用）
- `RUST_LOG`（tracing 过滤器，默认 `vlog=info,volo_http=info`）

示例：

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
SITE_URL=https://blog.example.com SESSION_COOKIE_SECURE=1 ADMIN_PASSWORD='change-me' cargo run
```

## 项目结构

```text
config/                 运行时配置
deploy/                 systemd 单元 + 环境变量示例
docs/                   开发、架构和部署说明
migrations/             嵌入式 SQL 迁移
static/css/site.css     公开样式表
storage/uploads/        本地上传存储目录（首次运行时创建）
templates/              Askama HTML 与 XML 模板
src/bin/server.rs       服务入口
src/config/             设置加载
src/domain/             简单领域结构体
src/repositories/       SQLite 查询层
src/services/           读模型组装、认证、后台守卫、限流
src/handlers/           Volo 路由处理器（公开、信息流、后台）
src/templates.rs        Askama 模板结构体与 HTML/XML 响应适配器
src/utils/              Markdown、错误、时间、cookie、密码、slug、token 等辅助
Dockerfile              多阶段构建（nightly builder → debian-slim 运行时）
```

更多文档：

- `docs/DEVELOPMENT.md`：命令、本地运行时文件和 M1/M2 验证清单。
- `docs/ARCHITECTURE.md`：请求流、分层、数据模型和里程碑边界。
- `docs/DEPLOYMENT.md`：环境变量、Docker、systemd、反向代理与备份说明。

## 数据库

启动时按顺序应用以下迁移：

- `0001_initial.sql`：创建 `posts`、`categories`、`tags`、`post_tags`，并写入两篇示例文章。
- `0002_admin.sql`：创建 `users`、`sessions`、`site_settings`、`assets`，并写入 `site_settings` 默认值。
- `0003_microblog.sql`：扩展 `users`（增加 `display_name`、`bio`、`avatar_url`、`role`）；创建 `statuses`、`status_assets`、`likes`、`follows`；安装 SQL 触发器自动维护 `statuses.reply_count` / `like_count` / `repost_count`。

种子数据使用 `ON CONFLICT` / `INSERT OR IGNORE`，因此重启不会重复写入。如果 `users` 表为空，服务会自动创建默认管理员（`admin` / `admin`，可通过 `ADMIN_USERNAME` / `ADMIN_PASSWORD` 覆盖）。**生产环境上线前请修改默认密码。** 启动后再创建的账号通过 `/admin/users` 完成，不再依赖环境变量。

本地 SQLite 运行时文件会被 Git 忽略：

- `vlog.db`
- `vlog.db-shm`
- `vlog.db-wal`

## 验证状态

M1 只读实现、M2 后台与内容管理层、以及 M3 上线准备的一部分（信息流、SEO 元信息、登录限流、部署产物）均已就位。端到端编译与路由验证留给用户在本地完成。

工具链就绪后建议执行的验证：

```bash
cargo check
cargo run
# 微博
curl -i http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/u/admin
curl -i http://127.0.0.1:8080/h/rust
# 博客（迁移到 /blog）
curl -i http://127.0.0.1:8080/blog
curl -i http://127.0.0.1:8080/blog/posts/hello-world
curl -i http://127.0.0.1:8080/blog/categories/tech
curl -i http://127.0.0.1:8080/blog/tags/rust
curl -i http://127.0.0.1:8080/blog/archive
curl -i "http://127.0.0.1:8080/blog/search?q=hello"
curl -i http://127.0.0.1:8080/blog/about
curl -i http://127.0.0.1:8080/static/css/site.css
curl -i http://127.0.0.1:8080/nope
# 旧路径 301 跳转
curl -i http://127.0.0.1:8080/posts/hello-world
curl -i http://127.0.0.1:8080/categories/tech
# M3 信息流与 SEO
curl -i http://127.0.0.1:8080/rss.xml
curl -i http://127.0.0.1:8080/sitemap.xml
curl -i http://127.0.0.1:8080/robots.txt
# 登录限流（连续 5 次失败后应返回 HTTP 429 + Retry-After）
for i in 1 2 3 4 5 6; do
  curl -is -X POST -d 'username=admin&password=wrong' http://127.0.0.1:8080/admin/login | head -1
done
```

预期结果：

- HTML 路由返回 `Content-Type: text/html; charset=utf-8`，并附带 `X-Content-Type-Options: nosniff`。
- `/rss.xml` 与 `/sitemap.xml` 返回 `Content-Type: application/xml; charset=utf-8`。
- `/robots.txt` 返回 `Content-Type: text/plain; charset=utf-8`，引用 `SITE_URL`。
- `/static/css/site.css` 返回样式表。
- 未知路由返回 HTTP 404，并渲染 404 页面。
- 重启应用后，SQLite 中的种子数据不会产生重复行。
- 同一用户名连续 5 次密码错误后，后续请求在 60 秒内返回 HTTP 429 与 `Retry-After`。

## 部署

完整说明见 `docs/DEPLOYMENT.md`。快速上手：

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

## 里程碑

M1：只读博客。

- 项目脚手架。
- SQLite schema 和示例数据。
- 公开路由处理器。
- Askama 模板。
- 静态 CSS。

M2：后台和内容管理。

- 登录/session 认证。
- CSRF 保护。
- 文章、分类、标签 CRUD。
- 保存时渲染 Markdown。
- 上传端点和本地资源记录。
- 站点设置表单。

M3：发布准备（进行中）。

- RSS、站点地图、robots.txt — 已完成。
- Open Graph + canonical 等 SEO 元信息 — 已完成。
- 登录限流、安全响应头、可选的 `Secure` cookie — 已完成。
- Dockerfile、systemd 单元、部署手册 — 已完成。
- 自定义 500 页、结构化访问日志中间件、集成测试 — 仍待完成。

M4：微博（Weibo / X / GNU social 风格）。

- 多用户账号（仅管理员创建，不开放公开注册）。
- `users.role`（`user` / `admin`）配合 `auth_guard::require_user` 与 `require_admin`。
- `statuses` 表支持回复线程、点赞、转发、引用转发；计数列由 SQL 触发器维护。
- `follows` 关注图与 `/home` 关注流。
- 个人主页（`/u/{username}`）、话题页（`/h/{tag}`）、自助资料与头像（`/me/edit`、`/me/avatar`）。
- 站点首页 `/` 改为微博时间线，原博客整体迁到 `/blog/*`，旧路径 `301` 跳转。
- 详细规格见 `docs/M4_MICROBLOG.md`。
