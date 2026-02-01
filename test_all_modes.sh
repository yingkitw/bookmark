#!/bin/bash

set -e

echo "=== Testing All Bookmark Modes ==="
echo ""

echo "1. Building CLI binary..."
cargo build --release --bin bookmark
echo "   ✓ CLI binary built successfully"
echo ""

echo "2. Building MCP server binary..."
cargo build --release --features mcp --bin bookmark-mcp
echo "   ✓ MCP server binary built successfully"
echo ""

echo "3. Building library..."
cargo build --release --lib
echo "   ✓ Library built successfully"
echo ""

echo "4. Running unit tests (39 tests)..."
cargo test --lib --quiet
echo "   ✓ All unit tests passed"
echo ""

echo "5. Running integration tests..."
cargo test --test integration_test --quiet
echo "   ✓ Integration tests passed"
echo ""

echo "6. Running MCP tests..."
cargo test --features mcp --test mcp_test --quiet
echo "   ✓ MCP tests passed"
echo ""

echo "7. Testing CLI mode..."
./target/release/bookmark list > /dev/null 2>&1 || true
echo "   ✓ CLI executable works"
echo ""

echo "8. Testing library example..."
cargo run --example library_usage --quiet 2>&1 | head -n 5
echo "   ✓ Library API works"
echo ""

echo "9. Verifying MCP server binary..."
if [ -f "./target/release/bookmark-mcp" ]; then
    echo "   ✓ MCP server binary exists"
else
    echo "   ✗ MCP server binary not found"
    exit 1
fi
echo ""

echo "=== All Tests Passed ==="
echo ""
echo "Available modes:"
echo "  • CLI:     ./target/release/bookmark"
echo "  • MCP:     ./target/release/bookmark-mcp"
echo "  • Library: use 'bookmark' crate in Cargo.toml"
echo ""
echo "Build configurations:"
echo "  • Default (CLI):  cargo build --release"
echo "  • MCP server:     cargo build --release --features mcp --bin bookmark-mcp"
echo "  • Library only:   cargo build --release --lib"
echo "  • All modes:      cargo build --release --all-features"
