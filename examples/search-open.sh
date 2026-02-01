#!/bin/bash
# Search and Open Demo

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Search & Open Demo                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Ensure we have data
echo "📂 Step 1: Export bookmarks (if not already done)"
echo "────────────────────────────────────────────────────────────"
cargo run -- export --output ./examples/search-data
echo ""

echo "🔍 Step 2: Search examples"
echo "────────────────────────────────────────────────────────────"
echo ""

# Example 1: Basic search
echo "a) Basic search for 'github':"
echo "   $ cargo run -- search github"
cargo run -- search github --limit 3
echo ""

# Example 2: Title only search
echo "b) Search in titles only:"
echo "   $ cargo run -- search docs --title-only"
cargo run -- search docs --title-only --limit 3
echo ""

# Example 3: URL only search
echo "c) Search in URLs only:"
echo "   $ cargo run -- search github --url-only"
cargo run -- search github --url-only --limit 3
echo ""

# Example 4: Limited results
echo "d) Limit to 5 results:"
echo "   $ cargo run -- search rust --limit 5"
cargo run -- search rust --limit 5
echo ""

echo "🚀 Step 3: Open examples"
echo "────────────────────────────────────────────────────────────"
echo ""

# Example 5: Open first match
echo "a) Open first matching bookmark:"
echo "   $ cargo run -- open github --first"
echo ""
echo "   (Uncomment to run: cargo run -- open github --first)"
# cargo run -- open github --first
echo ""

# Example 6: Interactive open
echo "b) Interactive selection from multiple matches:"
echo "   $ cargo run -- open github"
echo ""
echo "   (Uncomment to run: cargo run -- open github)"
# cargo run -- open github
echo ""

echo "✨ Search tips:"
echo "   - Use --title-only to search only bookmark titles"
echo "   - Use --url-only to search only URLs"
echo "   - Use --limit N to show N results"
echo "   - Use --first with 'open' to skip selection"
echo ""
echo "Common search patterns:"
echo "   cargo run -- search 'rust|python'         # Multiple terms"
echo "   cargo run -- search 'github.*rust'        # Pattern"
echo "   cargo run -- search docs --limit 20       # More results"
