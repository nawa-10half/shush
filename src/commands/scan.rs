use std::fs;
use std::path::Path;
use std::process;

use anyhow::Result;

use crate::vault::Vault;

const SKIP_DIRS: &[&str] = &[".git", ".kagienv", "target", "node_modules", ".next", "__pycache__"];

struct Finding {
    file: String,
    line_number: usize,
    secret_name: String,
}

pub fn execute() -> Result<()> {
    let vault = Vault::open()?;
    let secrets = vault.get_all()?;

    if secrets.is_empty() {
        println!("No secrets in vault. Nothing to scan for.");
        return Ok(());
    }

    // Filter out very short values to avoid false positives
    let secrets: Vec<_> = secrets
        .into_iter()
        .filter(|(name, value)| {
            if value.len() < 4 {
                eprintln!(
                    "Warning: skipping '{}' (value too short for reliable scanning)",
                    name
                );
                false
            } else {
                true
            }
        })
        .collect();

    let cwd = std::env::current_dir()?;
    let mut findings = Vec::new();

    scan_dir(&cwd, &cwd, &secrets, &mut findings)?;

    if findings.is_empty() {
        println!("No hardcoded secrets detected.");
    } else {
        eprintln!("\n\u{26a0}  kagienv: {} secret(s) detected!\n", findings.len());
        for f in &findings {
            eprintln!("    File: {}:{}", f.file, f.line_number);
            eprintln!("    Key:  {} (matches vault value)\n", f.secret_name);
        }
        eprintln!("    Run `kagienv run <command>` to inject secrets safely.");
        process::exit(1);
    }

    Ok(())
}

fn scan_dir(
    dir: &Path,
    root: &Path,
    secrets: &[(String, String)],
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') && name != "." {
                if SKIP_DIRS.contains(&name) {
                    continue;
                }
                // Skip other hidden files/dirs
                continue;
            }
            if SKIP_DIRS.contains(&name) {
                continue;
            }
        }

        if path.is_dir() {
            scan_dir(&path, root, secrets, findings)?;
        } else if path.is_file() {
            scan_file(&path, root, secrets, findings)?;
        }
    }

    Ok(())
}

fn scan_file(
    path: &Path,
    root: &Path,
    secrets: &[(String, String)],
    findings: &mut Vec<Finding>,
) -> Result<()> {
    // Read first bytes to detect binary files
    let content = match fs::read(path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    // Skip binary files (check for null bytes in first 8KB)
    let check_len = content.len().min(8192);
    if content[..check_len].contains(&0) {
        return Ok(());
    }

    let text = match std::str::from_utf8(&content) {
        Ok(t) => t,
        Err(_) => return Ok(()),
    };

    let relative = path.strip_prefix(root).unwrap_or(path);
    let relative_str = relative.to_string_lossy();

    for (line_number, line) in text.lines().enumerate() {
        for (name, value) in secrets {
            if line.contains(value.as_str()) {
                findings.push(Finding {
                    file: relative_str.to_string(),
                    line_number: line_number + 1,
                    secret_name: name.clone(),
                });
            }
        }
    }

    Ok(())
}
