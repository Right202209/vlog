# 轻量级博客系统 PRD

## 1. 文档信息

- 项目名称：Volo Blog
- 文档版本：v0.1
- 文档日期：2026-04-10
- 文档目标：定义一个基于 CloudWeGo Volo-HTTP 的轻量级博客系统 MVP，作为后续实现、拆分任务与验收的统一依据。

## 2. 项目背景

目标是实现一个部署简单、维护成本低、可持续扩展的博客系统，用于个人开发者或小团队发布技术文章、项目更新、产品日志与文档型内容。

项目强调以下原则：

- 单机可用，优先简单部署与低资源占用。
- 内容发布链路完整，先解决“可写、可发、可看、可管理”。
- 技术架构与 Volo-HTTP 的路由、提取器、中间件、静态文件能力保持一致。
- 对未来扩展保留空间，但 MVP 不做重型 CMS。

## 3. 产品目标

### 3.1 核心目标

- 提供公开博客站点，支持文章浏览、分类、标签、归档与 SEO 基础能力。
- 提供后台管理，支持管理员登录、文章 CRUD、草稿/发布、分类标签管理与基础站点设置。
- 提供适合 Volo-HTTP 的轻量实现路径，优先服务端渲染页面与少量 JSON 接口。
- 支持单二进制部署，默认使用 SQLite，降低环境依赖。

### 3.2 非目标

- 不做多租户博客平台。
- 不做复杂权限系统，MVP 仅支持单管理员或少量后台账号。
- 不做评论系统、点赞系统、消息通知系统。
- 不做富文本编辑器，MVP 采用 Markdown 编辑。
- 不做分布式部署、搜索集群、对象存储编排。

## 4. 目标用户与角色

| 角色 | 描述 | 关键诉求 |
| --- | --- | --- |
| 访客 | 浏览博客内容的终端用户 | 快速访问、良好阅读体验、可检索文章 |
| 管理员 | 维护站点和内容的内部用户 | 低成本写作、发布、修改、管理站点基础信息 |
| 编辑（可选） | 未来扩展角色 | 与管理员类似，但权限可受限；MVP 不强制实现 |

## 5. 核心使用场景

### 5.1 访客侧

- 首页查看最新文章列表。
- 进入文章详情页阅读内容。
- 按分类、标签或归档浏览文章。
- 通过关键词搜索文章标题/摘要。
- 访问 RSS、站点地图、关于页等基础内容。

### 5.2 管理侧

- 管理员登录后台。
- 创建文章草稿，填写标题、摘要、正文、标签、分类。
- 预览并发布文章。
- 编辑、下线、删除文章。
- 上传封面图或正文配图。
- 修改站点名称、描述、导航项、页脚信息等基础配置。

## 6. MVP 范围

### 6.1 前台页面

- 首页
- 文章列表页
- 文章详情页
- 分类页
- 标签页
- 归档页
- 搜索结果页
- 关于页
- 404 页面

### 6.2 后台页面

- 登录页
- 控制台首页
- 文章列表页
- 新建/编辑文章页
- 分类管理页
- 标签管理页
- 站点设置页

### 6.3 系统能力

- Markdown 渲染
- 草稿与发布状态
- 文章 slug
- SEO 元信息
- RSS 输出
- sitemap 输出
- 静态文件托管
- 本地文件上传
- 会话认证
- 基础访问日志

## 7. 产品范围细化

### 7.1 文章模块

每篇文章至少包含以下字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| id | 是 | 唯一标识 |
| title | 是 | 文章标题 |
| slug | 是 | URL 标识，需唯一 |
| summary | 否 | 列表摘要，用于首页与 SEO 描述 |
| content_md | 是 | Markdown 原文 |
| content_html | 是 | 渲染后 HTML，便于展示 |
| cover_image | 否 | 封面图地址 |
| status | 是 | draft / published / archived |
| category_id | 否 | 所属分类 |
| created_at | 是 | 创建时间 |
| updated_at | 是 | 更新时间 |
| published_at | 否 | 发布时间 |

文章需支持：

- 新建、保存草稿、更新、发布、取消发布、删除。
- 自动生成或手动指定 slug。
- 支持多标签关联。
- 支持摘要手动填写；未填写时可由正文截断生成。
- 支持阅读数统计，MVP 可做简单计数，不追求强一致。

### 7.2 分类与标签模块

- 分类为单选，标签为多选。
- 分类支持名称、slug、描述。
- 标签支持名称、slug。
- 删除分类时，文章需可迁移到“未分类”或空分类。

### 7.3 搜索模块

- MVP 仅支持站内基础搜索。
- 搜索范围：标题、摘要、标签名。
- 默认采用数据库 `LIKE` 或等价轻量方案实现。
- 不要求全文检索，不引入外部搜索引擎。

