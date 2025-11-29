#!/bin/bash
# Ziron Setup Script

set -e

echo "🚀 Ziron Setup"
echo "=============="
echo ""

# Build project
echo "📦 Building project..."
cargo build --release

# Get binary paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$SCRIPT_DIR/target/release"

echo ""
echo "✅ Build completed!"
echo ""

# Initialize configuration
echo "⚙️  Initializing configuration..."
"$BIN_DIR/ziron-cli" init

echo ""
echo "📝 Configuration created at ~/.config/ziron/config.toml"
echo ""

# Add default plugins
echo "🔌 Adding default plugins..."
"$BIN_DIR/ziron-cli" plugin add git 2>/dev/null || true
"$BIN_DIR/ziron-cli" plugin add sysinfo 2>/dev/null || true

# Set default theme
echo "🎨 Setting default theme..."
"$BIN_DIR/ziron-cli" theme set default

echo ""
echo "✨ Setup completed!"
echo ""
echo "Next steps:"
echo "1. Add binaries to PATH (optional):"
echo "   export PATH=\"\$PATH:$BIN_DIR\""
echo ""
echo "2. Start the daemon:"
echo "   $BIN_DIR/ziron-daemon &"
echo ""
echo "3. Add to your shell config (~/.zshrc or ~/.bashrc):"
echo "   eval \"\$(ziron-daemon &)\""
echo "   export PROMPT='\$(ziron-prompt)'"
echo ""
echo "4. Reload your shell:"
echo "   source ~/.zshrc  # or source ~/.bashrc"
echo ""

