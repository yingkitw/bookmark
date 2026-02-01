# Bookmark Manager

A cross-platform Rust CLI tool to import, search, and open bookmarks from all browsers.

## Features

- 🌐 **Multi-Browser Support**: Chrome, Firefox, Safari, Edge, and more
- 🔍 **Instant Search**: Search across all your bookmarks by title or URL
- 🚀 **Quick Open**: Open bookmarks directly in your default browser
- 📊 **Export Support**: Export bookmarks, history, and passwords to structured formats
- 💾 **Multiple Formats**: YAML (primary), JSON and CSV support planned
- 🛡️ **Security-Focused**: Secure handling of sensitive data
- 🚀 **Cross-Platform**: Windows, macOS, and Linux support
- ⚡ **High Performance**: Efficient handling of large browser databases

## Quick Start

### Installation

```bash
cargo build --release
```

### Basic Usage

```bash
# Search for bookmarks
cargo run -- search --query "github"

# Open a bookmark (first match)
cargo run -- open --query "github" --first

# Open a bookmark (interactive selection)
cargo run -- open --query "github"

# List available browsers
cargo run -- list

# Export bookmarks to YAML
cargo run -- scan --data-type bookmarks --output ./exports
```

## Commands

### `search` - Search across all bookmarks

```bash
# Basic search (searches both title and URL)
cargo run -- search --query "github"

# Search in title only
cargo run -- search --query "github" --title-only

# Search in URL only
cargo run -- search --query "github" --url-only

# Limit results
cargo run -- search --query "github" --limit 10
```

### `open` - Open bookmarks in default browser

```bash
# Open first matching bookmark
cargo run -- open --query "github" --first

# Interactive selection from multiple matches
cargo run -- open --query "github"
```

### `list` - List available browsers and profiles

```bash
# List all browsers
cargo run -- list

# List profiles for specific browser
cargo run -- list --browser chrome
```

### `scan` - Automatically detect and export all browsers

```bash
# Export bookmarks from all detected browsers
cargo run -- scan --data-type bookmarks --output ./exports

# Export history from all detected browsers
cargo run -- scan --data-type history --output ./exports

# Export all data types
cargo run -- scan --data-type all --output ./exports
```

### `dedupe` - Remove duplicate bookmarks

```bash
# Deduplicate bookmarks with merge metadata strategy
cargo run -- dedupe --input ./bookmarks.yaml --output ./deduped.yaml

# Use different merge strategies
cargo run -- dedupe --input ./bookmarks.yaml --output ./deduped.yaml --strategy recent
cargo run -- dedupe --input ./bookmarks.yaml --output ./deduped.yaml --strategy first

# Preview changes without applying them
cargo run -- dedupe --input ./bookmarks.yaml --output ./deduped.yaml --preview

# Create backup of original file
cargo run -- dedupe --input ./bookmarks.yaml --output ./deduped.yaml --backup
```

### `organize` - Organize bookmarks into folders

```bash
# Organize bookmarks using custom rules (default)
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml

# Organize by domain only
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml --strategy domain

# Organize by category only
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml --strategy category

# Organize by date
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml --strategy date

# Preserve existing folder structure
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml --preserve-existing

# Preview changes without applying them
cargo run -- organize --input ./bookmarks.yaml --output ./organized.yaml --preview
```

### `process` - Complete deduplication and organization

```bash
# Full processing with default settings
cargo run -- process --input ./bookmarks.yaml --output ./processed.yaml

# Custom merge and organization strategies
cargo run -- process --input ./bookmarks.yaml --output ./processed.yaml --merge_strategy recent --organization_strategy domain

# Generate detailed report
cargo run -- process --input ./bookmarks.yaml --output ./processed.yaml --report ./report.md

# Preview changes before applying
cargo run -- process --input ./bookmarks.yaml --output ./processed.yaml --preview

# Use configuration file settings
cargo run -- process --input ./bookmarks.yaml --output ./processed.yaml --config ./my-config.yaml
```

