#!/bin/bash
# Bookmark Processing Demo - Deduplicate and Organize

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Bookmark Processing Demo - Deduplicate & Organize            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Setup output directory
mkdir -p ./examples/processing

# Step 1: Export raw bookmarks
echo "📂 Step 1: Export raw bookmarks from all browsers"
echo "────────────────────────────────────────────────────────────"
cargo run -- export --output ./examples/processing/raw
echo ""

# Step 2: Combine all exported files
echo "📦 Step 2: Combine exported files"
echo "────────────────────────────────────────────────────────────"
cat ./examples/processing/raw/*.yaml > ./examples/processing/all-bookmarks.yaml
echo "✓ Combined into: ./examples/processing/all-bookmarks.yaml"
echo "  Total bookmarks: $(grep -c "  id:" ./examples/processing/all-bookmarks.yaml || echo "0")"
echo ""

# Step 3: Preview deduplication only
echo "🔍 Step 3: Preview deduplication (remove duplicates)"
echo "────────────────────────────────────────────────────────────"
cargo run -- process \
    --input ./examples/processing/all-bookmarks.yaml \
    --output ./examples/processing/deduped.yaml \
    --mode dedupe \
    --preview
echo ""

# Step 4: Run deduplication
echo "⚡ Step 4: Run deduplication"
echo "────────────────────────────────────────────────────────────"
cargo run -- process \
    --input ./examples/processing/all-bookmarks.yaml \
    --output ./examples/processing/deduped.yaml \
    --mode dedupe \
    --strategy merge
echo ""

# Step 5: Preview organization
echo "📁 Step 5: Preview organization (by domain)"
echo "────────────────────────────────────────────────────────────"
cargo run -- process \
    --input ./examples/processing/deduped.yaml \
    --output ./examples/processing/organized.yaml \
    --mode organize \
    --org-strategy domain \
    --preview
echo ""

# Step 6: Run organization
echo "🗂️  Step 6: Run organization"
echo "────────────────────────────────────────────────────────────"
cargo run -- process \
    --input ./examples/processing/deduped.yaml \
    --output ./examples/processing/organized.yaml \
    --mode organize \
    --org-strategy custom
echo ""

# Step 7: Full processing (dedupe + organize) with different strategies
echo "⚙️  Step 7: Full processing with different strategies"
echo "────────────────────────────────────────────────────────────"

echo ""
echo "a) By domain only:"
cargo run -- process \
    --input ./examples/processing/all-bookmarks.yaml \
    --output ./examples/processing/by-domain.yaml \
    --mode both \
    --org-strategy domain

echo ""
echo "b) By category only:"
cargo run -- process \
    --input ./examples/processing/all-bookmarks.yaml \
    --output ./examples/processing/by-category.yaml \
    --mode both \
    --org-strategy category

echo ""
echo "c) Custom rules (default):"
cargo run -- process \
    --input ./examples/processing/all-bookmarks.yaml \
    --output ./examples/processing/custom.yaml \
    --mode both \
    --org-strategy custom
echo ""

# Step 8: Show results
echo "📊 Step 8: Processing complete!"
echo "────────────────────────────────────────────────────────────"
ls -lh ./examples/processing/*.yaml
echo ""

echo "✨ Try comparing the outputs:"
echo "   diff ./examples/processing/all-bookmarks.yaml ./examples/processing/custom.yaml"
echo ""
echo "Available merge strategies:"
echo "   first   - Keep first occurrence"
echo "   last    - Keep last occurrence"
echo "   recent  - Keep most recently added"
echo "   merge   - Merge metadata (default)"
