# Changelog

All notable changes to this project will be documented in this file.

## [0.1.2] - 2026-02-02

### Added
- **MCP Server Implementation**
  - JSON-RPC 2.0 protocol support
  - Five core tools: export_bookmarks, search_bookmarks, list_browsers, process_bookmarks, generate_graph
  - Stdin/stdout communication for AI assistant integration
  - Complete MCP server documentation (`docs/MCP_SERVER.md`)
  - Binary target: `bookmark-mcp`

- **Library API**
  - Public `BookmarkManager` API for programmatic use
  - `export_bookmarks()` method for browser data export
  - `search()` method for bookmark searching
  - `graph_from_bookmarks()` method for knowledge graph generation
  - Complete library API documentation (`docs/LIBRARY_API.md`)
  - Example usage code (`examples/library_usage.rs`)

- **Build System**
  - Library and binary separation in Cargo.toml
  - Feature flags: `default`, `cli`, `mcp`
  - Optional dependencies for CLI-only features
  - Multiple binary targets support

- **Testing**
  - Integration tests (3 tests)
  - Documentation tests (1 test)
  - MCP server tests
  - Comprehensive test script (`test_all_modes.sh`)
  - Total: 80 tests passing

- **Documentation**
  - Updated README with all three usage modes
  - Created `docs/index.html` landing page
  - Created `IMPLEMENTATION_SUMMARY.md`
  - Created `CHANGELOG.md`

### Fixed
- Removed unused imports in library code
- Fixed unused variable warnings
- Fixed doctest examples to use correct synchronous API
- Fixed Bookmark struct field references in examples
- All compilation warnings resolved

### Changed
- Enhanced README with usage mode sections
- Updated TODO.md with completed items
- Improved documentation structure

## [0.1.1] - Previous Release

### Features
- CLI tool for bookmark management
- Multi-browser support (Chrome, Firefox, Safari, Edge)
- Export bookmarks and history
- Search and open bookmarks
- Deduplication and organization
- Knowledge graph generation (DOT, JSON, GEXF)
- 39 unit tests

---

## Version History

- **v0.1.2** (2026-02-02) - MCP server, Library API, Multi-mode support
- **v0.1.1** - Initial CLI implementation
- **v0.1.0** - Project foundation