### `config` - Manage configuration

```bash
# Show current configuration
cargo run -- config --show

# Create sample configuration file
cargo run -- config --create-sample ./sample-config.yaml

# Add custom organization rule
cargo run -- config --add-rule '{"name":"Custom Rule","pattern":"example\\.com","folder":"Examples","priority":5}'

# Remove custom organization rule
cargo run -- config --remove-rule "Custom Rule"

# List all custom rules
cargo run -- config --list-rules

# Validate configuration
cargo run -- config --validate

# Use custom config file
cargo run -- config --show --config-file ./my-config.yaml
```

### `export` - Export from specific browser or all browsers

```bash
# Export from all browsers
cargo run -- export --browser all --data-type bookmarks --output ./exports

# Export from specific browser
cargo run -- export --browser chrome --data-type bookmarks --output chrome.yaml

# Export to stdout
cargo run -- export --browser firefox --data-type history
```

## Options

- `--browser`: Browser to export from (chrome, firefox, safari, edge, all)
- `--data-type`: Type of data to export (bookmarks, history, passwords, all)
- `--output`: Output directory or file path (defaults to current directory)
- `--profile-dir`: Custom browser data directory

## Supported Browsers

| Browser | Bookmarks | History | Passwords | Status               |
| ------- | --------- | ------- | --------- | -------------------- |
| Chrome  | ✅        | ✅      | 🔄        | Basic support        |
| Firefox | ✅        | ✅      | 🔄        | Basic support        |
| Safari  | ✅        | ✅      | 🔄        | Manual copy required |
| Edge    | ✅        | ✅      | 🔄        | Basic support        |
| Brave   | 🔄        | 🔄      | 🔄        | Planned              |
| Vivaldi | 🔄        | 🔄      | 🔄        | Planned              |
| Opera   | 🔄        | 🔄      | 🔄        | Planned              |

_✅ Implemented | 🔄 In Progress | 📋 Planned_

## Safari Manual Export

On macOS, Safari bookmarks are protected. To export Safari bookmarks:

1. Open Finder
2. Press Shift+Command+G
3. Enter: `~/Library/Safari/`
4. Copy `Bookmarks.plist` to your Desktop or Downloads
5. Run: `cargo run -- export --browser safari --profile-dir ~/Desktop/Bookmarks.plist --data-type bookmarks --output safari-bookmarks.yaml`

## Firefox Database Lock

If Firefox is running, close it first or copy the database manually:

```bash
# Copy Firefox database and export from copy
cp ~/Library/Application\ Support/Firefox/Profiles/*/places.sqlite ~/Desktop/places.sqlite
cargo run -- export --browser firefox --profile-dir ~/Desktop/places.sqlite --data-type bookmarks --output firefox-bookmarks.yaml
```

## Output Format

The tool exports data in structured YAML format:

```yaml
- browser: chrome
  profile: Default
  export_date: 2026-02-01T08:04:05.082245Z
  bookmarks:
    - id: "4316"
      title: CRM Analytics | Salesforce
      url: https://ibmsc.lightning.force.com/analytics/dashboard/0FK3h000000dENLGA2
      folder: bookmark_bar/ibm
      date_added: 2026-01-28T01:16:01Z
      children: null
  history:
    urls:
      - url: https://example.com
        title: Example Page
        visit_count: 42
        last_visit: 2026-01-30T15:30:00Z
  passwords: null
```

## Performance

- **Memory Usage**: < 512MB for typical browser databases
- **Export Speed**: Complete browser export in < 30 seconds
- **Database Size**: Handles databases up to 1GB efficiently
- **Concurrent Exports**: Multiple browsers processed in parallel (planned)

## Security

