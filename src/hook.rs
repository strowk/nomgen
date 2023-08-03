use std::fs;
use std::path::{Path, PathBuf};

use crate::command::HookCmd;

pub(crate) fn generate_pre_commit_hook(
    target_dir: Option<PathBuf>,
    target_name: String,
    command: HookCmd,
) -> eyre::Result<()> {
    // Detect the target directory
    let target_dir = if let Some(dir) = target_dir {
        dir
    } else {
        let mut current_dir = Path::new(".").canonicalize().unwrap();
        loop {
            if current_dir.join(".git").is_dir() {
                break current_dir.join(".git").join("hooks");
            }
            if !current_dir.pop() {
                panic!("Failed to locate .git directory.");
            }
        }
    };

    let hook_path = target_dir.join(target_name);

    // When commit calls hook, generate command should not commit,
    // but only stage the changes to avoid loosing original commit message
    let only_stage = command == HookCmd::Generate;

    let only_stage = if only_stage { " --only-stage true" } else { "" };

    let command = format!("nomgen {command}{only_stage}");

    let hook_content = format!(
        r#"#!/bin/sh
# Nomgen Pre-Commit Hook
#
# This hook runs the 'check' command before each commit to ensure that no undesired changes are present.
# If the commit message contains "nomgen generation", the check will be skipped.

# Check if the commit message contains "nomgen generation"
commit_message=$(git log --format=%B -n 1 HEAD)
if echo "$commit_message" | grep -q "nomgen generation"; then
    echo "Commit message contains 'nomgen generation', skipping nomgen runs."
    exit 0
fi

# Run the nomgen command checking for undesired changes
{command}

# If the check command failed, prevent the commit
if [ $? -ne 0 ]; then
    echo "Pre-commit hook failed: check command detected undesired changes. Aborting commit."
    exit 1
fi

exit 0
    
"#
    );

    // Write the hook content to the pre-commit hook file
    if let Err(err) = fs::write(&hook_path, hook_content) {
        log::error!("Error writing pre-commit hook: {}", err);
        std::process::exit(1);
    } else {
        // Set executable permissions on Unix-like systems (Linux and macOS)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&hook_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions)?;
        }

        println!("Pre-commit hook generated successfully in {:?}", hook_path);
    };

    Ok(())
}
