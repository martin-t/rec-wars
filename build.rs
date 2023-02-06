use std::process::Command;

fn main() {
    // Ideally we'd save --dirty status but that often means
    // recompiling when non-code files in the repo changed.
    let describe = Command::new("git")
        .args(["describe", "--long", "--always"])
        .output()
        .unwrap()
        .stdout;
    let describe = String::from_utf8(describe).unwrap();
    let describe = describe.trim_end_matches('\n');
    // e.g. v0.1.0-109-g6a10529

    let log = Command::new("git")
        .args([
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
    // e.g. 2021-09-09 14:17 Bigger missiles

    let git_version = format!("{} {}", describe, log);
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
}
