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

    /// Serial port
    #[arg(long)]
    port: String,
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let args = Args::parse();
//     setup_tracing();
//     let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;
//     info!("Value is {:?}", config.object.field);
//     Ok(())
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_tracing();
    // let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;

    let mut driver = lss_driver::LSSDriver::new(&args.port)?;

    let mut ids = vec![];

    for id in 0..254 {
        if driver.query_status(id).await.is_ok() {
            ids.push(id);
        }
    }

    info!("Found {} servos {:?}", ids.len(), ids);

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
