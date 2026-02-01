#!/bin/bash

# Demo script for bookmark deduplication and organization features

echo "🚀 Bookmark Manager Demo - Deduplication & Organization"
echo "=================================================="

# Step 1: Export bookmarks from all browsers
echo "Step 1: Exporting bookmarks from all browsers..."
cargo run -- scan --data-type bookmarks --output ./demo-exports

# Step 2: Show sample of raw exported bookmarks
echo -e "\nStep 2: Sample of raw exported bookmarks..."
find ./demo-exports -name "*.yaml" -exec echo "File: {}" \; -exec head -20 {} \;

# Step 3: Create a sample configuration
echo -e "\nStep 3: Creating sample configuration..."
cargo run -- config --create-sample ./demo-config.yaml

# Step 4: Show configuration rules
echo -e "\nStep 4: Showing organization rules..."
cargo run -- config --list-rules --config-file ./demo-config.yaml

# Step 5: Process bookmarks with deduplication and organization
echo -e "\nStep 5: Processing bookmarks (deduplication + organization)..."

# Combine all exported files into one for processing
cat ./demo-exports/*.yaml > ./all-bookmarks.yaml

# Process with preview first
echo "Preview of changes:"
cargo run -- process --input ./all-bookmarks.yaml --output ./processed-bookmarks.yaml --preview --config-file ./demo-config.yaml

# Step 6: Ask user if they want to apply changes
echo -e "\nDo you want to apply these changes? (y/n)"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    echo "Applying changes..."
    cargo run -- process --input ./all-bookmarks.yaml --output ./processed-bookmarks.yaml --config-file ./demo-config.yaml --report ./processing-report.md
    
    echo -e "\n✅ Processing complete! Check:"
    echo "   - Processed bookmarks: ./processed-bookmarks.yaml"
    echo "   - Processing report: ./processing-report.md"
    echo "   - Configuration: ./demo-config.yaml"
else
    echo "❌ Changes not applied. Preview mode only."
fi

echo -e "\n🎉 Demo complete!"
echo "\nAvailable commands:"
echo "  cargo run -- dedupe --help      # Show deduplication options"
echo "  cargo run -- organize --help     # Show organization options"
echo "  cargo run -- process --help      # Show combined processing options"
echo "  cargo run -- config --help       # Show configuration options"