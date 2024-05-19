use tracing::info;

pub mod configuration;
pub mod error;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub async fn turn_on_display() -> anyhow::Result<()> {
    // WAYLAND_DISPLAY="wayland-1" wlr-randr --output HDMI-A-1 --on --transform 90
    let status = tokio::process::Command::new("wlr-randr")
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
    // WAYLAND_DISPLAY="wayland-1" wlr-randr --output HDMI-A-1 --on --transform 90
    let status = tokio::process::Command::new("wlr-randr")
        .env("WAYLAND_DISPLAY", "wayland-1")
        .arg("--output")
        .arg("HDMI-A-1")
        .arg("--off")
        .status()
        .await?;
    info!("Turning off display {:?}", status);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
