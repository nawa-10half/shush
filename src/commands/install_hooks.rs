use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
# kagienv pre-push hook — scans for hardcoded secrets before push
if command -v kagienv >/dev/null 2>&1; then
    kagienv scan
    if [ $? -ne 0 ]; then
        echo ""
        echo "Push blocked by kagienv. Fix the issues above before pushing."
        exit 1
    fi
fi
"#;

const KAGIENV_MARKER: &str = "kagienv scan";

pub fn execute() -> anyhow::Result<()> {
    let git_dir = find_git_dir()?;
    install_pre_push_hook(&git_dir)?;
    install_claude_hooks()?;
    println!("\nDone! Hooks installed successfully.");
    Ok(())
}

fn find_git_dir() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let git_dir = dir.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
        if !dir.pop() {
            anyhow::bail!("Not a git repository (no .git directory found)");
        }
    }
}

fn install_pre_push_hook(git_dir: &Path) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks/")?;
    }

    let hook_path = hooks_dir.join("pre-push");

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)?;
        if existing.contains(KAGIENV_MARKER) {
            println!("[git] pre-push hook already contains kagienv scan. Skipping.");
            return Ok(());
        }
        // Append to existing hook
        let updated = format!("{}\n{}", existing.trim_end(), PRE_PUSH_HOOK);
        fs::write(&hook_path, updated)?;
        println!("[git] Appended kagienv scan to existing pre-push hook.");
    } else {
        fs::write(&hook_path, PRE_PUSH_HOOK)?;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;
        println!("[git] Installed pre-push hook.");
    }

    Ok(())
}

fn install_claude_hooks() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let claude_dir = cwd.join(".claude");

    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir).context("Failed to create .claude/")?;
    }

    let settings_path = claude_dir.join("settings.local.json");

    if settings_path.exists() {
        let existing = fs::read_to_string(&settings_path)?;
        if existing.contains(KAGIENV_MARKER) {
            println!("[claude] Claude hooks already contain kagienv scan. Skipping.");
            return Ok(());
        }
        println!(
            "[claude] {} already exists. Add the following manually:\n\n\
             {}\n",
            settings_path.display(),
            claude_hooks_json()
        );
    } else {
        fs::write(&settings_path, claude_hooks_json())?;
        println!("[claude] Installed Claude Code hooks.");
    }

    Ok(())
}

fn claude_hooks_json() -> &'static str {
    r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "kagienv scan"
          }
        ]
      }
    ]
  }
}"#
}
