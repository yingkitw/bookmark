# Bookmark Manager - Examples

This directory contains demonstration scripts showcasing the capabilities of the Bookmark Manager.

## Scripts

### [quick-start.sh](quick-start.sh)
**Quick introduction to core features**
- Export bookmarks from all browsers
- List available browsers
- Search bookmarks
- Generate knowledge graph

**Run:**
```bash
chmod +x examples/quick-start.sh
./examples/quick-start.sh
```

---

### [knowledge-graph.sh](knowledge-graph.sh)
**Knowledge graph generation demo**
- Export bookmarks
- Generate DOT format (Graphviz)
- Generate JSON format (web visualization)
- Generate GEXF format (Gephi)

**Run:**
```bash
chmod +x examples/knowledge-graph.sh
./examples/knowledge-graph.sh
```

**Output:**
- `./examples/graphs/bookmarks.dot` - Graphviz DOT format
- `./examples/graphs/bookmarks.json` - JSON for web libraries
- `./examples/graphs/analysis.gexf` - GEXF for Gephi

**Visualize:**
```bash
# PNG using Graphviz
dot -Tpng ./examples/graphs/bookmarks.dot -o bookmarks.png
open bookmarks.png

# Gephi network analysis
# 1. Open Gephi
# 2. File → Open → ./examples/graphs/analysis.gexf
# 3. Apply layout (Force Atlas 2, Fruchterman Reingold, etc.)
```

---

### [processing.sh](processing.sh)
**Deduplication and organization demo**
- Export raw bookmarks
- Remove duplicates
- Organize by domain, category, or custom rules
- Compare different merge strategies

**Run:**
```bash
chmod +x examples/processing.sh
./examples/processing.sh
```

**Outputs:**
- `all-bookmarks.yaml` - Raw exported data
- `deduped.yaml` - Duplicates removed
- `organized.yaml` - Organized into folders
- `by-domain.yaml` - Organized by website domain
- `by-category.yaml` - Organized by content category
- `custom.yaml` - Organized with custom rules

**Strategies:**
| Mode | Description |
|------|-------------|
| `dedupe` | Remove duplicate bookmarks only |
| `organize` | Organize into folders only |
| `both` | Dedupe and organize (default) |

| Merge Strategy | Description |
|----------------|-------------|
| `first` | Keep first occurrence |
| `last` | Keep last occurrence |
| `recent` | Keep most recently added |
| `merge` | Merge metadata from all (default) |

| Org Strategy | Description |
|-------------|-------------|
| `domain` | Organize by website domain (github.com → Domains/github) |
| `category` | Organize by content category (docs → Development) |
| `custom` | Use predefined intelligent rules (default) |

---

### [search-open.sh](search-open.sh)
**Search and open bookmarks demo**
- Basic search
- Title-only search
- URL-only search
- Limited results
- Open first match
- Interactive selection

**Run:**
```bash
chmod +x examples/search-open.sh
./examples/search-open.sh
```

---

## Common Workflows

### Workflow 1: Clean up browser bookmarks
```bash
# 1. Export from all browsers
cargo run -- export --output ./cleanup/raw

# 2. Process (dedupe + organize)
cargo run -- process \
    -i ./cleanup/raw/chrome-bookmarks.yaml \
    -o ./cleanup/clean.yaml \
    --mode both \
    --strategy merge

# 3. Search to verify
cargo run -- search github --limit 5
```

### Workflow 2: Generate knowledge graph
```bash
# 1. Export data
cargo run -- export --data-type both --output ./graph/raw

# 2. Generate graph
cargo run -- graph \
    --format dot \
    --output ./graph/bookmarks.dot \
    --min-threshold 3

# 3. Visualize
dot -Tpng ./graph/bookmarks.dot -o ./graph/bookmarks.png
open ./graph/bookmarks.png
```

### Workflow 3: Analyze browsing history
```bash
# Export history
cargo run -- export --data-type history --output ./history

# Generate graph to see browsing patterns
cargo run -- graph \
    --data-type history \
    --format json \
    --output ./history-graph.json
```

### Workflow 4: Merge multiple browser exports
```bash
# Export from all browsers
cargo run -- export --output ./merge

# Combine files
cat ./merge/*.yaml > ./all-browsers.yaml

# Process (dedupe across browsers)
cargo run -- process \
    -i ./all-browsers.yaml \
    -o ./clean-merged.yaml \
    --mode dedupe \
    --strategy merge
```

---

## Command Reference

### Export
```bash
cargo run -- export                                   # All browsers, bookmarks
cargo run -- export --browser chrome                 # Specific browser
cargo run -- export --data-type history              # History only
cargo run -- export --data-type both --output ./data  # Both + custom dir
```

### Search
```bash
cargo run -- search query                             # Basic search
cargo run -- search query --title-only                # Title only
cargo run -- search query --url-only                  # URL only
cargo run -- search query --limit 10                  # Limit results
```

### Open
```bash
cargo run -- open query --first                       # Open first match
cargo run -- open query                               # Interactive selection
```

### Process
```bash
cargo run -- process -i in.yaml -o out.yaml           # Full processing
cargo run -- process --mode dedupe                    # Dedupe only
cargo run -- process --mode organize                  # Organize only
cargo run -- process --strategy recent                # Merge strategy
cargo run -- process --org-strategy domain            # Org strategy
cargo run -- process --preview                        # Preview changes
```

### Graph
```bash
cargo run -- graph --format dot -o graph.dot         # DOT format
cargo run -- graph --format json -o graph.json       # JSON format
cargo run -- graph --format gexf -o graph.gexf       # GEXF format
cargo run -- graph --min-threshold 5                  # Domain threshold
```

### Config
```bash
cargo run -- config --show                             # Show config
cargo run -- config --list-rules                       # List rules
cargo run -- config --create-sample config.yaml       # Create sample
```
