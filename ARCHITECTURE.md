# Architecture Documentation

## System Overview

Bookmark Exporter is a modular, cross-platform CLI application built with Rust that extracts browser data from multiple web browsers and converts it to structured YAML format.

## High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Layer     │────│  Detection      │────│  Extraction     │────│   Output Layer  │
│   (main.rs)     │    │   (browser.rs)  │    │  (exporter.rs)  │    │   (YAML)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │                       │
         ▼                       ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Command Parse  │    │  Browser Enum   │    │  Data Models    │    │  File I/O       │
│  Validation     │    │  Path Resolver  │    │  Parsers        │    │  Serialization  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Module Architecture

### 1. CLI Layer (`main.rs`)

**Purpose**: Command-line interface and application entry point

**Components**:

- `Cli`: Main argument parser using clap
- `Commands`: Enum for subcommands (Export, List, Scan)
- `export_all_browsers()`: Batch export coordination

**Responsibilities**:

- Parse and validate CLI arguments
- Coordinate between detection and extraction modules
- Handle user feedback and error display
- Manage output directory creation

**Key Design Patterns**:

- Command pattern for different operations
- Strategy pattern for different data types

### 2. Browser Detection (`browser.rs`)

**Purpose**: Discover and enumerate browser installations and profiles

**Components**:

- `Browser` enum: Supported browser types
- `get_default_data_dir()`: Platform-specific path resolution
- `find_profiles()`: Profile discovery logic
- `list_all_browsers()`: Browser enumeration

**Responsibilities**:

- Cross-platform browser detection
- Profile directory discovery
- Path resolution for browser data files
- Browser availability validation

**Platform-Specific Logic**:

```rust
match platform {
    MacOS => ~/Library/Application Support/{Browser}
    Windows => %APPDATA%/{Browser}/User Data
    Linux => ~/.config/{browser}
}
```

### 3. Data Extraction (`exporter.rs`)

**Purpose**: Extract and parse browser-specific data formats

**Components**:

- `BrowserData`: Unified data model
- `extract_bookmarks()`: Bookmark extraction dispatcher
- `extract_history()`: History extraction dispatcher
- Browser-specific parsers

**Responsibilities**:

- Handle different browser data formats
- Parse structured data (JSON, SQLite, plist)
- Normalize data to unified model
- Handle locked/corrupted databases

**Data Flow**:

```
Raw Browser Data → Format Parser → Normalized Model → Validation
```

### 4. Data Models

**Core Structures**:

```rust
pub struct BrowserData {
    pub browser: String,
    pub profile: String,
    pub export_date: DateTime<Utc>,
    pub bookmarks: Option<Vec<Bookmark>>,
    pub history: Option<HistoryEntry>,
    pub passwords: Option<Vec<Password>>,
}

pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub folder: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
    pub children: Option<Vec<Bookmark>>,
}
```

## Cross-Platform Architecture

### Abstraction Layers

1. **File System Layer**
   - Uses `dirs` crate for standard directories
   - Platform-specific path templates
   - Custom path override support

2. **Database Access Layer**
   - SQLite with `rusqlite`
   - Connection pooling (future enhancement)
   - Lock handling and recovery

3. **Security Layer**
   - Platform-specific keychain access
   - Permission handling
   - Secure memory management

### Browser-Specific Implementations

#### Chrome/Chromium

```
Profile/Bookmarks (JSON) → parse_chrome_bookmarks()
Profile/History (SQLite) → extract_chrome_history()
Profile/Login Data (SQLite) → extract_chrome_passwords()
```

#### Firefox

```
Profile/places.sqlite (SQLite) → extract_firefox_bookmarks()
Profile/places.sqlite (SQLite) → extract_firefox_history()
Profile/logins.json (JSON) → extract_firefox_passwords()
```

#### Safari

```
~/Library/Safari/Bookmarks.plist (Property List) → extract_safari_bookmarks()
~/Library/Safari/History.db (SQLite) → extract_safari_history()
System Keychain → extract_safari_passwords()
```

## Error Handling Architecture

### Error Types