### 7.4 站点设置模块

管理员可修改：

- 站点名称
- 站点副标题
- 站点描述
- 页脚版权信息
- 关于页内容
- 首页每页文章数
- SEO 默认标题模板
- 自定义导航项

### 7.5 媒体资源模块

- 支持后台上传图片。
- 默认存储在本地目录，例如 `storage/uploads/`。
- 通过静态文件路径对外访问，例如 `/static/uploads/...`。
- MVP 不要求图片压缩、裁剪、CDN、对象存储。

### 7.6 认证与安全模块

- 管理员使用用户名 + 密码登录。
- 密码必须加盐哈希存储。
- 后台接口必须鉴权。
- 需要基础防暴力破解策略，例如登录失败次数限制或短时限流。
- 对后台表单与写接口进行基础 CSRF/来源校验设计。

## 8. 页面与接口范围

### 8.1 前台路由建议

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 首页 |
| GET | `/posts` | 文章列表 |
| GET | `/posts/{slug}` | 文章详情 |
| GET | `/categories/{slug}` | 分类页 |
| GET | `/tags/{slug}` | 标签页 |
| GET | `/archive` | 归档页 |
| GET | `/search` | 搜索结果页 |
| GET | `/about` | 关于页 |
| GET | `/rss.xml` | RSS |
| GET | `/sitemap.xml` | 站点地图 |

### 8.2 后台路由建议

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/admin/login` | 登录页 |
| POST | `/admin/login` | 提交登录 |
| POST | `/admin/logout` | 退出 |
| GET | `/admin` | 控制台 |
| GET | `/admin/posts` | 文章列表 |
| GET | `/admin/posts/new` | 新建文章页 |
| POST | `/admin/posts` | 创建文章 |
| GET | `/admin/posts/{id}/edit` | 编辑文章页 |
| POST | `/admin/posts/{id}` | 更新文章 |
| POST | `/admin/posts/{id}/publish` | 发布文章 |
| POST | `/admin/posts/{id}/delete` | 删除文章 |
| GET | `/admin/categories` | 分类管理 |
| GET | `/admin/tags` | 标签管理 |
| GET | `/admin/settings` | 站点设置 |
| POST | `/admin/upload` | 上传资源 |

## 9. 关键流程

### 9.1 内容发布流程

1. 管理员登录后台。
2. 新建文章并填写 Markdown 内容。
3. 保存为草稿。
4. 预览文章。
5. 设置标签、分类、摘要、封面。
6. 点击发布。
7. 系统生成对外可访问页面并出现在首页、分类、标签、归档、RSS 中。

### 9.2 访客浏览流程

1. 访客进入首页。
2. 查看文章列表并进入详情页。
3. 在详情页继续通过标签、分类或归档跳转到相关内容。
4. 搜索特定主题并查看结果。

## 10. 数据模型建议

MVP 建议包含以下核心实体：

| 实体 | 说明 |
| --- | --- |
| users | 后台用户 |
| posts | 文章主体 |
| categories | 分类 |
| tags | 标签 |
| post_tags | 文章与标签关联 |
| site_settings | 站点配置 |
| sessions | 登录会话 |
| assets | 上传资源记录 |

约束要求：

- `posts.slug` 唯一。
- `categories.slug` 唯一。
- `tags.slug` 唯一。
- 删除文章时需要同步清理 `post_tags`。
- 发布状态文章必须具备 `published_at`。

## 11. 非功能需求

### 11.1 性能

- 在 2C2G 单机环境下，公开页面读取接口应满足日常博客访问需求。
- 公开页面 P95 响应时间目标小于 200ms（数据库与模板已预热情况下）。
- 后台写操作以稳定性优先，不强求极致低延迟。

### 11.2 可部署性

- 支持本地开发、Docker 部署、systemd 部署。
- 优先实现单二进制 + 配置文件 + SQLite 数据文件的交付方式。
- 启动流程应简单明确，避免引入多服务依赖。

### 11.3 可维护性

- 项目模块边界清晰，至少拆分为路由层、处理层、服务层、存储层、模板层。
- 关键错误需有结构化日志。
- 核心配置项需支持环境变量或配置文件读取。

### 11.4 安全性

- 后台接口默认不暴露调试信息。
- 密码哈希、会话过期、上传类型校验必须具备。
- 上传目录与可执行文件目录隔离。
- 对外错误信息不泄露数据库细节或堆栈。

## 12. 技术约束与实现建议

### 12.1 技术选型原则

- 后端框架：CloudWeGo Volo-HTTP
- 语言：Rust
- 数据库：SQLite（MVP 默认），后续可扩展 PostgreSQL
- 模板渲染：服务端渲染优先，推荐选择 Rust 模板引擎
- 前端策略：少量原生 JS，避免引入重量级前端框架作为 MVP 前提

### 12.2 基于 Volo-HTTP 的实现要求

- 使用 `volo http init` 方式初始化 HTTP 项目脚手架。
- 使用 `Router` 管理前台路由、后台路由与静态资源路由。
- 利用 `Query`、`Form`、`Json` 等提取器处理搜索、登录、文章编辑等请求。
- 使用 `IntoResponse` 返回 HTML、JSON、重定向与错误响应。
- 使用中间件处理日志、超时、认证、通用响应头。
- 使用 `ServeDir` 暴露静态资源与上传文件目录。

### 12.3 建议的目录结构

```text
src/
  bin/server.rs
  lib.rs
  config/
  domain/
  handlers/
  services/
  repositories/
  middleware/
  templates/
  utils/
