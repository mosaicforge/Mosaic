use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|hash| hash.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let git_tag = Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|desc| desc.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo::rustc-env=GIT_COMMIT={}", git_hash);
    println!("cargo::rustc-env=GIT_TAG={}", git_tag);

    // Always rerun if any git changes occur
    println!("cargo::rerun-if-changed=.git/HEAD");
    println!("cargo::rerun-if-changed=.git/refs/");
}
