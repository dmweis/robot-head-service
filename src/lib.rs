use std::time::Duration;

use anyhow::Result;
use lss_driver::{CommandModifier, LSSDriver, MotorStatus};
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod configuration;
pub mod error;

pub async fn turn_on_display() -> anyhow::Result<()> {
    // XDG_RUNTIME_DIR="/run/user/1000" WAYLAND_DISPLAY="wayland-1" wlr-randr --output HDMI-A-1 --on --transform 90
    let status = tokio::process::Command::new("wlr-randr")
        .env("XDG_RUNTIME_DIR", "/run/user/1000")
        .env("WAYLAND_DISPLAY", "wayland-1")
        .arg("--output")
        .arg("HDMI-A-1")
        .arg("--on")
        .arg("--transform")
        .arg("270")
        .status()
        .await?;
    info!("Turning on display {:?}", status);
    Ok(())
}

pub async fn turn_off_display() -> anyhow::Result<()> {
    // XDG_RUNTIME_DIR="/run/user/1000" WAYLAND_DISPLAY="wayland-1" wlr-randr --output HDMI-A-1 --off
    let status = tokio::process::Command::new("wlr-randr")
        .env("XDG_RUNTIME_DIR", "/run/user/1000")
        .env("WAYLAND_DISPLAY", "wayland-1")
        .arg("--output")
        .arg("HDMI-A-1")
        .arg("--off")
        .status()
        .await?;
    info!("Turning off display {:?}", status);
    Ok(())
}

const BASE_MOTOR_ID: u8 = 1;
const NECK_MOTOR_A_ID: u8 = 2;

const ALL_MOTOR_IDS: [u8; 2] = [BASE_MOTOR_ID, NECK_MOTOR_A_ID];

pub const NECK_MOTOR_A_NORMAL_POSITION: f32 = -10.0;

pub struct HeadController {
    driver: LSSDriver,
}

impl HeadController {
    pub fn new(port: &str) -> Result<Self> {
        let driver = lss_driver::LSSDriver::new(port)?;
        Ok(Self { driver })
    }

    pub async fn configure(&mut self) -> Result<()> {
        for id in ALL_MOTOR_IDS {
            self.driver.set_origin_offset(id, 0.0).await?;
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

    pub async fn move_base_to(&mut self, angle: f32, modifier: CommandModifier) -> Result<()> {
        self.driver
            .move_to_position_with_modifier(BASE_MOTOR_ID, angle, modifier)
            .await?;
        Ok(())
    }

    pub async fn move_neck_to(&mut self, angle: f32, speed: u32) -> Result<()> {
        self.driver
            .move_to_position_with_modifier(
                NECK_MOTOR_A_ID,
                angle,
                CommandModifier::SpeedDegrees(speed),
            )
            .await?;
        Ok(())
    }

    pub async fn wait_until_base_in_position(&mut self) -> Result<()> {
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

    pub async fn wait_until_neck_in_position(&mut self) -> Result<()> {
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

    pub async fn turn_off(&mut self) -> Result<()> {
        for id in ALL_MOTOR_IDS {
            self.driver.set_color(id, lss_driver::LedColor::Off).await?;
        }
        self.driver.limp(BASE_MOTOR_ID).await?;
        Ok(())
    }

    pub async fn limp_neck(&mut self) -> Result<()> {
        self.driver.limp(NECK_MOTOR_A_ID).await?;
        Ok(())
    }
}

pub fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
