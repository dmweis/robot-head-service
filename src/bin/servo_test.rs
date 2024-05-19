use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use clap::Parser;
use lss_driver::{CommandModifier, LSSDriver, MotorStatus};
use robot_head_service::{turn_off_display, turn_on_display};
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

const BASE_MOTOR_ID: u8 = 1;
const NECK_MOTOR_A_ID: u8 = 2;

const ALL_MOTOR_IDS: [u8; 2] = [BASE_MOTOR_ID, NECK_MOTOR_A_ID];

const NECK_MOTOR_A_NORMAL_POSITION: f32 = -10.0;

struct HeadController {
    driver: LSSDriver,
}

impl HeadController {
    fn new(port: &str) -> Result<Self> {
        let driver = lss_driver::LSSDriver::new(port)?;
        Ok(Self { driver })
    }

    async fn configure(&mut self) -> Result<()> {
        for id in ALL_MOTOR_IDS {
            self.driver
                .configure_color(id, lss_driver::LedColor::Off)
                .await?;
            self.driver
                .set_color(id, lss_driver::LedColor::Cyan)
                .await?;

            self.driver.set_motion_profile(id, true).await?;

            let position = self.driver.query_position(id).await?;
            info!("{} position is {}", id, position);
        }
        Ok(())
    }

    async fn move_base_to(&mut self, angle: f32, modifier: CommandModifier) -> Result<()> {
        self.driver
            .move_to_position_with_modifier(BASE_MOTOR_ID, angle, modifier)
            .await?;
        Ok(())
    }

    async fn move_neck_to(&mut self, angle: f32, speed: u32) -> Result<()> {
        self.driver
            .move_to_position_with_modifier(
                NECK_MOTOR_A_ID,
                angle,
                CommandModifier::SpeedDegrees(speed),
            )
            .await?;
        Ok(())
    }

    async fn wait_until_base_in_position(&mut self) -> Result<()> {
        sleep(Duration::from_millis(500)).await;
        loop {
            let status = self.driver.query_status(BASE_MOTOR_ID).await?;
            match status {
                MotorStatus::Accelerating | MotorStatus::Decelerating | MotorStatus::Traveling => {}
                _ => return Ok(()),
            }
            sleep(Duration::from_millis(250)).await;
        }
    }

    async fn wait_until_neck_in_position(&mut self) -> Result<()> {
        sleep(Duration::from_millis(500)).await;
        loop {
            let status = self.driver.query_status(NECK_MOTOR_A_ID).await?;
            match status {
                MotorStatus::Accelerating | MotorStatus::Decelerating | MotorStatus::Traveling => {}
                _ => return Ok(()),
            }
            sleep(Duration::from_millis(250)).await;
        }
    }

    async fn turn_off(&mut self) -> Result<()> {
        for id in ALL_MOTOR_IDS {
            self.driver.set_color(id, lss_driver::LedColor::Off).await?;
        }
        self.driver.limp(BASE_MOTOR_ID).await?;
        Ok(())
    }

    async fn limp_neck(&mut self) -> Result<()> {
        self.driver.limp(NECK_MOTOR_A_ID).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_tracing();
    // let config = robot_head_service::configuration::AppConfig::load_config(&args.config)?;
    turn_off_display().await?;

    let mut head_controller = HeadController::new(&args.port)?;

    head_controller.configure().await?;

    head_controller
        .move_neck_to(NECK_MOTOR_A_NORMAL_POSITION, 45)
        .await?;
    head_controller.wait_until_neck_in_position().await?;

    head_controller
        .move_base_to(75.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    turn_on_display().await?;

    sleep(Duration::from_secs(2)).await;

    head_controller
        .move_base_to(90.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(-90.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(75.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller
        .move_base_to(0.0, CommandModifier::SpeedDegrees(60))
        .await?;
    head_controller.wait_until_base_in_position().await?;

    head_controller.move_neck_to(40.0, 45).await?;
    head_controller.wait_until_neck_in_position().await?;

    sleep(Duration::from_secs(1)).await;

    head_controller.turn_off().await?;
    head_controller.limp_neck().await?;

    turn_off_display().await?;

    info!("finished");

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
