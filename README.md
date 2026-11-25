# Shrieker

Guitar feedback that blooms, chirps, squabbles and jumps harmonics.

<img src="docs/shrieker.png" width="400" alt="Shrieker">

A VST3 and CLAP plugin for macOS and Windows.

## Installing

Download the zip from [Releases](https://github.com/starzonmyarmz/shrieker/releases/latest) and follow `INSTALL.txt` inside it. The plugin isn't signed or notarized, so macOS may need `xattr -dr com.apple.quarantine` on it first; `INSTALL.txt` has the exact command.

## Tests

`cargo test` runs the engine offline, in host-sized blocks, against stored golden renders and synthetic checks. After an intended change to the sound, rerun with `BLESS=1` to rewrite the renders, and listen to them before committing.

## Develop

```sh
cargo test                                  # engine tests and golden renders, offline
BLESS=1 cargo test                          # rewrite golden renders and ui/mock.json after an intended change
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo xtask bundle shrieker --release     # target/bundled/Shrieker.{vst3,clap}
scripts/install.sh                          # bundle, sign ad hoc and install on macOS as X.Y.Z-dev+<sha>
scripts/clap-validator.sh "target/bundled/Shrieker.clap"
scripts/screenshot.sh                       # retake docs/shrieker.png
scripts/release --dry-run                   # what the next release would be
```

Work on the editor in a browser against the mock: `cd ui && python3 -m http.server 8765`, then open http://127.0.0.1:8765 (add `?version=1.2.0` to show a version). To load the page live in the plugin, set `SHRIEKER_UI_URL` to that URL before starting the host.

## Releases

Versions follow [SemVer](https://semver.org), and the commits since the last release decide the bump:

| Commits since the last release | Bump |
|---|---|
| a `!` or `BREAKING CHANGE:` footer | major (minor before 1.0) |
| `feat` | minor |
| `fix`, `perf`, `revert` | patch |
| anything else | no release |

`scripts/release` shows the computed bump and the changelog, then commits, tags `vX.Y.Z` and pushes. Pass `patch`, `minor`, `major` or an exact version to override, and `--dry-run` to only look. The tag runs `.github/workflows/release.yml`.

See [CHANGELOG.md](CHANGELOG.md) for what changed in each version.

## License

[MIT](LICENSE)
