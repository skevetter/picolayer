# Picolayer

A utility tool that helps keep container layers as small as possible by automatically cleaning up installation leftovers such as apt-get update lists, caches, and temporary files. Picolayer can run any programming language, devcontainer-feature, or install any GitHub release with ease.

This project is inspired by [nanolayer](https://github.com/devcontainers-extra/nanolayer).

## Commands

| Command      | Description                                                                |
|--------------|----------------------------------------------------------------------------|
| `apt-get`    | Install Debian/Ubuntu packages                                             |
| `apk`        | Install Alpine packages                                                    |
| `brew`       | Install packages using Homebrew                                            |
| `npm`        | Install npm packages (installs Node.js if needed)                          |
| `pipx`       | Install Python packages in isolated environments (installs pipx if needed) |
| `gh-release` | Install binaries from GitHub releases                                      |
| `pkgx`       | Execute commands with pkgx                                                 |

## Installation

### From source

```bash
cargo install --git https://github.com/skevetter/picolayer
```

### From binary

Download the latest release from the [releases page](https://github.com/skevetter/picolayer/releases).

_Or download as a one-liner_

```bash
curl -L https://github.com/skevetter/picolayer/releases/latest/download/picolayer-x86_64-unknown-linux-gnu.tar.gz | tar -xz && chmod +x picolayer && \
./picolayer \
    devcontainer-feature \
    "ghcr.io/devcontainers-extra/features/bash-command:1" \
    --option command="curl https://pkgx.sh | sh"
```

## Usage

| Package Manager                                                | Command                                                             |
|----------------------------------------------------------------|---------------------------------------------------------------------|
| [Apt-get](https://wiki.debian.org/apt-get)                     | `picolayer apt-get cowsay`                                          |
| [Apt](https://wiki.debian.org/Apt)                             | `picolayer apt cowsay`                                              |
| [Aptitude](https://wiki.debian.org/Aptitude)                   | `picolayer aptitude cowsay`                                         |
| [Apk](https://wiki.alpinelinux.org/wiki/Alpine_Package_Keeper) | `picolayer apk cowsay`                                              |
| [Homebrew](https://brew.sh/)                                   | `picolayer brew cowsay`                                             |
| [Npm](https://nodejs.org/)                                     | `picolayer npm cowsay`                                              |
| [Pipx](https://pipx.pypa.io/)                                  | `picolayer pipx cowsay`                                             |
| GitHub releases                                                | `picolayer gh-release --owner pkgxdev --repo pkgx --version latest` |
| [Pkgx](https://docs.pkgx.sh/)                                  | `picolayer pkgx --tool python -- -c "print('Hello World')"`         |
