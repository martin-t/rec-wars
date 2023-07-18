# Release checklist

- `git pull`
- Look for fixmes
- Pick one:
  - `cargo build --target wasm32-unknown-unknown --release                            && cp -f target/wasm32-unknown-unknown/release/rec-wars.wasm rec-wars.wasm`
  - `cargo build --target wasm32-unknown-unknown --release --features web_splitscreen && cp -f target/wasm32-unknown-unknown/release/rec-wars.wasm rec-wars.wasm`
- Test locally
  - `python3 -m http.server`
  - Open http://localhost:8000/macroquad.html
- Publish on GitLab: `cd ~/dev/gitlab-pages && ./copy-rec-wars.sh`
- Wait for CI to finish
- Test [on GitLab](https://martin-t.gitlab.io/gitlab-pages/rec-wars/macroquad.html)
  - Check version number