static/
storage/
  uploads/
migrations/
```

### 12.4 初始化要求

- 遵循官方快速开始的最低环境要求：Rustc >= 1.80.0。
- 初始化后应先完成最小可运行首页与健康检查，再逐步补齐后台和存储层。

## 13. 里程碑建议

### M1：项目初始化与只读博客

- 完成 Volo-HTTP 脚手架初始化。
- 接通首页、文章详情、静态资源、SQLite。
- 实现文章读取与模板渲染。

### M2：后台管理与内容发布

- 完成登录、文章 CRUD、分类标签管理。
- 完成 Markdown 渲染、草稿/发布流程。
- 完成上传与基础设置。

### M3：上线准备

- 完成 RSS、sitemap、SEO 元信息。
- 完成日志、错误页、限流与基础安全策略。
- 补充测试、部署文档与运行手册。

### M4：微博 / X / GNU social 风格微博

- 在保留博客的前提下，将站点首页改造为微博式时间线（Timeline）。
- 引入多用户：管理员可创建账号，公开侧不开放注册。
- 引入用户资料（display_name、bio、avatar）与角色（user / admin）。
- 引入 statuses 主体表，支持回复（threaded reply）、点赞（like）、转发与引用转发（repost / quote）、关注与首页时间线（follow + home timeline）。
- 状态正文沿用 Markdown 渲染，并在渲染后自动解析 `@username` / `#hashtag` 为站内链接。
- 路由调整：`/` 改为全站时间线，`/home` 为关注流（需登录），`/u/{username}` 为个人主页，`/s/{id}` 为状态详情，`/h/{tag}` 为话题页；原博客迁移到 `/blog/*` 并对历史路径返回 301。
- 不在本里程碑内：通知、全文检索、话题索引表、ActivityPub、注册/邮件、图片处理、状态编辑、定时发布、私信、对 compose/like 的限流、shadow-ban 等审核工具。

详细方案见 `docs/M4_MICROBLOG.md`。

## 14. 验收标准

- 管理员可以登录后台并完成一篇文章从创建到发布的全过程。
- 访客可以在前台完成文章浏览、分类/标签浏览、归档浏览与基础搜索。
- 静态资源与上传图片可正常访问。
- 站点重启后数据不会丢失。
- 默认部署方式不依赖 Redis、对象存储、外部搜索服务。
- 代码结构允许后续扩展评论、全文检索、对象存储等能力，而无需推翻主干架构。

## 15. 风险与后续扩展

### 15.1 主要风险

- 如果后台编辑体验要求快速提升，Markdown 纯文本输入可能不够友好。
- 如果文章量快速增长，SQLite + `LIKE` 搜索会成为瓶颈。
- 如果后续引入多作者、多角色，当前简单权限模型需要重构。

### 15.2 后续扩展方向

- 评论系统
- 多用户与 RBAC
- 全文检索
- 对象存储
- Open Graph / 社交分享增强
- 草稿自动保存
- 定时发布
- API Token 与外部发布接口

## 16. 参考资料

- CloudWeGo Volo 文档：https://www.cloudwego.io/zh/docs/volo
- Volo-HTTP 概览：https://www.cloudwego.io/zh/docs/volo/volo-http/overview/
- Volo-HTTP 快速开始：https://www.cloudwego.io/zh/docs/volo/volo-http/getting-started/
- Volo-HTTP 路由：https://www.cloudwego.io/docs/volo/volo-http/tutorials/route/
- Volo-HTTP 请求提取：https://www.cloudwego.io/docs/volo/volo-http/tutorials/request/
- Volo-HTTP 响应：https://www.cloudwego.io/docs/volo/volo-http/tutorials/response/
- Volo-HTTP 中间件：https://www.cloudwego.io/docs/volo/volo-http/tutorials/middleware/
- Volo-HTTP 静态文件：https://www.cloudwego.io/docs/volo/volo-http/tutorials/static-fs/

## 17. M4：微博 / X 风格微博详细范围

### 17.1 用户与角色

