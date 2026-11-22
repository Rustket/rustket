# Rustket

Rustket is a small Rust command-line package manager for macOS and Linux. It
installs prebuilt programs from GitHub taps, verifies each download with
SHA-256, and makes installed commands available through `~/.rustket/bin`.

## Install the Rustket command

Install the matching prebuilt Rustket binary with one command:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Rustket/rustket/main/install.sh)" && export PATH="$HOME/.rustket/bin:$PATH"
```

The installer selects your macOS or Linux architecture, verifies the archive
against the release SHA-256 checksums, and installs Rustket in
`~/.rustket/bin`. It also preconfigures the official
[`Rustket/rustket-bits`](https://github.com/Rustket/rustket-bits) tap and adds
the bin directory to the configuration for Bash, Zsh, or Fish. The `&& export`
updates the current Bash or Zsh shell; Fish users can open a new shell.

Prebuilt binaries are also available on the
[0.1.0 release page](https://github.com/Rustket/rustket/releases/tag/0.1.0).

## Install with Homebrew

Install the Rustket CLI from the official Homebrew tap:

```sh
brew install Rustket/rustket/rustket
```

To build from source instead, install Rust and Cargo, then run:

```sh
cargo install --git https://github.com/Rustket/rustket.git
```

Rustket automatically adds the official `Rustket/rustket-bits` tap the first
time you list or refresh taps, or run a command that needs package definitions.
This requires network access if the tap isn't already present; you don't need
to run `rustket tap add` for the official tap.

## macOS verification warning

The 0.1.0 prebuilt macOS binaries are not code-signed or notarized. If Gatekeeper
says Rustket "could not be verified," use the source-build instructions above
instead of bypassing the warning. For prebuilt downloads to pass Gatekeeper,
future macOS releases must be signed with an Apple Developer ID and notarized.
SHA-256 verification checks the downloaded file's integrity; it does not
replace Apple's signature and notarization checks.

## Install an app

The official tap is added automatically when needed, so you can install Rustket
itself with:

```sh
rustket install rustket
```

The official tap also provides ripgrep:

```sh
rustket install ripgrep
```

Rustket exposes the upstream `rg` executable as `ripgrep` in
`~/.rustket/bin`.

For an app from another tap, add that tap and install the app's bit:

```sh
rustket tap add <owner>/<tap-repo>
rustket install <app-name>
```

Rustket currently installs command-line executables, not macOS `.app` bundles.
The installer adds `~/.rustket/bin` to your shell configuration. If you
installed Rustket from source, add it to your `PATH` to run installed apps from
any directory:

```sh
export PATH="$HOME/.rustket/bin:$PATH"
```

`~/.cargo/bin` contains the Rustket CLI installed with Cargo;
`~/.rustket/bin` contains apps managed by Rustket.

## Commands

```text
rustket install <bit> [--skip-verify]
rustket uninstall <bit>
rustket list
rustket search <query>
rustket info <bit>
rustket doctor
rustket update
rustket upgrade

rustket tap add <owner/repo>
rustket tap list
rustket tap remove <owner/repo>
rustket tap update
```

- `install` downloads the current platform's archive, verifies its SHA-256,
  unpacks it, and links its executable into `~/.rustket/bin`.
- `--skip-verify` skips checksum verification and is intended for development
  only.
- `list` shows commands managed by Rustket. `search` matches bit names from
  configured taps; `info` shows a bit's version and available platforms.
- `doctor` checks the Rustket directories, `PATH`, Git, and broken links.
- `tap update` pulls fast-forward updates for all configured taps.
- `update` refreshes taps and reinstalls the `rustket` bit itself.
- `upgrade` refreshes taps, then reinstalls every bit currently listed in
  `~/.rustket/bin`. It attempts the remaining bits if one upgrade fails and
  reports a nonzero error if any failed.

Run `rustket --help` or `rustket <command> --help` for command-line help.

## Taps and bit definitions

A tap is a GitHub repository containing package definitions in its `bits/`
directory. Add a tap using `owner/repo` or its GitHub HTTPS URL. Rustket stores
cloned taps under `~/.rustket/taps/<owner>/<repo>`.

Each bit is a TOML file named `<name>.toml` directly inside a tap's `bits/`
directory. For example:

```toml
name = "hello"
version = "1.0.0"

[platform."linux-x86_64"]
url = "https://github.com/example/hello/releases/download/1.0.0/hello.tar.gz"
sha256 = "replace-with-the-64-character-sha256-digest"
bin = "hello"
```

Add one `[platform."<os>-<architecture>"]` section for each supported build.
The `url` must point to a `.tar.gz` archive, `sha256` must be its actual
64-character SHA-256 digest, and `bin` is the executable's path inside the
unpacked archive. Platform keys use Rust's OS and architecture names, such as
`macos-aarch64`, `macos-x86_64`, `linux-aarch64`, and `linux-x86_64`.

The archive must contain the executable at the path specified by `bin`.
Rustket needs Git to manage taps, `tar` to unpack archives, and `shasum` or
`sha256sum` to verify downloads.

Installed files are stored under `~/.rustket/cellar/<name>/<version>`, with
command links in `~/.rustket/bin`.

## Development

```sh
cargo fmt --check
cargo test
cargo check
cargo clippy
```

The release workflow builds macOS and Linux archives for ARM64 and x86_64 and
generates their checksums and a Rustket bit manifest.
