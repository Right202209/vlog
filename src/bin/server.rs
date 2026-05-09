use std::net::SocketAddr;

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

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> 
{
    let settings = vlog::config::load()?;
    std::fs::create_dir_all(&settings.upload_dir)?;

    let pool = vlog::repositories::db::init_pool(&settings.database_url).await?;
    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    let app = vlog::build_router(vlog::AppState {
        settings: settings.clone(),
        pool,
    });

    tracing::info!("Listening on {}", addr);
    Server::new(app).run(Address::from(addr)).await?;
    Ok(())
}

