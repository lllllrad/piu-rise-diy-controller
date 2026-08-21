FROM docker:29.1.3-cli AS docker-cli

FROM rust:1.97-bookworm

ARG DIM_AGENT_UID=1000
ARG DIM_AGENT_GID=1000

COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins /usr/local/libexec/docker/cli-plugins

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        bash \
        ca-certificates \
        git \
        libasound2-dev \
        nodejs \
        npm \
        pkg-config \
        tini \
    && npm install --global @openai/codex @anthropic-ai/claude-code \
    && install --directory --mode 0755 \
        --owner "${DIM_AGENT_UID}" --group "${DIM_AGENT_GID}" /home/dim-agent \
    && rm -rf /var/lib/apt/lists/* /root/.npm

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["sleep", "infinity"]
