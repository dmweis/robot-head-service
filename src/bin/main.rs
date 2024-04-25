use std::path::PathBuf;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, author)]
struct Args {
    /// Config path
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_tracing();
    let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;
    info!("Value is {:?}", config.object.field);
    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
