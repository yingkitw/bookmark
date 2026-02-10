# TODO List

## Completed (v0.1.5)

- [x] **Full Codebase Modularization**
  - [x] Split `exporter.rs` (536 lines) → `exporter/mod.rs` + `chrome.rs` + `firefox.rs` + `safari.rs`
  - [x] Split `main.rs` (538 lines) → `main.rs` (223 lines, CLI defs) + `cli.rs` (348 lines, handlers)
  - [x] Split `mcp.rs` (437 lines) → `mcp/mod.rs` (269 lines, protocol) + `mcp/tools.rs` (176 lines)
  - [x] Split `organization.rs` (527 lines) → `organization/mod.rs` + `rules.rs` + `tests.rs`
  - [x] Split `deduplication.rs` (560 lines) → `deduplication/mod.rs` + `tests.rs`
  - [x] All 25 source files now under 350 lines (except HTML templates and test files)
  - [x] 48 tests passing (44 lib + 3 integration + 1 doc)

## Completed (v0.1.4)

- [x] **Graph Module Refactor**
  - [x] Split monolithic graph.rs (1870 lines) into 5 modules: mod.rs, builder.rs, analyzer.rs, formats.rs, tests.rs
  - [x] DRY: Unified `ingest_items()` method eliminates code duplication across from_bookmarks/from_history/from_both
  - [x] Extracted analyzer functions (extract_tags, categorize, jaccard_similarity, extract_domain) into standalone module
  - [x] Live data loading: `exporter::load_browser_data()` reads directly from browser DBs (no temp YAML files)
  - [x] Fixed CLI, Library API, and MCP server to all use live in-memory data loading
  - [x] 93 tests passing (46 lib + 44 bin + 3 integration + 1 doc)

## Completed (v0.1.3)

- [x] **Knowledge Graph Enhancement**
  - [x] Tag extraction from bookmark titles and URLs (stop-word filtering)
  - [x] Auto-categorization (Development, AI & ML, Cloud, Shopping, etc.)
  - [x] Similarity detection via Jaccard similarity on extracted tags
  - [x] New node types: Tag, Category
  - [x] New edge types: HasTag, InCategory, SimilarContent
  - [x] Interactive HTML visualization (D3.js force-directed graph)
  - [x] Dark/light theme, node type filters, adjustable physics
  - [x] HTML format support in CLI, Library API, and MCP server

## Completed (v0.1.2)

- [x] **MCP Server Implementation**
  - [x] JSON-RPC 2.0 protocol support
  - [x] Five core tools (export, search, list, process, graph)
  - [x] Stdin/stdout communication
  - [x] Error handling and validation
  - [x] Full MCP documentation

- [x] **Library API**
  - [x] BookmarkManager public API
  - [x] Export bookmarks method
  - [x] Search functionality
  - [x] Knowledge graph generation
  - [x] Example usage code
  - [x] API documentation with doctests

- [x] **Build Configuration**
  - [x] Library and binary separation
  - [x] Feature flags (cli, mcp)
  - [x] Optional dependencies
  - [x] Multiple binary targets
  - [x] Clean compilation warnings

- [x] **Testing**
  - [x] Unit tests (45 tests - all passing)
  - [x] Integration tests (3 tests - all passing)
  - [x] Documentation tests (1 test - passing)
  - [x] MCP server tests
  - [x] Test script for all modes
  - [x] Total: 92 tests passing (45 lib + 43 bin + 3 integration + 1 doc)

- [x] **Code Quality**
  - [x] Fixed all compilation warnings
  - [x] Fixed unused imports
  - [x] Fixed unused variables
  - [x] Fixed doctest examples
  - [x] Clean cargo test output

## High Priority

- [ ] **Password Export Implementation**
  - [ ] Chrome/Edge password decryption (OS-specific keychain access)
  - [ ] Firefox password extraction from key4.db
  - [ ] Safari password extraction from Keychain
  - [ ] Add secure password handling (no plaintext in logs)

## Medium Priority

- [ ] **Enhanced Browser Support**
  - [ ] Brave browser support
  - [ ] Vivaldi browser support
  - [ ] Opera browser support
  - [ ] Edge Canary/Dev/Beta profiles detection

- [ ] **Performance Optimizations**
  - [ ] Parallel export for multiple browsers
  - [ ] Large database handling optimization
  - [ ] Memory usage optimization for large exports

- [ ] **Data Validation**
  - [ ] YAML schema validation
  - [ ] Duplicate detection and removal
  - [ ] Broken URL detection
  - [ ] Export data integrity checks

## Low Priority

- [ ] **Advanced Features**
  - [ ] Export to other formats (JSON, CSV, HTML)
  - [ ] Bookmark synchronization between browsers
  - [ ] Bookmark deduplication across browsers
  - [ ] Bookmark statistics and analytics
  - [ ] CLI progress bars for large exports

- [ ] **User Experience**
  - [ ] Interactive mode with browser selection
  - [ ] Configuration file support
  - [ ] Verbose output modes
  - [ ] Colored output for better readability

- [ ] **Platform Enhancements**
  - [ ] Windows Certificate Store integration
  - [ ] Linux Secret Service integration
  - [ ] Android data directory support
  - [ ] iOS simulator data access

## Technical Debt

- [ ] **Code Quality**
  - [ ] Remove unused code and imports
  - [ ] Add comprehensive unit tests
  - [ ] Add integration tests
  - [ ] Improve error messages clarity

- [ ] **Documentation**
  - [ ] API documentation generation
  - [ ] Code comments and documentation
  - [ ] User guide with screenshots
  - [ ] Troubleshooting guide

## Security Considerations

- [ ] **Security Hardening**
  - [ ] Input validation and sanitization
  - [ ] Safe handling of sensitive data
  - [ ] Audit logging for security events
  - [ ] Permissions and access control

## Known Issues

- [ ] Safari bookmarks require manual copying on macOS
- [ ] Firefox database requires browser to be closed
- [ ] Windows registry access not implemented
- [ ] Large history exports may cause memory issues

## Future Releases

### v0.2.0

- Password export support
- Brave and Vivaldi browser support
- JSON export format

### v0.3.0

- Web-based UI
- Bookmark synchronization
- Advanced filtering and search

### v1.0.0

- Full browser ecosystem support
- Comprehensive security audit
- Production-ready stability
