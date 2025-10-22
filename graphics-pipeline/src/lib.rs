//! Graphics Pipeline for SteamOS Emulator
//! 
//! This crate provides graphics rendering and display capabilities for the
//! SteamOS emulator, supporting multiple backends including Vulkan, OpenGL ES,
//! and software rendering.

pub mod backend;
pub mod surface;
pub mod renderer;
pub mod buffer;
pub mod texture;
pub mod shader;
pub mod pipeline;
pub mod error;

use anyhow::Result;
use log::{info, warn, error};
use std::sync::Arc;

pub use error::{GraphicsError, GraphicsResult};
pub use backend::{GraphicsBackend, BackendType};
pub use surface::{Surface, SurfaceConfig};
pub use renderer::Renderer;

/// Main graphics pipeline manager
pub struct GraphicsPipeline {
    backend: Arc<dyn GraphicsBackend>,
    renderer: Renderer,
    surfaces: Vec<Surface>,
}

impl GraphicsPipeline {
    /// Create a new graphics pipeline with the specified backend
    pub async fn new(backend_type: BackendType) -> Result<Self> {
        info!("Initializing graphics pipeline with backend: {:?}", backend_type);
        
        let backend = backend::create_backend(backend_type).await?;
        let renderer = Renderer::new(backend.clone()).await?;
        
        Ok(Self {
            backend,
            renderer,
            surfaces: Vec::new(),
        })
    }
    
    /// Create a new surface for rendering
    pub async fn create_surface(&mut self, config: SurfaceConfig) -> Result<usize> {
        let surface = Surface::new(self.backend.clone(), config).await?;
        let surface_id = self.surfaces.len();
        self.surfaces.push(surface);
        
        info!("Created surface with ID: {}", surface_id);
        Ok(surface_id)
    }
    
    /// Render a frame to the specified surface
    pub async fn render_frame(&mut self, surface_id: usize) -> Result<()> {
        if surface_id >= self.surfaces.len() {
            return Err(GraphicsError::InvalidSurface(surface_id).into());
        }
        
        let surface = &mut self.surfaces[surface_id];
        self.renderer.render_to_surface(surface).await?;
        
        Ok(())
    }
    
    /// Get information about the graphics backend
    pub fn get_backend_info(&self) -> backend::BackendInfo {
        self.backend.get_info()
    }
    
    /// Check if a specific feature is supported
    pub fn is_feature_supported(&self, feature: GraphicsFeature) -> bool {
        self.backend.supports_feature(feature)
    }
}

/// Graphics features that may be supported by different backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsFeature {
    /// Hardware acceleration
    HardwareAcceleration,
    
    /// Vulkan API support
    Vulkan,
    
    /// OpenGL ES support
    OpenGLES,
    
    /// Compute shaders
    ComputeShaders,
    
    /// Multi-sampling anti-aliasing
    MSAA,
    
    /// Anisotropic filtering
    AnisotropicFiltering,
    
    /// HDR rendering
    HDR,
    
    /// Variable rate shading
    VariableRateShading,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_graphics_pipeline_creation() {
        // Test software backend creation (should always work)
        let pipeline = GraphicsPipeline::new(BackendType::Software).await;
        assert!(pipeline.is_ok());
    }
    
    #[test]
    fn test_graphics_feature_enum() {
        let feature = GraphicsFeature::HardwareAcceleration;
        assert_eq!(feature, GraphicsFeature::HardwareAcceleration);
    }
}