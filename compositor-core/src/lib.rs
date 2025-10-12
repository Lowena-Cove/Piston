//! Android Compositor Core
//! 
//! This crate provides the core compositor functionality for bridging
//! Linux/SteamOS applications to Android display surfaces.

use anyhow::Result;
use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub mod surface;
pub mod window;
pub mod input;
pub mod graphics;
pub mod ipc;

use surface::Surface;
use window::{Window, WindowId};
use input::InputEvent;

/// Main compositor instance that manages all windows and surfaces
pub struct Compositor {
    /// Collection of managed windows
    windows: Arc<Mutex<HashMap<WindowId, Window>>>,
    /// Graphics context for rendering
    graphics_context: Arc<graphics::GraphicsContext>,
    /// Input event channel
    input_sender: mpsc::UnboundedSender<InputEvent>,
    input_receiver: Arc<Mutex<mpsc::UnboundedReceiver<InputEvent>>>,
    /// IPC handler for communication with Linux processes
    ipc_handler: Arc<ipc::IpcHandler>,
}

impl Compositor {
    /// Create a new compositor instance
    pub async fn new() -> Result<Self> {
        info!("Initializing Android Compositor");
        
        let graphics_context = Arc::new(graphics::GraphicsContext::new().await?);
        let (input_sender, input_receiver) = mpsc::unbounded_channel();
        let ipc_handler = Arc::new(ipc::IpcHandler::new().await?);
        
        Ok(Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            graphics_context,
            input_sender,
            input_receiver: Arc::new(Mutex::new(input_receiver)),
            ipc_handler,
        })
    }
    
    /// Start the compositor main loop
    pub async fn run(&self) -> Result<()> {
        info!("Starting compositor main loop");
        
        // Start IPC listener for Linux applications
        let ipc_handler = Arc::clone(&self.ipc_handler);
        let windows = Arc::clone(&self.windows);
        let graphics_context = Arc::clone(&self.graphics_context);
        
        tokio::spawn(async move {
            if let Err(e) = ipc_handler.listen_for_connections(windows, graphics_context).await {
                error!("IPC handler error: {}", e);
            }
        });
        
        // Main compositor loop
        loop {
            // Process input events
            if let Ok(mut receiver) = self.input_receiver.try_lock() {
                while let Ok(event) = receiver.try_recv() {
                    self.handle_input_event(event).await?;
                }
            }
            
            // Render all windows
            self.render_frame().await?;
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60 FPS
        }
    }
    
    /// Handle input events from Android
    async fn handle_input_event(&self, event: InputEvent) -> Result<()> {
        // Route input events to appropriate Linux applications
        if let Ok(windows) = self.windows.lock() {
            for window in windows.values() {
                if window.contains_point(event.position()) {
                    window.send_input_event(event.clone()).await?;
                    break;
                }
            }
        }
        Ok(())
    }
    
    /// Render a frame with all visible windows
    async fn render_frame(&self) -> Result<()> {
        let windows = self.windows.lock().unwrap();
        let mut surfaces_to_render = Vec::new();
        
        // Collect surfaces from all windows
        for window in windows.values() {
            if let Some(surface) = window.get_surface() {
                surfaces_to_render.push(surface);
            }
        }
        
        // Render all surfaces to the Android display
        self.graphics_context.render_surfaces(&surfaces_to_render).await?;
        
        Ok(())
    }
    
    /// Create a new window for a Linux application
    pub async fn create_window(&self, app_id: String, title: String) -> Result<WindowId> {
        let window_id = WindowId::new();
        let window = Window::new(window_id, app_id, title)?;
        
        {
            let mut windows = self.windows.lock().unwrap();
            windows.insert(window_id, window);
        }
        
        info!("Created window {} for application", window_id);
        Ok(window_id)
    }
    
    /// Remove a window
    pub async fn destroy_window(&self, window_id: WindowId) -> Result<()> {
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.remove(&window_id) {
            window.cleanup().await?;
            info!("Destroyed window {}", window_id);
        } else {
            warn!("Attempted to destroy non-existent window {}", window_id);
        }
        Ok(())
    }
    
    /// Get input event sender for Android integration
    pub fn get_input_sender(&self) -> mpsc::UnboundedSender<InputEvent> {
        self.input_sender.clone()
    }
}

/// Initialize the compositor system
pub async fn init_compositor() -> Result<Arc<Compositor>> {
    env_logger::init();
    info!("Initializing compositor system");
    
    let compositor = Compositor::new().await?;
    Ok(Arc::new(compositor))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_compositor_creation() {
        let compositor = Compositor::new().await;
        assert!(compositor.is_ok());
    }
}