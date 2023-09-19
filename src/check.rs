use crate::config::Config;
use eyre::{eyre, Result};
use git2::{Repository, Status, StatusOptions};
use glob::Pattern;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub enum CheckResult {
    UndesiredChangesDetected,
    NoChangesDetected,
}

pub fn run_check(config: Config) -> Result<CheckResult> {
    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(_) => {
            log::error!("No Git repository found in the current directory.");
            return Err(eyre!("No Git repository found in the current directory."));
        }
    };

    let mut status_options = StatusOptions::new();
    status_options.include_untracked(true); // Include untracked files

    let staged_paths: HashSet<PathBuf> = repo
        .statuses(Some(&mut status_options))?
        .iter()
        .filter(|entry| {
            entry.status().intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            )
        })
        .filter_map(|entry| entry.path().map(|p| Path::new(p).to_owned()))
        .collect();

    let unstaged_paths: HashSet<PathBuf> = repo
        .statuses(Some(&mut status_options))?
        .iter()
        .filter(|entry| {
            entry.status().intersects(
                Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            )
        })
        .filter_map(|entry| entry.path().map(|p| Path::new(p).to_owned()))
        .collect();

    let mut changes_detected = false;

    log::debug!("Nomgen: staged changes: {:?}", staged_paths);
    log::debug!("Nomgen: unstaged changes: {:?}", unstaged_paths);

    for generator in config.generators {
        if let Some(pattern) = generator.patterns {
            for pattern_str in pattern.iter() {
                log::info!("Nomgen: checking for changes in: {}", pattern_str);
                let pattern = Pattern::new(&pattern_str)?;
                let glob_path = PathBuf::from(pattern.as_str());
                let relative_paths = glob::glob(glob_path.to_str().unwrap())?
                    .filter_map(|entry| entry.ok())
                    .collect::<Vec<_>>();
    
                for path in relative_paths {
                    log::debug!(
                        "Nomgen: found modification in path: {}",
                        path.to_str()
                            .ok_or_else(|| eyre::eyre!("Failed to convert path to string."))?
                    );
                    let stripped_path = if let Ok(stripped) = path.strip_prefix("./") {
                        stripped
                    } else {
                        &path
                    };
    
                    if staged_paths.contains(stripped_path) || unstaged_paths.contains(stripped_path) {
                        log::error!("Changes detected in: {:?}", stripped_path.display());
                        changes_detected = true;
                    }
                }
            }
        }
    }

    if changes_detected {
        Ok(CheckResult::UndesiredChangesDetected)
    } else {
        Ok(CheckResult::NoChangesDetected)
    }
}

pub fn run_check_cli(config: Config) -> eyre::Result<()> {
    match run_check(config) {
        Ok(CheckResult::UndesiredChangesDetected) => {
            std::process::exit(1);
        }
        Ok(CheckResult::NoChangesDetected) => Ok(()),
        Err(err) => Err(err),
    }
}
