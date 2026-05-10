use std::net::{IpAddr, SocketAddr};

use tracing_subscriber::EnvFilter;
use volo::net::Address;
use volo_http::Server;

#[volo::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("vlog=info,volo_http=info")
        }))
        .init();

    if let Err(error) = run().await {
        tracing::error!(%error, "server failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = vlog::config::load()?;
    std::fs::create_dir_all(&settings.upload_dir)?;

    let pool = vlog::repositories::db::init_pool(&settings.database_url).await?;

    if admin_password_env_is_blank_or_missing()
        && !is_loopback_host(&settings.host)
        && vlog::repositories::user_repo::count(&pool).await? == 0
    {
        return Err(
            "ADMIN_PASSWORD must be set and non-empty before bootstrapping an admin user on a non-localhost bind"
                .into(),
        );
    }
    vlog::services::auth_service::ensure_default_admin(&pool).await?;
    let _ = vlog::repositories::session_repo::purge_expired(&pool).await;

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    let app = vlog::build_router(vlog::AppState {
        settings: settings.clone(),
        pool,
    });

    tracing::info!("Listening on {}", addr);
    Server::new(app).run(Address::from(addr)).await?;
    Ok(())
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .map(|addr| addr.is_loopback())
            .unwrap_or(false)
}

fn admin_password_env_is_blank_or_missing() -> bool {
    std::env::var("ADMIN_PASSWORD")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
}
