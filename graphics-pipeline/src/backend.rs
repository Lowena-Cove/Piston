//! Graphics backend abstraction

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::{GraphicsError, GraphicsFeature};

/// Types of graphics backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Vulkan backend
    Vulkan,
    
    /// OpenGL ES backend
    OpenGLES,
    
    /// Software rendering backend
    Software,
    
    /// Auto-detect best available backend
    Auto,
}

/// Information about a graphics backend
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub backend_type: BackendType,
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub device_name: String,
    pub supported_features: Vec<GraphicsFeature>,
}

/// Trait for graphics backends
#[async_trait]
pub trait GraphicsBackend: Send + Sync {
    /// Get backend information
    fn get_info(&self) -> BackendInfo;
    
    /// Check if a feature is supported
    fn supports_feature(&self, feature: GraphicsFeature) -> bool;
    
    /// Initialize the backend
    async fn initialize(&mut self) -> Result<()>;
    
    /// Shutdown the backend
    async fn shutdown(&mut self) -> Result<()>;
}

/// Create a graphics backend of the specified type
pub async fn create_backend(backend_type: BackendType) -> Result<Arc<dyn GraphicsBackend>> {
    match backend_type {
        BackendType::Vulkan => {
            #[cfg(feature = "vulkan")]
            {
                let mut backend = VulkanBackend::new()?;
                backend.initialize().await?;
                Ok(Arc::new(backend))
            }
            #[cfg(not(feature = "vulkan"))]
            {
                Err(GraphicsError::UnsupportedFeature("Vulkan not compiled in".to_string()).into())
            }
        }
        BackendType::OpenGLES => {
            #[cfg(feature = "opengl")]
            {
                let mut backend = OpenGLESBackend::new()?;
                backend.initialize().await?;
                Ok(Arc::new(backend))
            }
            #[cfg(not(feature = "opengl"))]
            {
                Err(GraphicsError::UnsupportedFeature("OpenGL ES not compiled in".to_string()).into())
            }
        }
        BackendType::Software => {
            let mut backend = SoftwareBackend::new()?;
            backend.initialize().await?;
            Ok(Arc::new(backend))
        }
        BackendType::Auto => {
            // Try backends in order of preference
            if let Ok(backend) = create_backend(BackendType::Vulkan).await {
                return Ok(backend);
            }
            if let Ok(backend) = create_backend(BackendType::OpenGLES).await {
                return Ok(backend);
            }
            create_backend(BackendType::Software).await
        }
    }
}

/// Software rendering backend (always available)
pub struct SoftwareBackend {
    info: BackendInfo,
}

impl SoftwareBackend {
    pub fn new() -> Result<Self> {
        let info = BackendInfo {
            backend_type: BackendType::Software,
            name: "Software Renderer".to_string(),
            version: "1.0".to_string(),
            vendor: "Piston Team".to_string(),
            device_name: "CPU".to_string(),
            supported_features: vec![
                // Software backend supports basic features
            ],
        };
        
        Ok(Self { info })
    }
}

#[async_trait]
impl GraphicsBackend for SoftwareBackend {
    fn get_info(&self) -> BackendInfo {
        self.info.clone()
    }
    
    fn supports_feature(&self, _feature: GraphicsFeature) -> bool {
        // Software backend has limited feature support
        false
    }
    
    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing software graphics backend");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        log::info!("Shutting down software graphics backend");
        Ok(())
    }
}

/// Vulkan backend (optional)
#[cfg(feature = "vulkan")]
pub struct VulkanBackend {
    info: BackendInfo,
}

#[cfg(feature = "vulkan")]
impl VulkanBackend {
    pub fn new() -> Result<Self> {
        // TODO: Initialize Vulkan instance and detect capabilities
        let info = BackendInfo {
            backend_type: BackendType::Vulkan,
            name: "Vulkan".to_string(),
            version: "1.3".to_string(),
            vendor: "Khronos".to_string(),
            device_name: "Unknown".to_string(),
            supported_features: vec![
                GraphicsFeature::HardwareAcceleration,
                GraphicsFeature::Vulkan,
                GraphicsFeature::ComputeShaders,
                GraphicsFeature::MSAA,
                GraphicsFeature::AnisotropicFiltering,
            ],
        };
        
        Ok(Self { info })
    }
}

#[cfg(feature = "vulkan")]
#[async_trait]
impl GraphicsBackend for VulkanBackend {
    fn get_info(&self) -> BackendInfo {
        self.info.clone()
    }
    
    fn supports_feature(&self, feature: GraphicsFeature) -> bool {
        self.info.supported_features.contains(&feature)
    }
    
    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing Vulkan graphics backend");
        // TODO: Implement Vulkan initialization
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        log::info!("Shutting down Vulkan graphics backend");
        // TODO: Implement Vulkan cleanup
        Ok(())
    }
}

/// OpenGL ES backend (optional)
#[cfg(feature = "opengl")]
pub struct OpenGLESBackend {
    info: BackendInfo,
}

#[cfg(feature = "opengl")]
impl OpenGLESBackend {
    pub fn new() -> Result<Self> {
        // TODO: Initialize OpenGL ES context and detect capabilities
        let info = BackendInfo {
            backend_type: BackendType::OpenGLES,
            name: "OpenGL ES".to_string(),
            version: "3.2".to_string(),
            vendor: "Khronos".to_string(),
            device_name: "Unknown".to_string(),
            supported_features: vec![
                GraphicsFeature::HardwareAcceleration,
                GraphicsFeature::OpenGLES,
                GraphicsFeature::MSAA,
                GraphicsFeature::AnisotropicFiltering,
            ],
        };
        
        Ok(Self { info })
    }
}

#[cfg(feature = "opengl")]
#[async_trait]
impl GraphicsBackend for OpenGLESBackend {
    fn get_info(&self) -> BackendInfo {
        self.info.clone()
    }
    
    fn supports_feature(&self, feature: GraphicsFeature) -> bool {
        self.info.supported_features.contains(&feature)
    }
    
    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing OpenGL ES graphics backend");
        // TODO: Implement OpenGL ES initialization
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        log::info!("Shutting down OpenGL ES graphics backend");
        // TODO: Implement OpenGL ES cleanup
        Ok(())
    }
}