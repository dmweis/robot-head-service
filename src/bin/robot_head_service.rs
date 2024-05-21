use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use clap::Parser;
use lss_driver::CommandModifier;
use robot_head_service::{
    error::ErrorWrapper, setup_tracing, HeadController, NECK_MOTOR_A_NORMAL_POSITION,
};
use tokio::time::sleep;
use tracing::{error, info};
use zenoh::prelude::r#async::*;

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
    info!("Loading config {:?}", args.config);
    let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;

    info!("Starting LSS controller {:?}", &config.motors);
    let mut head_controller = HeadController::with_config(&config.motors)?;
    head_controller.configure().await?;

    info!("Starting zenoh");
    let zenoh_config = zenoh::config::Config::default();
    let session = zenoh::open(zenoh_config)
        .res()
        .await
        .map_err(ErrorWrapper::ZenohError)
        .context("Failed to create zenoh session")?
        .into_arc();

    let command_subscriber = session
        .declare_subscriber("robot-head/command")
        .res()
        .await
        .map_err(ErrorWrapper::ZenohError)
        .context("Failed to create subscriber")?;

    while let Ok(message) = command_subscriber.recv_async().await {
        let json_message: String = message
            .value
            .try_into()
            .context("Failed to convert value to string")?;
        let command = match serde_json::from_str::<RobotHeadCommand>(&json_message) {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to parse json {:?}", err);
                RobotHeadCommand::default()
            }
        };

        if let Some(active) = command.active {
            if !active {
                head_controller
                    .move_base_to(0.0, CommandModifier::SpeedDegrees(60))
                    .await?;
                head_controller.wait_until_base_in_position().await?;
                head_controller.move_neck_to(45.0, 30).await?;
                head_controller.wait_until_neck_in_position().await?;
                sleep(Duration::from_secs(1)).await;
                head_controller.turn_off().await?;
                head_controller.limp_neck().await?;
            } else {
                head_controller
                    .move_base_to(0.0, CommandModifier::SpeedDegrees(60))
                    .await?;
                head_controller
                    .move_neck_to(NECK_MOTOR_A_NORMAL_POSITION, 45)
                    .await?;
            }
        }

        if let Some(yaw) = command.yaw {
            head_controller
                .move_base_to(yaw, CommandModifier::SpeedDegrees(60))
                .await?;
        }
        if let Some(pitch) = command.pitch {
            head_controller.move_neck_to(pitch, 60).await?;
        }
    }

    Ok(())
}

#[derive(serde::Deserialize, Default)]
struct RobotHeadCommand {
    /// head is lifted in active mode
    #[serde(default)]
    active: Option<bool>,
    /// direction in which we are looking
    /// center is 0.0
    #[serde(default)]
    yaw: Option<f32>,
    /// angle at which we are looking
    /// 0.0 is flat horizon
    #[serde(default)]
    pitch: Option<f32>,
}
