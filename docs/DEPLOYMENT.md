# Volo Blog deployment

This document covers the M3 deployment artifacts. The current code targets a single-binary
deployment with SQLite; no external service dependencies are required.

## 1. Required environment

| Variable | Required | Default | Notes |
| --- | --- | --- | --- |
| `HOST` | no | `127.0.0.1` | Bind address. Use `0.0.0.0` inside containers. |
| `PORT` | no | `8080` | Listening port. |
| `DATABASE_URL` | no | `sqlite://vlog.db` | SQLite URL. Use `?mode=rwc` to create the file on first run. |
| `SITE_NAME` | no | from `config/default.toml` | Brand name. |
| `SITE_DESCRIPTION` | no | from `config/default.toml` | Used as the default meta description. |
| `SITE_URL` | yes (prod) | `http://localhost:8080` | Absolute URL of the public site. Used by RSS, sitemap, and Open Graph. |
| `POSTS_PER_PAGE` | no | `10` | Clamped to 1..=100. |
| `ADMIN_USERNAME` | no | `admin` | Only used when bootstrapping the first admin. |
| `ADMIN_PASSWORD` | yes | `admin` | **Set this before first run.** Empty values are rejected. |
| `SESSION_COOKIE_SECURE` | yes (TLS) | unset | Set to `1`/`true` when serving over HTTPS so the session cookie carries the `Secure` attribute. |
| `RUST_LOG` | no | `vlog=info,volo_http=info` | Tracing filter. |

## 2. Docker

```bash
docker build -t vlog:latest .
docker run --rm -p 8080:8080 \
    -e SITE_URL=https://blog.example.com \
    -e ADMIN_USERNAME=admin \
    -e ADMIN_PASSWORD='change-me' \
    -e SESSION_COOKIE_SECURE=1 \
    -v $(pwd)/storage:/app/storage \
    vlog:latest
```

The container runs as a non-root user `vlog` (uid 10001). The SQLite database lives under
`/app/storage` which is exposed as a volume.

## 3. systemd

Install the binary at `/opt/vlog/server`, copy `templates/`, `static/`, `migrations/`,
`config/` next to it, and create a `vlog` system user that owns `/opt/vlog/storage`.

```bash
sudo useradd --system --create-home --shell /usr/sbin/nologin vlog
sudo mkdir -p /opt/vlog/storage /etc/vlog
sudo cp -r target/release/server templates static migrations config /opt/vlog/
sudo chown -R vlog:vlog /opt/vlog/storage
sudo cp deploy/vlog.service /etc/systemd/system/vlog.service
sudo install -m 600 -o root -g root deploy/vlog.env.example /etc/vlog/vlog.env
sudo systemctl daemon-reload
sudo systemctl enable --now vlog
```

`/etc/vlog/vlog.env` should contain at minimum `SITE_URL`, `ADMIN_PASSWORD`, and
`SESSION_COOKIE_SECURE=1`. Remember to rotate `ADMIN_PASSWORD` and force a password
change after first login.

## 4. Reverse proxy

Run behind nginx/Caddy and terminate TLS there. Forward the original `Host` header so
RSS/sitemap absolute URLs match `SITE_URL`.

Minimal nginx fragment:

```nginx
location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

## 5. Backups

Back up `/opt/vlog/storage/vlog.db` and `/opt/vlog/storage/uploads/`. SQLite is fine to
back up while the server is running using `sqlite3 vlog.db ".backup '/path/to/snapshot.db'"`.

## 6. Operational notes

- Login attempts are rate-limited per username: 5 failures within 60 s triggers a 60 s
  lockout. Check the logs for `Login lockout triggered` warnings.
- Sessions expire after 7 days. Expired sessions are purged at every server start.
- Uploads are capped at 5 MiB and limited to PNG/JPEG/GIF/WebP.
