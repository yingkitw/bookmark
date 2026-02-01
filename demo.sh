#!/bin/bash
# Main Demo Script - Showcases all features

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        📚 Bookmark Manager - Feature Demo                      ║"
echo "║                                                                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Create output directory
mkdir -p ./demo-output

echo "This demo will showcase the main features:"
echo "  ✓ Export bookmarks from browsers"
echo "  ✓ Search and open bookmarks"
echo "  ✓ Process (dedupe & organize)"
echo "  ✓ Generate knowledge graphs"
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 1. EXPORT
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 1: Export Bookmarks"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Exporting bookmarks from all browsers..."
cargo run -- export --output ./demo-output
echo ""
echo "✓ Export complete!"
echo "  Files created in ./demo-output/"
ls -1 ./demo-output/
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 2. LIST
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 2: List Available Browsers"
echo "═══════════════════════════════════════════════════════════════"
echo ""
cargo run -- list
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 3. SEARCH
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 3: Search Bookmarks"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "a) Search for 'github':"
cargo run -- search github --limit 3
echo ""
echo "b) Search in titles only:"
cargo run -- search docs --title-only --limit 3
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 4. PROCESS
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 4: Process Bookmarks (Dedupe & Organize)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Combine exported files
cat ./demo-output/*.yaml > ./demo-output/all-bookmarks.yaml
TOTAL=$(grep -c "  id:" ./demo-output/all-bookmarks.yaml || echo "0")
echo "Total bookmarks: $TOTAL"
echo ""

echo "Processing with preview mode..."
cargo run -- process \
    -i ./demo-output/all-bookmarks.yaml \
    -o ./demo-output/processed.yaml \
    --preview
echo ""
echo "✓ Preview complete!"
echo ""
echo "Press Enter to apply changes..."
read -r

cargo run -- process \
    -i ./demo-output/all-bookmarks.yaml \
    -o ./demo-output/processed.yaml
echo ""
echo "✓ Processing complete!"
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 5. KNOWLEDGE GRAPH
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 5: Generate Knowledge Graph"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Generating knowledge graph (DOT format)..."
cargo run -- graph \
    --format dot \
    --output ./demo-output/bookmarks.dot
echo ""
echo "✓ Graph generated!"
echo ""
echo "Graph preview (first 15 lines):"
head -15 ./demo-output/bookmarks.dot
echo "..."
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# 6. CONFIG
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  STEP 6: Configuration"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Organization rules:"
cargo run -- config --list-rules
echo ""
echo "Press Enter to continue..."
read -r

# ============================================
# SUMMARY
# ============================================
clear
echo "═══════════════════════════════════════════════════════════════"
echo "  Demo Complete!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📁 Generated files:"
ls -lh ./demo-output/
echo ""
echo "📚 What you learned:"
echo "  1. Export bookmarks from browsers"
echo "  2. List available browsers"
echo "  3. Search bookmarks by title or URL"
echo "  4. Process (remove duplicates and organize)"
echo "  5. Generate knowledge graphs"
echo "  6. View configuration"
echo ""
echo "🔗 Next steps:"
echo "  • Try specific examples: cd examples && ./quick-start.sh"
echo "  • Generate graphs: ./examples/knowledge-graph.sh"
echo "  • Process bookmarks: ./examples/processing.sh"
echo "  • Search and open: ./examples/search-open.sh"
echo ""
echo "  • Visualize graph: dot -Tpng ./demo-output/bookmarks.dot -o graph.png"
echo "  • View help: cargo run -- <command> --help"
echo ""
