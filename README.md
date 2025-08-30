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
// Get data or return error
match local.get_data_or_error(false).await {
    Ok(DataResult::Fresh(data)) => println!("Fresh data: {:?}", data),
    Ok(DataResult::Stale(data)) => println!("Stale data: {:?}", data),
    Err(e) => eprintln!("Error: {}", e),
}

// Get data or default
let data = remote.get_data_or_default(false).await;
println!("Data: {:?}", data);
```

### Marking Data as Stale

```rust
// Force refresh on next read
local.mark_stale();
```

## Advanced Features

### Custom Parsing

You can implement custom parsing logic by implementing the `ReadOnlyResource` trait for your types.

### Error Handling

All operations return `Result` types with descriptive error messages for better error handling.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## TODO

## Future Improvements

- ✅ Improve error types and error handling
- ⬜️ Add comprehensive unit tests
- ⬜️ Add timeout handling for HTTP requests
- 🟧 Extract duplicate code to utilities
- ⬜️ Add observable pattern + watching for changes to allow reactive event driven work
- ⬜️ Add resource factory and builder patterns for more convenient creation
- ⬜️ Expand documentation with more examples and API references
- 🤔 Add more caching strategies
- 🤔 Add benchmarks for performance-critical paths

### New Features Planned

- ⬜️ Support CRUD on local files
- ⬜️ Support for more file formats (plain text, XML, TOML)
- ⬜️ Add RESTful API support (CRUD endpoints on remote) + non-RESTful (generic HTTP service for CRUD operations on remote)
- ⬜️ Explore authentication methods and security process flows for remote resources (e.g. API keys, OAuth, etc.)

### Nice to Have

- ⬜️ Add feature for other cache implementations (e.g. Redis, etc.)
- ⬜️ Add feature for other storage backends (e.g. database, etc.)
- ⬜️ Add feature to process zip (compress/decompress)
- ⬜️ Add feature to process binary
- ⬜️ Add feature for FTP, SFTP access
- ⬜️ Add feature for other hash algorithms besides md5 (sha2, etc.)
