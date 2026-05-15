use std::path::Path;
use std::process::Command;

/// Returns the repository directory name from `git rev-parse --show-toplevel`.
pub fn get_repo_name() -> Result<String, std::io::Error> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "git rev-parse failed",
        ));
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let name = Path::new(&root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repository")
        .to_string();

    Ok(name)
}

pub fn get_log() -> Result<String, std::io::Error> {
    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%H|%ae|%ai|%s",
            "--reverse",
        ])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Returns file paths per commit (`--name-only`, empty pretty format).
pub fn get_log_with_files() -> Result<String, std::io::Error> {
    let output = Command::new("git")
        .args(["log", "--pretty=format:", "--name-only", "--reverse"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}