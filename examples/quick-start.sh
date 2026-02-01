#!/bin/bash
# Quick Start Demo - Export, Search, Open, and Generate Knowledge Graph

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Bookmark Manager - Quick Start Demo                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Export bookmarks from all browsers
echo "📂 Step 1: Export bookmarks from all browsers"
echo "────────────────────────────────────────────────────────────"
cargo run -- export --output ./examples/output
echo ""

# 2. List available browsers
echo "🌐 Step 2: List available browsers"
echo "────────────────────────────────────────────────────────────"
cargo run -- list
echo ""

# 3. Search bookmarks
echo "🔍 Step 3: Search for 'github' in bookmarks"
echo "────────────────────────────────────────────────────────────"
cargo run -- search github --limit 5
echo ""

# 4. Generate knowledge graph
echo "📊 Step 4: Generate knowledge graph (DOT format)"
echo "────────────────────────────────────────────────────────────"
cargo run -- graph --format dot --output ./examples/bookmarks.dot
echo ""

# 5. Show graph statistics
echo "📈 Step 5: Graph generated successfully!"
echo "────────────────────────────────────────────────────────────"
if [ -f "./examples/bookmarks.dot" ]; then
    echo "✓ Graph file: ./examples/bookmarks.dot"
    echo "  Lines: $(wc -l < ./examples/bookmarks.dot)"
    echo ""
    echo "To visualize, run:"
    echo "  dot -Tpng ./examples/bookmarks.dot -o ./examples/bookmarks.png"
    echo "  open ./examples/bookmarks.png"
fi

echo ""
echo "✨ Quick start complete! Try these next:"
echo "   cargo run -- open github              # Open a bookmark"
echo "   cargo run -- graph --format json      # JSON graph"
echo "   cargo run -- process --help           # Process bookmarks"
