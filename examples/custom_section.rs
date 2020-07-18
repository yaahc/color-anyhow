use color_anyhow::{
    anyhow::Error,
    anyhow::{anyhow, Context},
    Section, SectionExt,
};
use std::process::Command;
use tracing::instrument;

trait Output {
    fn output2(&mut self) -> Result<String, Error>;
}

impl Output for Command {
    #[instrument]
    fn output2(&mut self) -> Result<String, Error> {
        let output = self.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("cmd exited with non-zero status code"))
                .with_section(move || stdout.trim().to_string().header("Stdout:"))
                .with_section(move || stderr.trim().to_string().header("Stderr:"))
        } else {
            Ok(stdout.into())
        }
    }
}

#[instrument]
fn main() -> Result<(), Error> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();
    color_anyhow::install()?;

    Ok(read_config().map(drop)?)
}

#[cfg(feature = "capture-spantrace")]
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[instrument]
fn read_file(path: &str) -> Result<String, Error> {
    Command::new("cat").arg(path).output2()
}

#[instrument]
fn read_config() -> Result<String, Error> {
    read_file("fake_file")
        .context("Unable to read config")
        .suggestion("try using a file that exists next time")
}
