#!/bin/bash
# Knowledge Graph Generation Demo

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Knowledge Graph Generation Demo                               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Ensure we have data to work with
echo "📂 Step 1: Export bookmarks from all browsers"
echo "────────────────────────────────────────────────────────────"
cargo run -- export --output ./examples/graph-data
echo ""

# Generate DOT format for Graphviz
echo "📊 Step 2: Generate DOT format (Graphviz)"
echo "────────────────────────────────────────────────────────────"
cargo run -- graph \
    --format dot \
    --output ./examples/graphs/bookmarks.dot \
    --min-threshold 2
echo ""

# Generate JSON format for web visualization
echo "📊 Step 3: Generate JSON format (Web visualization)"
echo "────────────────────────────────────────────────────────────"
cargo run -- graph \
    --format json \
    --output ./examples/graphs/bookmarks.json \
    --min-threshold 3
echo ""

# Generate GEXF format for Gephi
echo "📊 Step 4: Generate GEXF format (Gephi network analysis)"
echo "────────────────────────────────────────────────────────────"
cargo run -- graph \
    --format gexf \
    --output ./examples/graphs/analysis.gexf \
    --min-threshold 5
echo ""

# Show results
echo "📈 Step 5: Graph generation complete!"
echo "────────────────────────────────────────────────────────────"
ls -lh ./examples/graphs/
echo ""

# Display DOT graph preview
echo "🔍 Step 6: DOT graph preview (first 20 lines)"
echo "────────────────────────────────────────────────────────────"
head -20 ./examples/graphs/bookmarks.dot
echo "..."

echo ""
echo "✨ Graph generation complete! Next steps:"
echo ""
echo "View in Graphviz:"
echo "  dot -Tpng ./examples/graphs/bookmarks.dot -o ./examples/graphs/bookmarks.png"
echo "  open ./examples/graphs/bookmarks.png"
echo ""
echo "Import into Gephi:"
echo "  1. Open Gephi"
echo "  2. File → Open → ./examples/graphs/analysis.gexf"
echo "  3. Apply layout (e.g., Force Atlas 2)"
echo ""
echo "Web visualization (use JSON output with D3.js or Cytoscape.js):"
echo "  The JSON contains nodes, edges, and metadata for easy visualization"