- `users` 增加 `display_name`、`bio`、`avatar_url`、`role`（`user` / `admin`，默认 `user`，启动时自举的管理员标记为 `admin`）。
- 注册流程：仅管理员后台 `/admin/users` 可创建账号、重置密码、调整角色、删除账号。公开侧没有注册入口。
- 鉴权：`auth_guard::require_user` 用于普通用户写操作，`auth_guard::require_admin` 用于后台。两者共用 cookie + CSRF 模型。

### 17.2 内容模型

- 新表 `statuses(id, user_id, content_md, content_html, parent_id NULL, repost_of_id NULL, reply_count, like_count, repost_count, created_at)`：
  - `parent_id` 非空表示这是一条回复；
  - `repost_of_id` 非空表示这是转发；正文非空时即为引用转发；
  - 三个 `*_count` 字段由 SQL 触发器在 `likes` / `follows` / `statuses` 子表插入删除时维护。
- 新表 `status_assets(status_id, asset_id, sort)`：复用现有 `assets` 表与 `/static/uploads/` 目录。
- 新表 `likes(user_id, status_id, created_at)`、`follows(follower_id, followee_id, created_at)`，主键均为复合主键。
- 不在本里程碑引入 `status_hashtags` 表；话题页用 `LIKE '%#tag%'` 查询，与现博客搜索一致。

### 17.3 路由

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 全站时间线（顶层 status，不含纯回复） |
| GET | `/home` | 关注流（关注的人 + 自己），需登录 |
| GET | `/s/{id}` | 状态详情 + 回复线程 |
| POST | `/compose` | 发布顶层状态 |
| POST | `/s/{id}/reply` | 回复 |
| POST | `/s/{id}/like` / `/unlike` | 点赞 / 取消 |
| POST | `/s/{id}/repost` / `/unrepost` | 转发 / 取消转发；正文非空即引用转发 |
| POST | `/s/{id}/delete` | 仅作者可删 |
| GET | `/u/{username}` | 个人主页 |
| GET | `/u/{username}/followers` / `/following` | 粉丝 / 关注列表 |
| POST | `/u/{username}/follow` / `/unfollow` | 关注 / 取消关注 |
| GET | `/h/{tag}` | 话题聚合页 |
| GET / POST | `/me/edit` | 编辑当前用户资料 |
| POST | `/me/avatar` | 上传头像 |
| GET / POST | `/admin/users` | 后台用户列表 / 创建 |
| POST | `/admin/users/{id}/reset` | 重置密码 |
| POST | `/admin/users/{id}/role` | 切换角色 |
| POST | `/admin/users/{id}/delete` | 删除账号 |
| GET | `/blog`、`/blog/posts/{slug}`、`/blog/categories/{slug}` 等 | 博客整体迁移到 `/blog/*` |
| GET | 旧 `/posts/{slug}` 等 | 返回 301 到 `/blog/*` |
| GET | `/rss.xml`、`/sitemap.xml`、`/robots.txt` | 维持原路径，仍服务博客内容 |

### 17.4 模板与样式

- 新增 `templates/timeline.html`、`home.html`、`status_detail.html`、`profile.html`、`followers.html`、`following.html`、`hashtag.html`、`me_edit.html`、`_status_card.html`、`_composer.html`、`admin/users.html`。
- 旧博客模板移动到 `templates/blog/`，`#[template(path = "...")]` 同步调整。
- `static/css/site.css` 追加 `===== Microblog (M4) =====` 段落，沿用现有调色板（`--accent`、`--accent-2`、`--soft`、`--surface`、`--line`），不引入字体或图标库。

### 17.5 非目标（M4 内不做）

- 通知系统（站内 / 邮件）。
- 状态全文检索；话题页用 `LIKE`。
- 真正的话题索引表与字符级分词。
- WebSocket / SSE 实时刷新。
- ActivityPub / Diaspora 联邦。
- 公开注册、邮箱验证、邮箱找回密码。
- 图片裁剪、压缩、EXIF 清理、缩略图。
- 状态编辑（X 风格的可编辑窗口）；本期仅支持删除。
- 草稿、定时发布、私信、列表、收藏。
- 对 compose / like / follow 的限流（仅有登录限流）。
- 隐藏 / 锁定 / 影子封禁等审核工具。

### 17.6 验收标准

- 管理员可以创建一名普通用户、用普通用户登录后发布状态、回复、点赞、转发、引用转发、关注其他账号，并在 `/home` 看到关注的人的状态。
- `/u/{username}`、`/s/{id}`、`/h/{tag}` 都能正常渲染。
- 旧博客 `/posts/{slug}`、`/categories/{slug}` 等返回 301，`/blog/*` 内容完整。
- 所有写接口仍受 CSRF 校验保护；非管理员无法访问 `/admin/*`。
- `cargo check` 全量通过。
