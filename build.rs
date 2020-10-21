use std::process::Command;

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
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
}
