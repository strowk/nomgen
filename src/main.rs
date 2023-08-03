use eyre::Result;

mod check;
mod command;
mod config;
mod generate;
mod hook;

fn main() -> Result<()> {
    env_logger::init();

    let args: command::CliArgs = argh::from_env();

    if args.version {
        println!("nomgen {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    args.subcommand
        .ok_or_else(|| {
            eyre::format_err!(
                r#"No subcommand specified. Use "nomgen --help" for more information."#
            )
        })?
        .run()?;

    Ok(())
}
