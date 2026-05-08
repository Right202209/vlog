# Volo Blog

[English](README.md) | [简体中文](README.zh-CN.md)

Volo Blog 是一个轻量级 Markdown 博客，使用 Rust、CloudWeGo Volo-HTTP、SQLite、SQLx 迁移和 Askama 服务端模板实现。

当前代码库对应 `Prd.md` 中的 M1：项目初始化和只读公开博客。管理员登录、内容 CRUD、上传、RSS、站点地图、SEO 扩展和部署加固计划在后续里程碑中完成。

## 当前范围

已实现的公开页面：

- `GET /` 和 `GET /posts`：最新已发布文章列表。
- `GET /posts/{slug}`：文章详情。
- `GET /categories/{slug}`：分类页。
- `GET /tags/{slug}`：标签页。
- `GET /archive`：按月份分组的归档页。
- `GET /search?q=...`：基于标题、摘要和标签名的基础搜索。
- `GET /about`：关于页面。
- `GET /static/css/site.css`：站点样式表。
- 兜底 404 页面。

尚未实现：

- 管理员认证和 CRUD。
- 上传处理。
- RSS 和站点地图。
- 生产部署文件。
- 集成测试。

## 技术栈

- HTTP：`volo-http`
- 运行时：Volo 使用的 Tokio
- 数据库：SQLite，通过 `sqlx`
- 迁移：嵌入式 `sqlx::migrate!`
- 模板：Askama
- Markdown：`pulldown-cmark`
- 配置：`config/default.toml` 加环境变量覆盖
- 日志：`tracing` 和 `tracing-subscriber`

## 环境要求

- 通过 `rustup` 管理的 Rust 工具链。
- 仓库在 `rust-toolchain.toml` 中固定 Rust `1.86`。

原始 PRD 的最低要求是 Rust 1.80。实现过程中，最新的 Volo 依赖图选择了一些需要更新 Cargo/Rust 支持的 crate，因此将固定版本提升到 1.86，同时本 crate 仍保持 Rust 2021 edition。

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
- `HOST`
- `PORT`
- `DATABASE_URL`
- `POSTS_PER_PAGE`

示例：

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
```

## 项目结构

```text
config/                 运行时配置
docs/                   开发和架构说明
migrations/             嵌入式 SQL 迁移
static/css/site.css     公开样式表
storage/uploads/        未来的本地上传存储目录
templates/              Askama HTML 模板
src/bin/server.rs       服务入口
src/config/             设置加载
src/domain/             简单领域结构体
src/repositories/       SQLite 查询层
src/services/           读模型组装
src/handlers/           Volo 路由处理器
src/templates.rs        Askama 模板结构体和 HTML 响应适配器
src/utils/              Markdown 和错误辅助工具
```

更多文档：

- `docs/DEVELOPMENT.md`：命令、本地运行时文件和 M1 验证清单。
- `docs/ARCHITECTURE.md`：请求流、分层、数据模型和里程碑边界。

## 数据库

第一个迁移会创建：

- `posts`
- `categories`
- `tags`
- `post_tags`

它还会插入示例分类、标签和两篇已发布文章，并使用 `ON CONFLICT` / `INSERT OR IGNORE`，因此重启应用不会重复写入种子数据。

本地 SQLite 运行时文件会被 Git 忽略：

- `vlog.db`
- `vlog.db-shm`
- `vlog.db-wal`

## 验证状态

依赖设置完成后已经启动过 `cargo check`，但上一轮工作在完成前被中断。路由级验证仍待完成。

编译通过后的预期 M1 验证：

```bash
cargo check
cargo run
curl -i http://127.0.0.1:8080/
curl -i http://127.0.0.1:8080/posts/hello-world
curl -i http://127.0.0.1:8080/categories/tech
curl -i http://127.0.0.1:8080/tags/rust
curl -i http://127.0.0.1:8080/archive
curl -i "http://127.0.0.1:8080/search?q=hello"
curl -i http://127.0.0.1:8080/about
curl -i http://127.0.0.1:8080/static/css/site.css
curl -i http://127.0.0.1:8080/nope
```

预期结果：

- HTML 路由返回 `Content-Type: text/html; charset=utf-8`。
- `/static/css/site.css` 返回样式表。
- 未知路由返回 HTTP 404，并渲染 404 页面。
- 重启应用后，SQLite 中的种子数据不会产生重复行。

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

M3：发布准备。

- RSS 和站点地图。
- SEO meta 字段。
- 结构化请求日志。
- 生产错误页。
- Docker/systemd/release 打包。
- 集成测试和运行手册加固。
