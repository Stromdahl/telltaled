# Sandbox image for sandcastle-orchestrated agents working on telltaled.
#
# Derived from sandcastle's canonical Claude Code template (node:22-bookworm +
# Claude Code CLI), extended with the Rust toolchain and `just` so the agent can
# satisfy the repo's gate (`just check`). GitHub access is deliberately NOT
# installed here: all `gh` interaction (issue locking, PR creation) happens
# host-side in `.sandcastle/main.mts`, so the sandbox needs no GitHub token.
#
# Build with: npm run sandcastle:build-image   (== sandcastle docker build-image)
# Build context is the repo root; the worktree is bind-mounted at run time.

FROM node:22-bookworm

# System dependencies. build-essential + pkg-config cover native Rust crate
# builds; git/curl/jq are sandcastle's baseline. Installed as root before the
# USER switch below.
RUN apt-get update && apt-get install -y \
  git \
  curl \
  jq \
  ca-certificates \
  build-essential \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

# Build-args for UID/GID alignment: `sandcastle docker build-image` defaults
# these to the host user's UID/GID so image-built files and bind-mounted files
# share an owner without runtime chown.
ARG AGENT_UID=1000
ARG AGENT_GID=1000

# Rename the base image's "node" user to "agent" and align UID/GID.
RUN groupmod -o -g $AGENT_GID node && usermod -o -u $AGENT_UID -g $AGENT_GID -d /home/agent -m -l agent node
USER ${AGENT_UID}:${AGENT_GID}

# Claude Code CLI + Rust (cargo) on PATH.
ENV PATH="/home/agent/.local/bin:/home/agent/.cargo/bin:$PATH"

# Claude Code CLI.
RUN curl -fsSL https://claude.ai/install.sh | bash

# Rust toolchain. telltaled is edition 2024 (needs >= 1.85); `stable` tracks a
# recent toolchain so the agent's `just check` mirrors what a developer runs
# locally. rustfmt + clippy are required by the gate.
ENV RUSTUP_HOME=/home/agent/.rustup CARGO_HOME=/home/agent/.cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --default-toolchain stable --profile minimal
RUN rustup component add rustfmt clippy

# `just` — the task runner that fronts the gate.
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh \
  | bash -s -- --to /home/agent/.local/bin

# NOTE: the cargo dependency tree is not pre-warmed at build time, so the first
# `cargo`/`just check` of each run cold-compiles deps. telltaled's tree is tiny
# today, so this is negligible; revisit (COPY Cargo.{toml,lock} + a throwaway
# build) if the dependency tree grows enough to make cold compiles painful.

WORKDIR /home/agent

# Sandcastle bind-mounts the git worktree and overrides WORKDIR to it at start.
ENTRYPOINT ["sleep", "infinity"]
