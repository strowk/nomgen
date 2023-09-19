use crate::check::{run_check, CheckResult};
use crate::config::Config;
use eyre::{eyre, Result};
use git2::{IndexAddOption, Repository};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn run_generate(
    config: Config,
    config_path: PathBuf,
    ignore_check_result: bool,
    only_stage: bool,
    skip_check: bool,
) -> Result<()> {
    let config_dir = config_path
        .parent()
        .ok_or_else(|| eyre!("Invalid config path"))?;

    if skip_check {
        println!("Skipping modifications check. Generating files...");
        generate(config, config_dir, only_stage)?;
        return Ok(());
    }

    match run_check(config.clone())? {
        CheckResult::UndesiredChangesDetected => {
            if !ignore_check_result {
                log::error!("Undesired changes detected. Aborting generation.");
                std::process::exit(1);
            } else {
                println!("Ignore check results. Generating files...");
                generate(config, config_dir, only_stage)?;
            }
        }
        CheckResult::NoChangesDetected => {
            println!("No undesired changes detected. Generating files...");
            generate(config, config_dir, only_stage)?;
        }
    }

    Ok(())
}

fn generate(config: Config, config_dir: &Path, only_stage: bool) -> Result<()> {
    // Run generator commands
    for generator in &config.generators {
        if let Some(command) = &generator.command {
            println!("Running generator command: {}", command);
            let mut command = Command::new(command);

            command.current_dir(config_dir);

            if let Some(args) = &generator.args {
                command.args(args);
            }

            let child = command.spawn()?;

            let output = child.wait_with_output()?;

            // let output = command.output()?;

            if output.status.success() {
                println!("Generator command executed successfully.");
            } else {
                log::error!(
                    "Generator command failed. Error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    thread::sleep(Duration::from_secs(2));

    // Commit resulting changes
    let repo = Repository::discover(".")?;
    let mut index = repo.index()?;
    for generator in &config.generators {
        if let Some(pattern_str) = &generator.patterns {
            for pattern_str in pattern_str.iter() {
                println!("Staging generated files matching pattern: {}", pattern_str);
                index.add_all([pattern_str].iter(), IndexAddOption::DEFAULT, None)?;
            }
        }
    }
    index.write()?;
    if only_stage {
        println!("Changes staged. Committing skipped.");
        return Ok(());
    } else {
        println!("Changes staged. Committing...");
        let oid = index.write_tree()?;
        let signature = repo.signature()?;
        let parent_commit = repo.head()?.peel_to_commit()?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "nomgen generated",
            &repo.find_tree(oid)?,
            &[&parent_commit],
        )?;
    }
    println!("Changes committed with 'nomgen generated' message.");
    Ok(())
}
