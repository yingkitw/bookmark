# Architecture Documentation

## System Overview

Bookmark Exporter is a modular, cross-platform CLI application built with Rust that extracts browser data from multiple web browsers and converts it to structured YAML format.

## High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Layer     │────│  Detection      │────│  Extraction     │────│   Output Layer  │
│ main.rs + cli.rs│    │  browser.rs     │    │  exporter/      │    │  graph/formats  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │                       │
         ▼                       ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Command Parse  │    │  Browser Enum   │    │  chrome.rs      │    │  DOT/JSON/GEXF  │
│  Dispatch       │    │  Path Resolver  │    │  firefox.rs     │    │  HTML (D3.js)   │
│  (clap)         │    │  Profile Finder │    │  safari.rs      │    │  Serialization  │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Module Architecture

### 1. CLI Layer (`main.rs` + `cli.rs`)

**Purpose**: Command-line interface and application entry point

**Module Structure**:

- `main.rs`: CLI definitions (clap Parser/Subcommand) and dispatch (~220 lines)
- `cli.rs`: Handler functions for each command (~350 lines)

**Components**:

- `Cli`: Main argument parser using clap
- `Commands`: Enum for subcommands (Export, List, Search, Open, Process, Graph, Config)
- Handler functions: `export_all_browsers`, `process_bookmarks`, `generate_graph`, `handle_config`, `list_all_browsers`, `list_browser_profiles`

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

### 3. Data Extraction (`exporter/`)

**Purpose**: Extract and parse browser-specific data formats

**Module Structure**:

- `exporter/mod.rs`: Types (Bookmark, UrlEntry, BrowserData), public API (`load_browser_data`, `export_data`), browser dispatch
- `exporter/chrome.rs`: Chrome/Edge bookmark JSON + History SQLite parsing
- `exporter/firefox.rs`: Firefox places.sqlite bookmark + history parsing (with lock-safe copy)
- `exporter/safari.rs`: Safari Bookmarks.plist parsing

**Key API**: `load_browser_data(browser, data_type)` — reads live from browser databases in-memory, no temp files

**Data Flow**:

```
Browser DB/JSON/plist → Browser-specific parser → Unified Bookmark/UrlEntry model
```

### 4. Knowledge Graph Engine (`graph/`)

**Purpose**: Generate rich knowledge graphs from bookmark/history data

**Module Structure**:

- `graph/mod.rs`: Types (NodeType, EdgeType, GraphNode, GraphEdge, KnowledgeGraph, GraphConfig)
- `graph/builder.rs`: GraphBuilder with unified `ingest_items()` pipeline (DRY)
- `graph/analyzer.rs`: Tag extraction, categorization, similarity (Jaccard), domain extraction
- `graph/formats.rs`: Export formats (DOT, JSON, GEXF, HTML with D3.js)
- `graph/tests.rs`: 18 unit tests

**Data Loading**: Reads live from browser databases via `exporter::load_browser_data()` — no intermediate file I/O

**Components**:

- `GraphBuilder`: Stateful builder with unified `ingest_items()` method
- `GraphConfig`: Configuration for edge types, thresholds, detail levels, similarity
- `KnowledgeGraph`: Output structure with nodes, edges, metadata

**Node Types**: Bookmark, Domain, Folder, Tag, Category

**Edge Types**: BelongsToDomain, InFolder, SameDomain, HasTag, InCategory, SimilarContent

**Processing Pipeline**:

```
Bookmarks → Tag Extraction → Category Assignment → Node/Edge Creation → Similarity Detection → Graph Output
```

**Tag Extraction**: Splits titles/URLs into tokens, filters stop words, extracts URL path segments

**Auto-Categorization**: Keyword-based classification into 10 categories (Development, AI & ML, Cloud & DevOps, News, Social, Shopping, Finance, Education, Design, Reference)

**Similarity Detection**: Jaccard similarity on extracted tag sets between bookmark pairs

**Export Formats**:

- HTML: Interactive D3.js force-directed graph with dark/light theme, filters, zoom/pan
- DOT: Graphviz format with color-coded node/edge types
- JSON: Structured data for web visualization
- GEXF: Gephi network analysis format

### 5. Data Models

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
