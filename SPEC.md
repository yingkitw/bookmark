# Technical Specification

## Overview

Bookmark Exporter is a cross-platform Rust CLI tool that extracts browser data (bookmarks, history, passwords) from multiple web browsers and exports them to structured YAML format.

## Requirements

### Functional Requirements

1. **Browser Support**
   - Chrome/Chromium-based browsers
   - Firefox/Mozilla-based browsers
   - Safari (macOS only)
   - Edge (Chromium-based)
   - Future: Brave, Vivaldi, Opera

2. **Data Types**
   - Bookmarks (title, URL, folder structure, timestamps)
   - History (URL, title, visit count, timestamps)
   - Passwords (URL, username, encrypted password)

3. **Export Formats**
   - YAML (primary)
   - JSON (future)
   - CSV (future)

4. **Knowledge Graph Generation**
   - Node types: Bookmark, Domain, Folder, Tag, Category
   - Edge types: BelongsToDomain, InFolder, SameDomain, HasTag, InCategory, SimilarContent
   - Tag extraction from titles and URLs with stop-word filtering
   - Auto-categorization into 10 categories via keyword matching
   - Similarity detection via Jaccard similarity on tag sets
   - Export formats: HTML (interactive D3.js), DOT (Graphviz), JSON, GEXF (Gephi)
   - Configurable thresholds for domain, tag, and similarity edges

5. **Operating Systems**
   - macOS (10.15+)
   - Windows (10+)
   - Linux (major distributions)

### Non-Functional Requirements

1. **Performance**
   - Export complete browser data within 30 seconds
   - Handle databases up to 1GB
   - Memory usage under 512MB

2. **Security**
   - No plaintext passwords in logs
   - Secure handling of encrypted data
   - Respect OS security boundaries

3. **Reliability**
   - Graceful handling of locked databases
   - Recovery from corrupted data
   - Comprehensive error reporting

## Architecture

### Core Components

1. **CLI Interface** (`main.rs`)
   - Command parsing using clap
   - Argument validation
   - Output formatting

2. **Browser Detection** (`browser.rs`)
   - Cross-platform browser discovery
   - Profile enumeration
   - Data path resolution

3. **Data Extraction** (`exporter.rs`)
   - Browser-specific parsers
   - Database access layer
   - Format conversion

4. **Output Generation**
   - YAML serialization
   - File I/O operations
   - Error handling

### Data Flow

```
CLI Arguments → Browser Detection → Data Extraction → Format Conversion → File Output
```

### Browser Data Storage

#### Chrome/Chromium

- Bookmarks: `Profile/Bookmarks` (JSON)
- History: `Profile/History` (SQLite)
- Passwords: `Profile/Login Data` (SQLite, encrypted)

#### Firefox

- Bookmarks/History: `Profile/places.sqlite` (SQLite)
- Passwords: `Profile/logins.json` (encrypted)
- Encryption: `Profile/key4.db` (SQLite)

#### Safari

- Bookmarks: `~/Library/Safari/Bookmarks.plist` (Property List)
- History: `~/Library/Safari/History.db` (SQLite)
- Passwords: System Keychain

### Security Considerations

1. **Data Access**
   - Safari: Protected by macOS permissions
   - Firefox: Database locking mechanism
   - Chrome/Edge: OS-level encryption

2. **Password Handling**
   - Use platform-specific keychain APIs
   - No plaintext storage in temporary files
   - Secure memory management

3. **File Permissions**
   - Read-only access to browser data
   - No modification of original files
   - Temporary file cleanup

## Implementation Details

### Error Handling Strategy

1. **Graceful Degradation**
   - Continue export when one browser fails
   - Provide clear error messages
   - Offer alternative solutions

2. **Error Types**
   - Permission denied
   - Database locked
   - File not found
   - Corrupted data
   - Platform limitations

### Cross-Platform Support

1. **Path Resolution**
   - Use `dirs` crate for standard directories
   - Platform-specific default locations
   - Custom path support

2. **Database Access**
   - SQLite with appropriate flags
   - File copying for locked databases
   - Connection pooling for performance

3. **Security Integration**
   - macOS: Keychain Services
   - Windows: Credential Manager
   - Linux: Secret Service API

### Performance Optimization

1. **Memory Management**
   - Streaming for large datasets
   - Lazy loading of data structures
   - Efficient string handling

2. **I/O Optimization**
   - Batch database operations
   - Async file operations (future)
   - Compression for large exports

## Testing Strategy

### Unit Tests

- Browser detection logic
- Data parsing functions
- Error handling paths

### Integration Tests

- End-to-end export workflows
- Multi-browser scenarios
- Platform-specific behavior

### Performance Tests

- Large database handling
- Memory usage profiling
- Export time benchmarks

## Extensibility

### New Browser Support

1. Add browser enum variant
2. Implement data path detection
3. Create extraction functions
4. Add platform-specific handling

### New Export Formats

1. Define data structure
2. Implement serialization
3. Add CLI option
4. Update documentation

### New Data Types

1. Define data model
2. Implement extraction logic
3. Add to export pipeline
4. Update tests

## Dependencies

### Core Dependencies

- `clap`: CLI argument parsing
- `serde`: Serialization framework
- `serde_yaml`: YAML output
- `rusqlite`: SQLite database access
- `dirs`: Cross-platform directories
- `anyhow`: Error handling
- `chrono`: Date/time handling

### Browser-Specific

- `plist`: Safari property list parsing
- `libsqlite3-sys`: SQLite bindings

### Platform-Specific

- macOS: Security framework
- Windows: Win32 API
- Linux: D-Bus interfaces

## Release Planning

### v0.1.0 (Current)

- Basic bookmark/history export
- Chrome, Firefox, Safari support
- YAML output format
- Cross-platform compatibility

### v0.2.0

- Password export implementation
- Additional browser support
- Performance improvements
- Enhanced error handling

### v1.0.0

- Full feature set
- Comprehensive testing
- Production stability
- Complete documentation
