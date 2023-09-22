use argh::FromArgs;
use core::fmt;
use eyre::Result;
use std::{path::PathBuf, str::FromStr};

use crate::{config, hook::generate_pre_commit_hook};

#[derive(Debug, FromArgs)]
/// CLI tool for nomgen.
pub(crate) struct CliArgs {
    #[argh(subcommand)]
    pub(crate) subcommand: Option<Subcommand>,

    /// print version information
    #[argh(switch, short = 'v')]
    pub(crate) version: bool,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub(crate) enum Subcommand {
    Generate(GenerateArgs),
    Check(CheckArgs),
    Hook(HookArgs),
}

#[derive(Debug, FromArgs)]
/// Generate would by default firstly check if there are any changes
/// to protected files (see "check" command), and if there are none,
/// it would run commands of all generators specified in the configuration file.
/// Finally it would commit the changes to the repository.
#[argh(subcommand, name = "generate")]
pub(crate) struct GenerateArgs {
    #[argh(option, short = 'c')]
    /// path to the configuration file
    config: Option<PathBuf>,
    #[argh(switch, short = 'f')]
    /// force overwrite of existing files, by ignoring check result
    force: bool,
    #[argh(switch, short = 'o')]
    /// only stage the changes, do not commit, useful if generate is called from a git hook
    only_stage: bool,
    #[argh(switch, short = 's')]
    /// skip the check step
    skip_check: bool,
}

#[derive(Debug, FromArgs)]
/// Check if there were not modifications to protected files.
/// Every generator can specify a list of patterns that match generated
/// files. If any of the files matching any of the patterns is modified
/// when this command is run, the command will fail. This command is
/// intended to be used in git hook to prevent committing manual changes to
/// generated files.
#[argh(subcommand, name = "check")]
pub(crate) struct CheckArgs {
    #[argh(option, short = 'c')]
    /// path to the configuration file
    config: Option<PathBuf>,
}

#[derive(Debug, FromArgs)]
/// Command generating hook for git that would execute nomgen.
/// If hook-dir not specified, the hook will be installed under closest .git directory
/// found in current or any parent directory, under "hooks" subfolder.
/// So in case if .git found in current directory, the hook will be installed
/// under ".git/hooks" directory.
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
                let (config, config_path) = config::read_config(&args.config)?;
                crate::generate::run_generate(
                    config,
                    config_path,
                    args.force,
                    args.only_stage,
                    args.skip_check,
                )?;
            }
            Subcommand::Check(args) => {
                let (config, _) = config::read_config(&args.config)?;
                crate::check::run_check_cli(config)?;
            }
            Subcommand::Hook(args) => {
                generate_pre_commit_hook(&args.hook_dir, &args.hook_name, &args.hook_cmd)?;
            }
        }

        Ok(())
    }
}
