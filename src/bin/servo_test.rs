use std::{path::PathBuf, time::Duration};

use clap::Parser;
use lss_driver::CommandModifier;
use robot_head_service::{
    setup_tracing, turn_off_display, turn_on_display, HeadController, NECK_MOTOR_A_NORMAL_POSITION,
};
use tokio::time::sleep;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, author)]
struct Args {
    /// Config path
    #[arg(long)]
    config: Option<PathBuf>,

    /// Serial port
    #[arg(long, default_value = "/dev/ttyUSB0")]
    port: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_tracing();
    // let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;

    let mut head_controller = HeadController::new(&args.port)?;
    turn_on_display().await?;
    head_controller.configure().await?;
    head_controller
        .move_neck_to(NECK_MOTOR_A_NORMAL_POSITION, 45)
        .await?;
    head_controller.wait_until_neck_in_position().await?;

    // start movement
    head_controller
        .move_base_to(65.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(90.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(-90.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(65.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(0.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller.move_neck_to(45.0, 30).await?;
    head_controller.wait_until_neck_in_position().await?;

    sleep(Duration::from_secs(1)).await;

    head_controller.turn_off().await?;
    head_controller.limp_neck().await?;

    turn_off_display().await?;

    info!("finished");

    Ok(())
}
