//! Error handling utilities for Android JNI bridge
//! 
//! This module provides error handling and result types that can be safely
//! passed between Rust and Java through JNI.

use anyhow::Error;
use log::error;

/// Result type for Android operations
pub type AndroidResult<T> = Result<T, AndroidError>;

/// Error types that can occur in Android operations
#[derive(Debug, thiserror::Error)]
pub enum AndroidError {
    #[error("System capability detection failed: {0}")]
    SystemDetection(String),
    
    #[error("ISO file validation failed: {0}")]
    IsoValidation(String),
    
    #[error("Emulator initialization failed: {0}")]
    EmulatorInit(String),
    
    #[error("Emulator runtime error: {0}")]
    EmulatorRuntime(String),
    
    #[error("JNI operation failed: {0}")]
    Jni(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Resource allocation failed: {0}")]
    ResourceAllocation(String),
    
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AndroidError {
    /// Convert to error code for JNI return values
    pub fn to_error_code(&self) -> i32 {
        match self {
            AndroidError::SystemDetection(_) => -1,
            AndroidError::IsoValidation(_) => -2,
            AndroidError::EmulatorInit(_) => -3,
            AndroidError::EmulatorRuntime(_) => -4,
            AndroidError::Jni(_) => -5,
            AndroidError::FileSystem(_) => -6,
            AndroidError::Configuration(_) => -7,
            AndroidError::ResourceAllocation(_) => -8,
            AndroidError::Unsupported(_) => -9,
            AndroidError::Internal(_) => -10,
        }
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AndroidError::SystemDetection(_) => {
                "Failed to detect system capabilities. Your device may not be supported.".to_string()
            }
            AndroidError::IsoValidation(_) => {
                "The selected ISO file is invalid or corrupted. Please select a valid SteamOS recovery image.".to_string()
            }
            AndroidError::EmulatorInit(_) => {
                "Failed to initialize the emulator. Please check that your device meets the minimum requirements.".to_string()
            }
            AndroidError::EmulatorRuntime(_) => {
                "The emulator encountered an error during execution. This may be due to insufficient resources or hardware limitations.".to_string()
            }
            AndroidError::Jni(_) => {
                "Internal communication error. Please restart the application.".to_string()
            }
            AndroidError::FileSystem(_) => {
                "File system error. Please check that the ISO file is accessible and you have sufficient storage space.".to_string()
            }
            AndroidError::Configuration(_) => {
                "Configuration error. Please check your settings and try again.".to_string()
            }
            AndroidError::ResourceAllocation(_) => {
                "Insufficient system resources. Please close other applications and try again.".to_string()
            }
            AndroidError::Unsupported(_) => {
                "This operation is not supported on your device.".to_string()
            }
            AndroidError::Internal(_) => {
                "An internal error occurred. Please report this issue if it persists.".to_string()
            }
        }
    }
}

impl From<anyhow::Error> for AndroidError {
    fn from(error: anyhow::Error) -> Self {
        let error_str = error.to_string();
        
        // Try to categorize the error based on its message
        if error_str.contains("ISO") || error_str.contains("image") {
            AndroidError::IsoValidation(error_str)
        } else if error_str.contains("system") || error_str.contains("capability") {
            AndroidError::SystemDetection(error_str)
        } else if error_str.contains("emulator") || error_str.contains("virtualization") {
            AndroidError::EmulatorRuntime(error_str)
        } else if error_str.contains("file") || error_str.contains("directory") {
            AndroidError::FileSystem(error_str)
        } else if error_str.contains("memory") || error_str.contains("allocation") {
            AndroidError::ResourceAllocation(error_str)
        } else {
            AndroidError::Internal(error_str)
        }
    }
}

impl From<std::io::Error> for AndroidError {
    fn from(error: std::io::Error) -> Self {
        AndroidError::FileSystem(error.to_string())
    }
}

/// Log an error and return its error code
pub fn log_and_return_error(error: &AndroidError) -> i32 {
    error!("Android operation failed: {}", error);
    error.to_error_code()
}

/// Convert a Result to an error code, logging any errors
pub fn result_to_error_code<T>(result: AndroidResult<T>) -> i32 {
    match result {
        Ok(_) => 0, // Success
        Err(error) => log_and_return_error(&error),
    }
}

/// Error recovery strategies
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    
    /// Fall back to a simpler mode
    Fallback,
    