- 🔒 **Read-Only Access**: Never modifies original browser data
- 🛡️ **Secure Handling**: No plaintext passwords in logs or temporary files
- 🔑 **Platform Integration**: Uses OS keychain APIs for password decryption
- 🚫 **No Telemetry**: All data processing is local and private

## Requirements

- Rust 1.70+
- Operating Systems:
  - macOS 10.15+
  - Windows 10+
  - Linux (Ubuntu 20.04+, Fedora 36+, Debian 11+)

## Examples

### Search and Open Bookmarks

```bash
# Search for GitHub repositories
cargo run -- search --query "github"

# Open GitHub homepage directly
cargo run -- open --query "github.com" --first

# Search for documentation bookmarks
cargo run -- search --query "docs" --title-only --limit 15
```

### Export all data from all browsers

```bash
cargo run -- scan --data-type all --output ./browser-exports
```

### Export specific data types

```bash
# Bookmarks only
cargo run -- scan --data-type bookmarks --output ./bookmarks

# History only
cargo run -- scan --data-type history --output ./history
```

### Export from custom profile directory

```bash
cargo run -- export --browser chrome --profile-dir "/custom/path/profile" --data-type bookmarks
```

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

### Run with debug logging

```bash
RUST_LOG=debug cargo run -- scan
```

## Documentation

- [📋 TODO List](TODO.md) - Development roadmap and tasks
- [📖 Technical Specification](SPEC.md) - Detailed technical requirements
- [🏗️ Architecture](ARCHITECTURE.md) - System design and implementation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Troubleshooting

### Common Issues

1. **"database is locked"**: Close the target browser before exporting
2. **"Operation not permitted"**: Grant appropriate permissions (Safari on macOS)
3. **"No browsers found"**: Check if browsers are installed in standard locations

### Getting Help

- Check the [TODO.md](TODO.md) for known issues
- Review the [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- Open an issue for bugs or feature requests

## Bookmark Deduplication & Organization

### Deduplication Features

The bookmark manager now includes advanced deduplication capabilities:

- **URL Normalization**: Automatically normalizes URLs by ignoring protocols, www subdomains, query parameters, and fragments
- **Smart Merging**: Multiple merge strategies to handle duplicate bookmarks:
  - `first`: Keep the first occurrence
  - `last`: Keep the last occurrence  
  - `recent`: Keep the most recently added bookmark
  - `frequent`: Keep the bookmark with the most common title
  - `merge`: Combine metadata from all duplicates (default)
- **Similarity Detection**: Finds potential duplicates using URL pattern analysis

### Organization Features

Intelligent bookmark organization with multiple strategies:

- **Custom Rules**: Predefined rules for common categories (Social, Development, Shopping, News, etc.)
- **Domain-based**: Organizes bookmarks by website domain
- **Category-based**: Uses content analysis to categorize bookmarks
- **Date-based**: Groups bookmarks by creation date
- **Preserve Existing**: Option to maintain existing folder structure while adding new organization

### Advanced Processing

The `process` command combines both deduplication and organization:

```bash
# Complete workflow: export → dedupe → organize
cargo run -- scan --data-type bookmarks --output ./raw.yaml
cargo run -- process --input ./raw.yaml --output ./clean.yaml --report ./report.md
```

## Roadmap

### v0.2.0 - Enhanced Search & UI ✅ COMPLETED

- ✅ Fuzzy search functionality
- ✅ Bookmark tags and categories
- 🔄 Interactive TUI interface (in progress)
- 🔄 Keyboard shortcuts (in progress)

### v0.3.0 - Advanced Features ✅ COMPLETED

- 🔄 Password export from all browsers (in progress)
- 🔄 Brave and Vivaldi browser support (planned)
- 🔄 JSON and CSV export formats (planned)
- ✅ Bookmark deduplication
- 🔄 Web-based interface (planned)

### v1.0.0 - Production Release

- Full browser ecosystem support
- Comprehensive testing suite
- Production-ready stability
- Complete documentation
