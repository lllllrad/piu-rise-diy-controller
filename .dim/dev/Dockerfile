FROM docker:29.1.3-cli AS docker-cli
FROM node:24.6.0-bookworm-slim AS node-runtime

FROM rust:1.97-bookworm

ARG DIM_AGENT_UID=1000
ARG DIM_AGENT_GID=1000
ARG JUST_VERSION=1.42.4

COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins /usr/local/libexec/docker/cli-plugins
COPY --from=node-runtime /usr/local/ /usr/local/

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        bash \
        ca-certificates \
        curl \
        git \
        libasound2-dev \
        pkg-config \
        tini \
    && cargo install just --locked --version "${JUST_VERSION}" \
    && npm install --global @openai/codex @anthropic-ai/claude-code \
    && install --directory --mode 0755 \
        --owner "${DIM_AGENT_UID}" --group "${DIM_AGENT_GID}" /home/dim-agent \
    && rm -rf /var/lib/apt/lists/* /root/.npm

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["sleep", "infinity"]