    /// Abort the operation
    Abort,
    
    /// Ask user for input
    UserInput,
}

impl AndroidError {
    /// Get suggested recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            AndroidError::SystemDetection(_) => RecoveryStrategy::Fallback,
            AndroidError::IsoValidation(_) => RecoveryStrategy::UserInput,
            AndroidError::EmulatorInit(_) => RecoveryStrategy::Fallback,
            AndroidError::EmulatorRuntime(_) => RecoveryStrategy::Retry,
            AndroidError::Jni(_) => RecoveryStrategy::Abort,
            AndroidError::FileSystem(_) => RecoveryStrategy::UserInput,
            AndroidError::Configuration(_) => RecoveryStrategy::UserInput,
            AndroidError::ResourceAllocation(_) => RecoveryStrategy::Retry,
            AndroidError::Unsupported(_) => RecoveryStrategy::Abort,
            AndroidError::Internal(_) => RecoveryStrategy::Abort,
        }
    }
}

/// Utility for building error context
pub struct ErrorContext {
    operation: String,
    details: Vec<String>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            details: Vec::new(),
        }
    }
    
    pub fn add_detail(mut self, detail: &str) -> Self {
        self.details.push(detail.to_string());
        self
    }
    
    pub fn build_error(self, error_type: AndroidError) -> AndroidError {
        let context = if self.details.is_empty() {
            format!("Operation: {}", self.operation)
        } else {
            format!("Operation: {}, Details: {}", self.operation, self.details.join(", "))
        };
        
        match error_type {
            AndroidError::SystemDetection(msg) => AndroidError::SystemDetection(format!("{} ({})", msg, context)),
            AndroidError::IsoValidation(msg) => AndroidError::IsoValidation(format!("{} ({})", msg, context)),
            AndroidError::EmulatorInit(msg) => AndroidError::EmulatorInit(format!("{} ({})", msg, context)),
            AndroidError::EmulatorRuntime(msg) => AndroidError::EmulatorRuntime(format!("{} ({})", msg, context)),
            AndroidError::Jni(msg) => AndroidError::Jni(format!("{} ({})", msg, context)),
            AndroidError::FileSystem(msg) => AndroidError::FileSystem(format!("{} ({})", msg, context)),
            AndroidError::Configuration(msg) => AndroidError::Configuration(format!("{} ({})", msg, context)),
            AndroidError::ResourceAllocation(msg) => AndroidError::ResourceAllocation(format!("{} ({})", msg, context)),
            AndroidError::Unsupported(msg) => AndroidError::Unsupported(format!("{} ({})", msg, context)),
            AndroidError::Internal(msg) => AndroidError::Internal(format!("{} ({})", msg, context)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_codes() {
        assert_eq!(AndroidError::SystemDetection("test".to_string()).to_error_code(), -1);
        assert_eq!(AndroidError::IsoValidation("test".to_string()).to_error_code(), -2);
        assert_eq!(AndroidError::EmulatorInit("test".to_string()).to_error_code(), -3);
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .add_detail("detail1")
            .add_detail("detail2");
        
        let error = context.build_error(AndroidError::Internal("test error".to_string()));
        
        match error {
            AndroidError::Internal(msg) => {
                assert!(msg.contains("test_operation"));
                assert!(msg.contains("detail1"));
                assert!(msg.contains("detail2"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
    
    #[test]
    fn test_recovery_strategies() {
        assert!(matches!(
            AndroidError::SystemDetection("test".to_string()).recovery_strategy(),
            RecoveryStrategy::Fallback
        ));
        
        assert!(matches!(
            AndroidError::IsoValidation("test".to_string()).recovery_strategy(),
            RecoveryStrategy::UserInput
        ));
    }
}