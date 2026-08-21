# Development Environments

The project supports several development environments. None of the optional
tools below is a requirement for contributors who already have a suitable Rust
toolchain.

## Isolated Tauri check

Do not install Tauri's Linux WebKitGTK, D-Bus, or indicator development
packages on the host solely to check this Windows application. Run
`just check-gui-container` instead. It builds the pinned Debian-based image in
`docker/tauri-linux.Dockerfile` and checks the independent `src-tauri` crate.
Docker image and BuildKit cache layers own all downloaded system and Rust
dependencies.

## Native development

Native Windows is required for meaningful validation of Windows integrity
levels, input injection, MIDI hot-plug behavior, and physical
Launchpad integration. Install the Rust version recorded in `mise.toml` using
rustup, mise, or another toolchain manager.

## just

The repository `justfile` is an optional development-only command index. Run
`just` to list the small supported recipe set:

```sh
just build
just release
just run                 # prints CLI help by default
just run doctor
just run list
just run monitor --input "Launchpad"
just test
just verify
```

`just run` uses Cargo's debug profile. The resulting application runs without
requesting UAC elevation. Use
`just run run --input "Launchpad" --dry-run` when exercising mappings without
injecting keyboard input.

Just is not installed with or required by a release. Every recipe is a short
wrapper around the corresponding Cargo or repository command, so direct tools
remain fully supported.

## mise

The repository includes `mise.toml` so a machine without a preconfigured Rust
environment can install the pinned toolchain and `just`, and expose common
tasks:

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

This remains a single-repository DIM project. Its reviewed `.dim/setup.sh`
starts a project-owned `agent` container and a separate privileged rootless
DinD service. The agent receives only the private DinD TCP endpoint, not the
host or trusted-workspace Docker socket. This lets the agent build the Tauri
check image without installing WebKitGTK or D-Bus packages on the host or in
the ordinary agent image.

The agent uses the project-owned `agent-home` volume at `/home/dim-agent` so
tool configuration and caches survive agent service recreation. Normal DIM
restart and workspace-container replacement preserve it through the outer
workspace Docker volume; `.dim/teardown.sh` removes it on workspace discard.

After registering this repository as a DIM project root, common commands are:

```sh
dim create piu-rise-diy-controller piu-rise-dev
dim run piu-rise-dev bootstrap
dim run piu-rise-dev doctor
dim run piu-rise-dev shell
dim run piu-rise-dev fmt
dim run piu-rise-dev check
dim run piu-rise-dev test
dim run piu-rise-dev verify
dim run piu-rise-dev check-gui-container
dim run piu-rise-dev codex
```

The exact DIM CLI version and host backend are managed by the DIM installation,
not this repository. DIM is pre-stable software; pin and review the version at
the host level. The agent image pins Rust 1.97, Node.js 24.6.0, just 1.42.4,
and Docker CLI/DinD 29.1.3.
`bootstrap` reports the preinstalled tool versions; it does not modify the
host. `check-gui-container` uses the agent's private Docker daemon.

The trusted workspace setup exposes only an ingress-restricted controller
proxy at `/run/dim/dev-controller/controller.sock`. The same runtime directory
is mounted read-only into the agent; the original workspace controller socket
and grant are not passed through. Setup discovers the host-approved ingresses
available to the workspace and allows all of them through the restricted
proxy; all other controller capabilities remain unavailable to the agent.

The setup rewrites the checkout's `origin` to the routable
`DIM_GIT_BASE_URL` supplied by DIM and passes that URL into the nested agent.
Nested services cannot resolve the workspace-root-only `dim-gitea` hostname.

## Verification boundaries

Passing tests in mise or DIM is software verification only. It does not change
a device or feature to `Bench verified` or `RISE verified`. See the
[verification strategy](testing/verification.md) for the required evidence.
