use eyre::Result;

mod check;
mod command;
mod config;
mod generate;
mod hook;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let args: command::CliArgs = argh::from_env();
    args.subcommand.run()?;

    Ok(())
}
