use clap::Parser;

use crate::utils::{get_config_dir, get_data_dir};

#[derive(Debug, Parser)]
#[clap(author, version = version(), about)]
pub struct Args {
    /// AWS profile to use.
    #[arg(
        short = 'p',
        long = "profile",
        help = "Name of the profile configured in your local AWS config file",
        default_value = "iotmgmt_prod"
    )]
    pub profile: String,

    /// AWS region to use.
    #[arg(
        short = 'r',
        long = "region",
        help = "AWS region to use",
        default_value = "eu-west-1"
    )]
    pub region: String,

    /// Increase verbosity. Can be used multiple times
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}