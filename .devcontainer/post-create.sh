#!/bin/bash
set -e

echo "=== Setting up boundform development environment ==="

# Activate mise in this shell
eval "$(~/.local/bin/mise activate bash)"

# Trust the project's mise.toml
mise trust

# Install all tools via mise (rust, node, claude-code)
echo "Installing tools via mise..."
mise install

# Add Windows cross-compile target
echo "Adding Windows cross-compile target..."
rustup target add x86_64-pc-windows-gnu

# Verify installations
echo "Verifying installations..."
rustc --version
node --version
claude --version 2>/dev/null || echo "Claude Code CLI installed (version check may require auth)"

# Initialize cargo project if Cargo.toml doesn't exist yet
if [ ! -f "Cargo.toml" ]; then
    echo "Initializing Cargo project..."
    cargo init --name boundform
fi

# Build the project
echo "Building project..."
cargo build

echo "=== Setup complete ==="
echo ""
echo "Available commands:"
echo "  cargo build          - Build the project"
echo "  cargo test           - Run tests"
echo "  cargo clippy         - Lint"
echo "  cargo fmt            - Format code"
echo "  claude               - Start Claude Code CLI"
echo "  mise ls              - List installed tools"
