use std::path::PathBuf;
use std::process::Command;

pub fn get_player_path(player_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(player_name);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

pub fn compile_player(name: &str) {
    let status = Command::new("cargo")
        .args(["build", "--bin", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("Failed to execute cargo build");

    assert!(status.success(), "Failed to compile {}", name);
}
