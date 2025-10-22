//! Main renderer implementation

use anyhow::Result;
use std::sync::Arc;
use crate::{backend::GraphicsBackend, surface::Surface};

/// Main renderer that handles drawing operations
pub struct Renderer {
    backend: Arc<dyn GraphicsBackend>,
}

impl Renderer {
    pub async fn new(backend: Arc<dyn GraphicsBackend>) -> Result<Self> {
        Ok(Self { backend })
    }
    
    pub async fn render_to_surface(&mut self, _surface: &mut Surface) -> Result<()> {
        // TODO: Implement actual rendering
        Ok(())
    }
}