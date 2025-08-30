<!-- markdownlint-disable MD033 -->

# Resourcely

[![Crates.io](https://img.shields.io/crates/v/resourcely)](https://crates.io/crates/resourcely)
[![Documentation](https://docs.rs/resourcely/badge.svg)](https://docs.rs/resourcely)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Dependency Status](https://deps.rs/repo/github/dominikj111/resourcely/status.svg)](https://deps.rs/repo/github/dominikj111/resourcely)

Resourcely is a Rust library that provides a convenient way to manage and access resources from both local and remote sources. It offers a unified interface for reading and writing structured data with built-in caching and staleness control.

## Features

- **Unified Resource Access**: Consistent API for both local and remote resources
- **Multiple Formats**: Support for JSON and YAML <span style="color:gray">_(TOML and plain text in development)_</span>
- **Caching**: Configurable caching with time-based expiration
- **Staleness Control**: Fine-grained control over when to use cached data
- **Thread-Safe**: Designed for concurrent access
- **Zero-copy Parsing**: Efficient handling of large files

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
resourcely = "0.1.0"
```

## Quick Start

```rust
use resourcely::{Local, Remote, ResourceFileType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Config {
    api_key: String,
    timeout: u32,
}

// Local file resource
let local = Local::new(
    "config.json".to_string(),
    ResourceFileType::Json,
    PathBuf::from("./config"),
    None, // No cache timeout
);

// Remote resource
let remote = Remote::new(
    "data.json".to_string(),
    ResourceFileType::Json,
    "https://api.example.com/data".to_string(),
    PathBuf::from("/tmp/resourcely/cache"),
    Some(std::time::Duration::from_secs(300)), // 5 minute cache
);
```

## Usage

### Reading Data

```rust
use resourcely::{DataResult, ResourceReader};

// Get data or return error
match local.get_data_or_error(false).await {
    Ok(DataResult::Fresh(data)) => println!("Fresh data: {:?}", data),
    Ok(DataResult::Stale(data)) => println!("Stale data: {:?}", data),
    Err(e) => eprintln!("Error: {}", e),
}

// Get data or default
let data = remote.get_data_or_default(false).await;
println!("Data: {:?}", data);

// Get data or none
if let Some(data) = local.get_data_or_none(true).await {
    println!("Got data: {:?}", data);
}
```

### Marking Data as Stale

```rust
// Force refresh on next read
local.mark_as_stale()?;

// Check if data is marked as stale
if local.is_marked_stale()? {
    println!("Data is marked as stale");
}

// Check if data is fresh (considering cache timeouts and stale marker)
if local.is_fresh()? {
    println!("Data is fresh");
}
```

## Advanced Features

### Custom Parsing

The library provides support for JSON and YAML formats out of the box. TOML and plain text formats are defined in the `ResourceFileType` enum but not yet implemented. You can extend functionality by implementing the `ResourceReader` trait for your custom types.

### Resource State Management

The `ResourceReader` trait provides several methods for managing resource state:

```rust
// Check if data is fresh (not stale and within cache timeout)
let is_fresh = resource.is_fresh()?;

// Check if data is marked as stale
let is_stale = resource.is_marked_stale()?;

// Mark data as stale to force refresh
resource.mark_as_stale()?;
```

### Builder Pattern (Incomplete)

A builder pattern is partially implemented but currently incomplete:

```rust
// Note: Builder implementation is currently incomplete
// and references undefined traits (LocalResource, RemoteResource)
// Use the direct state_manager API for now
```

### Error Handling

All operations return `Result` types with descriptive error messages for better error handling.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## TODO

### Completed Achievements ✅

- ✅ **Core Resource Management System** - Unified interface for local and remote resource access
- ✅ **Advanced Error Handling** - Comprehensive error types with descriptive messages using `thiserror`
- ✅ **Multi-format Support** - JSON and YAML parsing with extensible format system
- ✅ **Intelligent Caching** - Time-based cache expiration with staleness control and state management
- ✅ **Thread-safe Architecture** - Concurrent access support with proper synchronization
- ✅ **Flexible Resource State Management** - Mark as stale, freshness checking, and cache state inspection

### High Priority Improvements

- ⬜️ **Comprehensive Test Suite** - Unit and integration tests for all core functionality
- ⬜️ **HTTP Request Timeouts** - Configurable timeout handling for remote resource fetching
- 🟧 **Code Consolidation** - Extract and eliminate duplicate code across modules
- 🟧 **Builder Pattern Completion** - Finalize and export the fluent resource creation API
- ⬜️ **Reactive Resource Management** - Observable pattern with file system watching for real-time updates

### Medium Priority Features

- 🟧 **Resource Factory Patterns** - Convenient creation utilities (skeleton exists but needs implementation)
- ⬜️ **Enhanced Documentation** - Comprehensive examples and API reference guides
- 🤔 **Advanced Caching Strategies** - Redis, distributed caching, and cloud-based cache services
- 🤔 **Performance Benchmarking** - Automated benchmarks for performance-critical operations

### New Features Pipeline

- ⬜️ **Local File CRUD Operations** - Create, update, and delete capabilities for local resources
- ⬜️ **Extended Format Support** - TOML, XML, and plain text parsing
- ⬜️ **RESTful API Integration** - Full CRUD support for REST endpoints and generic HTTP services
- ⬜️ **Authentication Framework** - API keys, OAuth, and other security mechanisms for remote resources
- ⬜️ **Secure Protocol Support** - FTP, SFTP, and SSH-based file access
- ⬜️ **Large File Optimization** - Zero-copy processing and reference-based handling for big files

### Future Enhancements

- ⬜️ **Large File Downloads** - Efficient handling of multi-gigabyte file transfers
- 🤔 **Alternative Storage Backends** - Database integration and cloud storage support
- ⬜️ **Compression Support** - Built-in compression and decompression capabilities
- ⬜️ **Binary File Processing** - Native support for binary data formats
- ⬜️ **Advanced Hash Algorithms** - SHA-2, SHA-3, and other cryptographic hash support
- ⬜️ **Stream Processing** - Memory-efficient processing for very large files
