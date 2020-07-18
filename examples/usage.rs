use color_anyhow::{anyhow::Context, anyhow::Error, Section};
use tracing::{info, instrument};

#[instrument]
fn main() -> Result<(), Error> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    color_anyhow::install()?;

    Ok(read_config()?)
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
fn read_file(path: &str) -> Result<(), Error> {
    info!("Reading file");
    Ok(std::fs::read_to_string(path).map(drop)?)
}

#[instrument]
fn read_config() -> Result<(), Error> {
    read_file("fake_file")
        .context("Unable to read config")
        .suggestion("try using a file that exists next time")
}
