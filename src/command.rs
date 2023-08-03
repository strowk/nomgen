use argh::FromArgs;
use core::fmt;
use eyre::Result;
use std::{path::PathBuf, str::FromStr};

use crate::{config, hook::generate_pre_commit_hook};

#[derive(Debug, FromArgs)]
/// CLI tool for nomgen.
pub(crate) struct CliArgs {
    #[argh(subcommand)]
    pub(crate) subcommand: Subcommand,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub(crate) enum Subcommand {
    Generate(GenerateArgs),
    Check(CheckArgs),
    Hook(HookArgs),
}

#[derive(Debug, FromArgs)]
/// Generate command arguments.
#[argh(subcommand, name = "generate")]
pub(crate) struct GenerateArgs {
    #[argh(option, short = 'c')]
    /// path to the configuration file
    config: Option<PathBuf>,
    #[argh(option, short = 'f', default = "false")]
    /// force overwrite of existing files, by ignoring check result
    force: bool,
    #[argh(option, short = 'o', default = "false")]
    /// only stage the changes, do not commit, useful if generate is called from a git hook
    only_stage: bool,
}

#[derive(Debug, FromArgs)]
/// Check command arguments.
#[argh(subcommand, name = "check")]
pub(crate) struct CheckArgs {
    #[argh(option, short = 'c')]
    /// path to the configuration file
    config: Option<PathBuf>,
}

#[derive(Debug, FromArgs)]
/// Hook command arguments.
#[argh(subcommand, name = "hook")]
pub(crate) struct HookArgs {
    #[argh(option, short = 'd')]
    /// path to the directory for the git hook
    hook_dir: Option<PathBuf>,
    #[argh(option, short = 'h', default = "String::from(\"pre-commit\")")]
    /// fine name for the git hook
    hook_name: String,
    #[argh(option, short = 'c', default = "HookCmd::Check")]
    /// which command to run in the hook
    hook_cmd: HookCmd,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HookCmd {
    Generate,
    Check,
}

impl FromStr for HookCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "generate" => Ok(HookCmd::Generate),
            "check" => Ok(HookCmd::Check),
            _ => Err(format!("Unknown hook command: {}", s)),
        }
    }
}

impl fmt::Display for HookCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookCmd::Generate => write!(f, "generate"),
            HookCmd::Check => write!(f, "check"),
        }
    }
}

impl Subcommand {
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            Subcommand::Generate(args) => {
                let (config, config_path) = config::read_config(args.config.clone())?;
                crate::generate::run_generate(config, config_path, args.force, args.only_stage)?;
            }
            Subcommand::Check(args) => {
                let (config, _) = config::read_config(args.config.clone())?;
                crate::check::run_check_cli(config)?;
            }
            Subcommand::Hook(args) => {
                generate_pre_commit_hook(
                    args.hook_dir.clone(),
                    args.hook_name.clone(),
                    args.hook_cmd.clone(),
                )?;
            }
        }

        Ok(())
    }
}
