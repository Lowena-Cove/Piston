//! Error types for the SteamOS emulator

use thiserror::Error;

/// Result type alias for emulator operations
pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Main error type for the emulator
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Memory allocation error: {0}")]
    Memory(String),
    
    #[error("CPU execution error: {0}")]
    Cpu(String),
    
    #[error("Invalid instruction: {opcode:#x} at address {address:#x}")]
    InvalidInstruction { opcode: u8, address: u64 },
    
    #[error("Memory access violation: attempted to access {address:#x}")]
    MemoryAccessViolation { address: u64 },
    
    #[error("System call error: {syscall} ({errno})")]
    SystemCall { syscall: String, errno: i32 },
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Image format error: {0}")]
    ImageFormat(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Device error: {0}")]
    Device(String),
    
    #[error("Init process not found in any standard location")]
    InitNotFound,
    
    #[error("Emulator not initialized")]
    NotInitialized,
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

impl EmulatorError {
    /// Create a memory error
    pub fn memory<S: Into<String>>(msg: S) -> Self {
        Self::Memory(msg.into())
    }
    
    /// Create a CPU error
    pub fn cpu<S: Into<String>>(msg: S) -> Self {
        Self::Cpu(msg.into())
    }
    
    /// Create a file system error
    pub fn filesystem<S: Into<String>>(msg: S) -> Self {
        Self::FileSystem(msg.into())
    }
    
    /// Create a process error
    pub fn process<S: Into<String>>(msg: S) -> Self {
        Self::Process(msg.into())
    }
    
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create an image format error
    pub fn image_format<S: Into<String>>(msg: S) -> Self {
        Self::ImageFormat(msg.into())
    }
    
    /// Create a device error
    pub fn device<S: Into<String>>(msg: S) -> Self {
        Self::Device(msg.into())
    }
    
    /// Create a not supported error
    pub fn not_supported<S: Into<String>>(msg: S) -> Self {
        Self::NotSupported(msg.into())
    }
}

/// Convert from nix errors
impl From<nix::Error> for EmulatorError {
    fn from(err: nix::Error) -> Self {
        Self::SystemCall {
            syscall: "nix operation".to_string(),
            errno: err as i32,
        }
    }
}

/// Convert from TOML deserialization errors
impl From<toml::de::Error> for EmulatorError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(format!("TOML parse error: {}", err))
    }
}

/// Convert from anyhow errors
impl From<anyhow::Error> for EmulatorError {
    fn from(err: anyhow::Error) -> Self {
        Self::Parse(format!("Parse error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = EmulatorError::memory("Out of memory");
        assert!(matches!(err, EmulatorError::Memory(_)));
        
        let err = EmulatorError::InvalidInstruction { opcode: 0xFF, address: 0x1000 };
        assert!(err.to_string().contains("0xff"));
        assert!(err.to_string().contains("0x1000"));
    }
    
    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let emu_err: EmulatorError = io_err.into();
        assert!(matches!(emu_err, EmulatorError::Io(_)));
    }
}