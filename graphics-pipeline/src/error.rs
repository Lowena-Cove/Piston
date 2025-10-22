//! Error types for the graphics pipeline

use thiserror::Error;

/// Result type for graphics operations
pub type GraphicsResult<T> = Result<T, GraphicsError>;

/// Errors that can occur in graphics operations
#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Backend initialization failed: {0}")]
    BackendInit(String),
    
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
    
    #[error("Invalid surface ID: {0}")]
    InvalidSurface(usize),
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    
    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),
    
    #[error("Rendering failed: {0}")]
    Rendering(String),
    
    #[error("Resource allocation failed: {0}")]
    ResourceAllocation(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Backend error: {0}")]
    Backend(String),
}