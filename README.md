## Envelope

![A terminal with a vertically split list of environment variables and path components](./assets/envelop.png)

Envelope is a terminal UI for inspecting environment variables, searching them, creating or editing shell-persisted variables, and viewing PATH entries.

Envelope writes changes to your detected shell config file so future shell sessions can use them. For bash-like shells this is usually `~/.bashrc`, `~/.zshrc`, `~/.bash_profile`, or `~/.profile`. For fish, Envelope writes simple `set -gx` assignments to `~/.config/fish/config.fish`.

## Install

With Cargo:

```sh
cargo install envelope
```

From GitHub releases, download the binary for your platform, place it somewhere on your PATH, and run:

```sh
envelope
```

## Controls

| Key | Action |
| --- | --- |
| `Tab` | Switch between environment variables and PATH entries |
| `/` | Search names, values, and PATH entries |
| `n` | Create a new environment variable |
| `e` | Edit the selected variable value or PATH entry |
| `Enter` | Save the current edit or advance the new-variable form |
| `Esc` | Cancel the active edit/search/modal |
| `q` | Quit |
| `Ctrl-C` | Quit |

## Persistence And Safety

Environment variable names must start with a letter or `_` and may contain only letters, numbers, and `_`.

When editing or creating variables, Envelope writes shell-quoted values and replaces existing matching assignments instead of appending duplicate stale exports.

PATH edits update the current TUI list and append a future-shell PATH entry to your shell config. Envelope does not currently rewrite complex existing PATH expressions.

## Development

Run the local quality gates:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --list
```

The GitHub release workflow builds Linux, macOS, and Windows binaries when a `v*` tag is pushed. Publish to crates.io after verifying package contents:

```sh
cargo package --list
cargo publish
```
