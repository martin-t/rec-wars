use std::{env, fs, path::Path, process::Command};

fn main() {
    let describe = Command::new("git")
        .args(&["describe", "--long", "--always", "--dirty"])
        .output()
        .unwrap()
        .stdout;
    let describe = String::from_utf8(describe).unwrap();
    let describe = describe.trim_end_matches('\n');

    let log = Command::new("git")
        .args(&[
            "log",
            "-n",
            "1",
            "--pretty=format:%cd %s",
            "--date=format:%Y-%m-%d %H:%M",
        ])
        .output()
        .unwrap()
        .stdout;
    let log = String::from_utf8(log).unwrap();

    let git_version = format!("{} {}", describe, log);

    let path = Path::new(&env::var("OUT_DIR").expect("failed to get OUT_DIR")).join("git_version");
    fs::write(path, git_version).expect("Failed to write git_version");
}
