use std::{fmt, io};

/// Error type for registry operations.
///
/// All fallible registry operations return this error type to indicate
/// what went wrong during the operation.
#[derive(Debug)]
pub enum ResourceError {
    /// Failed to acquire the cache lock.
    ///
    /// This occurs when the internal cache lock cannot be acquired,
    /// typically due to lock poisoning or contention.
    CacheLock,

    /// No fresh or stale data found in the cache.
    ///
    /// This indicates that the requested data is not available in either
    /// the fresh or stale cache state.
    StaleInternalNone,

    /// Unable to refresh data from the source.
    ///
    /// This occurs when an attempt to fetch fresh data fails,
    /// preventing the cache from being updated.
    UnableToFreshData,

    /// Failed to deserialize data.
    ///
    /// This occurs when data cannot be deserialized from the specified format.
    /// The string contains the format type (e.g., "JSON", "YAML").
    Deserialization(String),

    /// Failed to serialize data.
    ///
    /// This occurs when data cannot be serialized to the specified format.
    /// The string contains the format type (e.g., "JSON", "YAML").
    Serialization(String),

    /// IO operation failed.
    ///
    /// This wraps standard IO errors that occur during file operations.
    Io(io::Error),

    /// Unsupported file type encountered.
    ///
    /// This occurs when attempting to process a file with an unsupported format.
    /// The string contains the file type that was attempted.
    UnsupportedFileType(String),

    /// Incorrect target path name.
    ///
    /// This occurs when a file path cannot be extracted or is malformed.
    IncorrectTargetPathName,

    /// Invalid unicode encoding in filename.
    ///
    /// This occurs when a filename contains invalid UTF-8 sequences.
    InvalidUnicodeEncoding,

    /// Missing timestamp separator in filename.
    ///
    /// This occurs when parsing a timestamped filename that lacks
    /// the expected separator character.
    MissingTimestampSeparator,

    /// Missing timestamp file extension.
    ///
    /// This occurs when parsing a timestamped filename that lacks
    /// a file extension after the timestamp.
    MissingTimestampExtension,

    /// Failed to parse timestamp value.
    ///
    /// This occurs when a timestamp string cannot be parsed as a valid number.
    TimestampParseError,
}

/// Helper constructors for common error patterns.
impl ResourceError {
    /// Creates a deserialization error for the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format type that failed to deserialize (e.g., "JSON", "YAML")
    pub fn deserialization(format: &str) -> ResourceError {
        ResourceError::Deserialization(format.to_string())
    }

    /// Creates a serialization error for the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format type that failed to serialize (e.g., "JSON", "YAML")
    pub fn serialization(format: &str) -> ResourceError {
        ResourceError::Serialization(format.to_string())
    }

    /// Creates an unsupported file type error.
    ///
    /// # Arguments
    ///
    /// * `file_type` - The file type that is not supported
    pub fn unsupported_file_type(file_type: &str) -> ResourceError {
        ResourceError::UnsupportedFileType(file_type.to_string())
    }

    /// Creates an IO error from a standard IO error.
    ///
    /// # Arguments
    ///
    /// * `error` - The underlying IO error
    pub fn io(error: io::Error) -> ResourceError {
        ResourceError::Io(error)
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::CacheLock => write!(f, "Failed to acquire cache lock"),
            ResourceError::StaleInternalNone => {
                write!(f, "No fresh or stale data found")
            }
            ResourceError::UnableToFreshData => {
                write!(f, "Unable to refresh data")
            }
            ResourceError::Deserialization(format) => {
                write!(f, "Failed to deserialize {} data", format)
            }
            ResourceError::Serialization(format) => {
                write!(f, "Failed to serialize {} data", format)
            }
            ResourceError::Io(e) => write!(f, "IO error: {}", e),
            ResourceError::UnsupportedFileType(file_type) => {
                write!(f, "Unsupported file type: {}", file_type)
            }
            ResourceError::IncorrectTargetPathName => {
                write!(f, "Incorrect target path name")
            }
            ResourceError::InvalidUnicodeEncoding => {
                write!(f, "Invalid unicode encoding")
            }
            ResourceError::MissingTimestampSeparator => {
                write!(f, "Invalid filename format: Missing timestamp separator")
            }
            ResourceError::MissingTimestampExtension => {
                write!(f, "Invalid filename format: Missing timestamp extension")
            }
            ResourceError::TimestampParseError => {
                write!(f, "Failed to parse timestamp")
            }
        }
    }
}

impl std::error::Error for ResourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_format() {
        let err = ResourceError::StaleInternalNone;
        assert!(format!("{:?}", err).contains("StaleInternalNone"));
    }

    #[test]
    fn test_pattern_matching() {
        // Test that variants can be matched correctly
        assert!(matches!(ResourceError::CacheLock, ResourceError::CacheLock));
        assert!(!matches!(
            ResourceError::CacheLock,
            ResourceError::StaleInternalNone
        ));

        // Test matching with data-carrying variants
        let err = ResourceError::Deserialization("JSON".to_string());
        assert!(matches!(err, ResourceError::Deserialization(_)));

        // Test that we can extract values from variants
        if let ResourceError::Deserialization(format) = err {
            assert_eq!(format, "JSON");
        } else {
            panic!("Expected Deserialization variant");
        }
    }

    #[test]
    fn test_error_trait() {
        let err_concrete = ResourceError::UnableToFreshData;
        let err_trait: &dyn std::error::Error = &ResourceError::UnableToFreshData;
        assert_eq!(err_trait.to_string(), err_concrete.to_string());
    }
}
