use clap::Parser;
use args::Args;
use color_eyre::Result;

use crate::{
    app::App,
    aws::AwsCloud,
    utils::{initialise_logging, initialise_panic_handler}
};

mod action;
mod app;
mod args;
mod aws;
mod components;
mod enums;
mod layout;
mod tui;
mod utils;

async fn tokio_main() -> Result<()> {
    initialise_panic_handler()?;
    initialise_logging()?;

    let args = Args::parse();
    
    let mut aws = AwsCloud::new(&args.profile, &args.region).await.unwrap();
    aws.load().await?;

    let mut app = App::new(aws)?;
    app.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}