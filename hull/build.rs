use std::process::Command;

fn main() {
    // 获取 Git commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok();
    
    if let Some(output) = output {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash);
        }
    }
    
    // 监听 Cargo.toml 和 build.rs 变更
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}
