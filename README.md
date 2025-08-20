# Stateful

[![Crates.io](https://img.shields.io/crates/v/stateful)](https://crates.io/crates/stateful)
[![Documentation](https://docs.rs/stateful/badge.svg)](https://docs.rs/stateful)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![Build Status](https://github.com/yourusername/stateful/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/stateful/actions)
[![codecov](https://codecov.io/gh/yourusername/stateful/graph/badge.svg?token=YOUR-TOKEN)](https://codecov.io/gh/yourusername/stateful)

A high-performance Rust library for managing both local and remote state with built-in caching, observation, and synchronization capabilities.

## 🚀 Features

- **Local State Management**

  - Thread-safe operations
  - File persistence
  - Atomic writes

- **Remote State Management**

  - Configurable caching strategies
  - Automatic refresh mechanisms
  - Efficient resource handling

- **Observable Pattern**

  - Event-driven architecture
  - Multiple subscriber support
  - Efficient change propagation


- Async/await support

The lib is able to keep specific amount of staled files for remote resources

vision:
support create, update, delete, read REST API
