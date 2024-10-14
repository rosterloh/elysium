use clap::Parser;

/// Argument parser powered by [`clap`].
#[derive(Clone, Debug, Default, Parser)]
#[clap(
    version,
    author = clap::crate_authors!("\n"),
    about,
    rename_all_env = "screaming-snake",
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading}
  {usage}

{all-args}{after-help}
",
)]
pub struct Args {
    /// AWS profile to use.
    #[arg(env, short = 'p', long = "profile", default_value = "iotmgmt_prod")]
    pub profile: String,

    /// AWS region to use.
    #[arg(env, short = 'r', long = "region", default_value = "eu-west-1")]
    pub region: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    #[test]
    fn test_args() {
        Args::command().debug_assert();
    }
}