use std::time::Duration;

use anyhow::Result;
use configuration::MotorsConfig;
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

pub const DEFAULT_BASE_MOTOR_ID: u8 = 1;
pub const DEFAULT_NECK_MOTOR_A_ID: u8 = 2;

pub const NECK_MOTOR_A_NORMAL_POSITION: f32 = -10.0;

pub struct HeadController {
    driver: LSSDriver,
    base_motor_id: u8,
    neck_motor_id: u8,
}

impl HeadController {
    pub fn new(port: &str, base_motor_id: u8, neck_motor_id: u8) -> Result<Self> {
        let driver = lss_driver::LSSDriver::new(port)?;
        Ok(Self {
            driver,
            base_motor_id,
            neck_motor_id,
        })
    }

    pub fn with_config(config: &MotorsConfig) -> Result<Self> {
        Self::new(
            &config.serial_port,
            config.base_motor_id,
            config.neck_motor_id,
        )
    }

    fn all_motor_ids(&self) -> Vec<u8> {
        vec![self.base_motor_id, self.neck_motor_id]
    }

    pub async fn configure(&mut self) -> Result<()> {
        for id in self.all_motor_ids() {
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
            .move_to_position_with_modifier(self.base_motor_id, angle, modifier)
            .await?;
        Ok(())
    }

    pub async fn move_neck_to(&mut self, angle: f32, speed: u32) -> Result<()> {
        self.driver
            .move_to_position_with_modifier(
                self.neck_motor_id,
                angle,
                CommandModifier::SpeedDegrees(speed),
            )
            .await?;
        Ok(())
    }

    pub async fn wait_until_base_in_position(&mut self) -> Result<()> {
        sleep(Duration::from_millis(500)).await;
        loop {
            let status = self.driver.query_status(self.base_motor_id).await?;
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
            let status = self.driver.query_status(self.neck_motor_id).await?;
            match status {
                MotorStatus::Accelerating | MotorStatus::Decelerating | MotorStatus::Traveling => {}
                _ => return Ok(()),
            }
            sleep(Duration::from_millis(250)).await;
        }
    }

    pub async fn turn_off(&mut self) -> Result<()> {
        for id in self.all_motor_ids() {
            self.driver.set_color(id, lss_driver::LedColor::Off).await?;
        }
        self.driver.limp(self.base_motor_id).await?;
        Ok(())
    }

    pub async fn limp_neck(&mut self) -> Result<()> {
        self.driver.limp(self.neck_motor_id).await?;
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
