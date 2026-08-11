# Development Environments

The project supports several development environments. None of the optional
tools below is a requirement for contributors who already have a suitable Rust
toolchain.

## Native development

Native Windows is required for meaningful validation of administrator
elevation, Windows input injection, MIDI hot-plug behavior, and physical
Launchpad integration. Install the Rust version recorded in `mise.toml` using
rustup, mise, or another toolchain manager.

## mise

The repository includes `mise.toml` so a machine without a preconfigured Rust
environment can install the pinned toolchain and expose common tasks:

```sh
mise install
mise run fmt
mise run check
mise run test
mise run verify
```

Using mise is optional. Equivalent direct `cargo` commands remain supported.
The Rust version is pinned rather than set to `latest` so local development and
CI can remain reproducible.

## DIM

[dev-infra-manager (DIM)](https://github.com/slop-lab/dev-infra-manager)
provides optional persistent isolated workspaces. DIM itself is Linux-host
only. It is useful for documentation work, compilation, static checks, and
platform-independent unit tests, but it cannot replace native Windows or real
RISE hardware verification.

This repository intentionally uses only the minimal DIM project contract:
`.dim/entrypoint.sh`. It does not start project services or require a DIM
Compose stack. After registering this repository as a DIM project root, common
commands are:

```sh
dim create piu-rise-diy-controller piu-rise-dev
dim run piu-rise-dev bootstrap
dim run piu-rise-dev doctor
dim run piu-rise-dev shell
dim run piu-rise-dev fmt
dim run piu-rise-dev check
dim run piu-rise-dev test
dim run piu-rise-dev verify
```

The exact DIM CLI version and host backend are managed by the DIM installation,
not this repository. DIM is pre-stable software; pin and review the version at
the host level. The `bootstrap` task installs tools declared by `mise.toml` and
therefore requires mise to be available inside the workspace. Other tasks use
mise when available and otherwise use tools already on `PATH`.

## Verification boundaries

Passing tests in mise or DIM is software verification only. It does not change
a device or feature to `Bench verified` or `RISE verified`. See the
[verification strategy](testing/verification.md) for the required evidence.