1. **Recoverable Errors**
   - Browser not installed
   - Database locked
   - Permission denied
   - Corrupted data

2. **Configuration Errors**
   - Invalid arguments
   - Missing dependencies
   - Path not found

3. **System Errors**
   - Out of memory
   - Disk full
   - Network errors (future)

### Error Handling Strategy

```rust
match operation {
    Ok(result) => proceed_with(result),
    Err(error) => {
        match error.kind() {
            PermissionDenied => offer_manual_workaround(),
            DatabaseLocked => suggest_close_browser(),
            NotFound => continue_with_next_browser(),
            _ => log_and_continue(),
        }
    }
}
```

## Performance Architecture

### Memory Management

1. **Streaming for Large Datasets**
   - Iterator-based database access
   - Lazy loading of bookmark trees
   - Chunked file processing

2. **Efficient Data Structures**
   - String interning for repeated URLs
   - Compact timestamp representation
   - Minimal temporary allocations

### I/O Optimization

1. **Database Access**
   - Prepared statements for repeated queries
   - Batch operations where possible
   - Connection reuse

2. **File Operations**
   - Buffered I/O for large files
   - Atomic file writes
   - Temporary file cleanup

## Security Architecture

### Data Protection

1. **Access Control**
   - Read-only access to browser data
   - No modification of original files
   - Permission validation

2. **Sensitive Data Handling**
   - No plaintext passwords in logs
   - Secure memory for password extraction
   - Temporary file encryption (future)

### Platform Security Integration

```rust
match platform {
    MacOS => Keychain Services API,
    Windows => Credential Manager API,
    Linux => Secret Service API,
}
```

## Extensibility Architecture

### Adding New Browsers

1. **Enum Extension**

```rust
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Brave,  // New browser
}
```

2. **Implementation Pattern**
   - Add path resolution logic
   - Implement extraction functions
   - Add platform-specific handling
   - Update tests

### Adding New Export Formats

1. **Trait-Based Design** (future enhancement)

```rust
trait Exporter {
    fn export(data: &BrowserData, output: &Path) -> Result<()>;
}
```

2. **Plugin Architecture** (future)
   - Dynamic format loading
   - Custom serializer registration
   - Format validation

## Dependency Management

### Core Dependencies

```
clap: CLI argument parsing
serde: Serialization framework
serde_yaml: YAML output
rusqlite: SQLite database access
dirs: Cross-platform directories
anyhow: Error handling
chrono: Date/time handling
plist: Safari property list parsing
```

### Dependency Graph

```
main.rs
├── clap
├── browser.rs
│   ├── dirs
│   └── std::fs
└── exporter.rs
    ├── serde_yaml
    ├── rusqlite
    ├── plist
    ├── chrono
    └── serde
```

## Testing Architecture

### Test Organization

1. **Unit Tests**
   - Browser detection logic
   - Data parsing functions
   - Error handling paths

2. **Integration Tests**
   - End-to-end export workflows
   - Cross-platform behavior
   - Database handling

3. **Mock Tests**
   - Browser simulation
   - File system mocking
   - Error scenario testing

### Test Data

1. **Sample Browser Data**
   - Chrome bookmark JSON
   - Firefox SQLite databases
   - Safari plist files

2. **Test Scenarios**
   - Empty databases
   - Corrupted files
   - Large datasets
   - Permission issues

## Future Architectural Enhancements

### Planned Improvements

1. **Async/Await Migration**
   - Parallel browser processing
   - Non-blocking I/O operations
   - Better resource utilization

2. **Plugin System**
   - Dynamic browser support
   - Custom export formats
   - Third-party extensions

3. **Web Interface**
   - REST API layer
   - Web-based UI
   - Remote operation capability

4. **Performance Optimizations**
   - Database connection pooling
   - Memory-mapped files
   - Compression for large exports

### Scalability Considerations

1. **Large Dataset Handling**
   - Streaming processing
   - Progress indicators
   - Resume capability

2. **Multi-User Support**
   - Profile isolation
   - Concurrent access
   - Resource limits
