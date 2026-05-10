# Volo Blog

[English](README.md) | [简体中文](README.zh-CN.md)

单二进制 Rust 站点，把 Weibo / X 风格的微博时间线与长文 Markdown 博客并列在一个进程中，基于 CloudWeGo Volo-HTTP、SQLite（通过 SQLx）、Askama 服务端模板和 Argon2 会话；migrations 嵌入二进制。

微博（回复、点赞、转发、引用转发、关注、个人主页、话题页）位于 `/`；博客（CRUD、RSS、sitemap、搜索、归档）迁到 `/blog/*`。账号仅管理员创建，不开放公开注册。

## 运行

```bash
cargo run
# http://127.0.0.1:8080
```

服务读取 `config/default.toml`，按需创建 `storage/uploads/`，打开 `vlog.db`，运行嵌入式迁移；若 `users` 为空则自动创建默认管理员（`admin` / `admin`）。**生产环境上线前请修改默认密码。**

## 配置

默认值在 `config/default.toml`。环境变量覆盖：`SITE_NAME`、`SITE_DESCRIPTION`、`SITE_URL`、`HOST`、`PORT`、`DATABASE_URL`、`POSTS_PER_PAGE`、`ADMIN_USERNAME`、`ADMIN_PASSWORD`（仅首次启动时使用）、`SESSION_COOKIE_SECURE`（HTTPS 时设置）、`RUST_LOG`。

```bash
PORT=3000 DATABASE_URL=sqlite://dev.db cargo run
SITE_URL=https://blog.example.com SESSION_COOKIE_SECURE=1 ADMIN_PASSWORD='change-me' cargo run
```

## 工具链

`rust-toolchain.toml` 把通道锁在 `nightly`：所选的 `volo` / `volo-http` 0.5 依赖图要求 edition-2024 manifest 和 Rust 2024 let chains。本 crate 自身仍是 Rust 2021 edition。

## 文档

- `docs/DEVELOPMENT.md`：命令、运行时文件、验证清单。
- `docs/ARCHITECTURE.md`：请求流、分层、数据模型、里程碑边界。
- `docs/DEPLOYMENT.md`：Docker、systemd、反代、环境变量、备份。
- `docs/M4_MICROBLOG.md`：微博详细规格。
