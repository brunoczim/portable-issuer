use std::{error::Error, io};

use clap::Parser;
use thiserror::Error;
use tokio::{net::TcpListener, signal};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::FromEnvError, EnvFilter};

#[derive(Debug, Error)]
enum LogSetupError {
    #[error("Failed to create environment filter")]
    EnvFilter(
        #[source]
        #[from]
        FromEnvError,
    ),
    #[error("Failed to initialize logging")]
    Init(#[source] Box<dyn Error + Send + Sync + 'static>),
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Failed to bind a TCP listener")]
    Bind(#[source] io::Error),
    #[error("Failed to serve app")]
    Serve(#[source] io::Error),
}

#[derive(Debug, Error)]
enum MainError {
    #[error("Failed to setup logging")]
    LogSetup(
        #[from]
        #[source]
        LogSetupError,
    ),
    #[error("Failed to run server")]
    App(
        #[from]
        #[source]
        AppError,
    ),
}

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short = 'b', long = "bind-addr")]
    bind_addr: String,
}

fn setup_logger() -> Result<(), LogSetupError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("PORTABLE_ISSUER_LOG")
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .with_writer(io::stderr)
        .try_init()
        .map_err(LogSetupError::Init)?;
    Ok(())
}

async fn run_server_app(bind_addr: &str) -> Result<(), AppError> {
    let app = portable_issuer::router();
    let listener =
        TcpListener::bind(bind_addr).await.map_err(AppError::Bind)?;
    tracing::info!(bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c();
        })
        .await
        .map_err(AppError::Serve)?;
    Ok(())
}

async fn try_main(cli: Cli) -> Result<(), MainError> {
    setup_logger()?;
    run_server_app(&cli.bind_addr).await?;
    Ok(())
}

fn print_fatal_error(error: MainError) {
    eprintln!("Server found a fatal error");
    let mut next = Some(&error as &dyn Error);
    while let Some(current) = next {
        eprintln!("caused by:");
        eprintln!("  {current}");
        next = current.source();
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = try_main(cli).await {
        print_fatal_error(error);
    }
}
