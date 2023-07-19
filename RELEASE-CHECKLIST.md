# Release checklist

- `git pull`
- Look for fixmes
- Look for outdated deps (`cargo outdated` or [deps.rs](https://deps.rs/repo/github/martin-t/cvars))
- `cargo update`
- Bump version
- Update CHANGELOG.md
- Commit, `git push`, make sure CI passes
- `git tag -a vX.Y.Z`
- `cargo build --target wasm32-unknown-unknown --release                            && cp -f target/wasm32-unknown-unknown/release/rec-wars.wasm rec-wars.wasm`
- `cargo build --target wasm32-unknown-unknown --release --features web_splitscreen && cp -f target/wasm32-unknown-unknown/release/rec-wars.wasm rec-wars-splitscreen.wasm`
- Publish on GitLab: `cd ~/dev/gitlab-pages && ./copy-rec-wars.sh`
- Wait for CI to finish
- Test [on GitLab](https://martin-t.gitlab.io/gitlab-pages/rec-wars/macroquad.html)
  - Check version number
- `cargo publish`
- `git push` the tag
- GitHub release
  - Copy relevant part of CHANGELOG.md to description
