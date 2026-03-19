use std::process::Command;

fn main() {
    // 优先从环境变量获取 Git commit hash（用于 Docker 构建）
    if let Ok(git_hash) = std::env::var("GIT_COMMIT_HASH") {
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash);
        return;
    }
    
    // 否则尝试从 git 命令获取（本地构建）
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
