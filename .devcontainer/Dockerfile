FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Using base:ubuntu (not devcontainers/rust) intentionally —
# Rust is managed by mise to avoid version conflicts with a pre-installed toolchain.
# Only native build dependencies are installed here.
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/*

# Install mise as the vscode user (not root)
USER vscode
RUN curl https://mise.run | sh
ENV PATH="/home/vscode/.local/share/mise/shims:/home/vscode/.local/bin:${PATH}"

# Back to root for devcontainer features installation
USER root
