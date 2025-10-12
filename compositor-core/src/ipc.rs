//! Inter-process communication for the compositor
//! 
//! Handles communication between the Android compositor and Linux applications

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::window::{Window, WindowId};
use crate::surface::{Surface, SurfaceId, SurfaceFormat};
use crate::graphics::GraphicsContext;
use crate::input::InputEvent;

/// IPC message types between compositor and Linux applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Application requests to create a window
    CreateWindow {
        app_id: String,
        title: String,
        width: u32,
        height: u32,
    },
    /// Compositor responds with window ID
    WindowCreated {
        window_id: WindowId,
        shared_memory_id: String,
    },
    /// Application requests to destroy a window
    DestroyWindow {
        window_id: WindowId,
    },
    /// Application updates window properties
    UpdateWindow {
        window_id: WindowId,
        title: Option<String>,
        geometry: Option<(i32, i32, u32, u32)>,
    },
    /// Application commits surface changes
    CommitSurface {
        window_id: WindowId,
        surface_id: SurfaceId,
        damage_regions: Vec<(i32, i32, u32, u32)>,
    },
    /// Compositor sends input events to application
    InputEvent {
        window_id: WindowId,
        event: InputEvent,
    },
    /// Application acknowledges input event
    InputAck {
        window_id: WindowId,
        timestamp: u64,
    },
    /// Error response
    Error {
        message: String,
    },
}

/// IPC client connection representing a Linux application
pub struct IpcClient {
    stream: UnixStream,
    app_id: String,
    windows: Vec<WindowId>,
    input_sender: mpsc::UnboundedSender<InputEvent>,
    input_receiver: mpsc::UnboundedReceiver<InputEvent>,
}

impl IpcClient {
    pub fn new(stream: UnixStream, app_id: String) -> Self {
        let (input_sender, input_receiver) = mpsc::unbounded_channel();
        
        Self {
            stream,
            app_id,
            windows: Vec::new(),
            input_sender,
            input_receiver,
        }
    }
    
    /// Send a message to the client
    pub async fn send_message(&mut self, message: IpcMessage) -> Result<()> {
        let serialized = bincode::serialize(&message)?;
        let len = serialized.len() as u32;
        
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&serialized).await?;
        self.stream.flush().await?;
        
        Ok(())
    }
    
    /// Receive a message from the client
    pub async fn receive_message(&mut self) -> Result<IpcMessage> {
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        let mut buffer = vec![0u8; len];
        self.stream.read_exact(&mut buffer).await?;
        
        let message = bincode::deserialize(&buffer)?;
        Ok(message)
    }
    
    /// Add a window to this client
    pub fn add_window(&mut self, window_id: WindowId) {
        self.windows.push(window_id);
    }
    
    /// Remove a window from this client
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.retain(|&id| id != window_id);
    }
    
    /// Get input event sender
    pub fn get_input_sender(&self) -> mpsc::UnboundedSender<InputEvent> {
        self.input_sender.clone()
    }
}

/// IPC handler managing all client connections
pub struct IpcHandler {
    listener: Option<UnixListener>,
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<IpcClient>>>>>,
    socket_path: String,
}

impl IpcHandler {
    pub async fn new() -> Result<Self> {
        let socket_path = "/data/local/tmp/android_compositor.sock".to_string();
        
        // Remove existing socket file if it exists
        let _ = std::fs::remove_file(&socket_path);
        
        let listener = UnixListener::bind(&socket_path)?;
        log::info!("IPC listener bound to {}", socket_path);
        
        Ok(Self {
            listener: Some(listener),
            clients: Arc::new(Mutex::new(HashMap::new())),
            socket_path,
        })
    }
    
    /// Listen for incoming connections from Linux applications
    pub async fn listen_for_connections(
        &self,
        windows: Arc<Mutex<HashMap<WindowId, Window>>>,
        graphics_context: Arc<GraphicsContext>,
    ) -> Result<()> {
        if let Some(listener) = &self.listener {
            log::info!("Starting IPC listener");
            
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        log::info!("New IPC client connected");
                        
                        let clients = Arc::clone(&self.clients);
                        let windows = Arc::clone(&windows);
                        let graphics_context = Arc::clone(&graphics_context);
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_client(
                                stream,
                                clients,
                                windows,
                                graphics_context,
                            ).await {
                                log::error!("Client handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept IPC connection: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a single client connection
    async fn handle_client(
        stream: UnixStream,
        clients: Arc<Mutex<HashMap<String, Arc<Mutex<IpcClient>>>>>,
        windows: Arc<Mutex<HashMap<WindowId, Window>>>,
        graphics_context: Arc<GraphicsContext>,
    ) -> Result<()> {
        let mut client = IpcClient::new(stream, "unknown".to_string());
        
        loop {
            match client.receive_message().await {
                Ok(message) => {
                    match Self::handle_message(
                        message,
                        &mut client,
                        &windows,
                        &graphics_context,
                    ).await {
                        Ok(Some(response)) => {
                            if let Err(e) = client.send_message(response).await {
                                log::error!("Failed to send response: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {
                            // No response needed
                        }
                        Err(e) => {
                            log::error!("Error handling message: {}", e);
                            let error_response = IpcMessage::Error {
                                message: e.to_string(),
                            };
                            let _ = client.send_message(error_response).await;
                        }
                    }
                }
                Err(e) => {
                    log::info!("Client disconnected: {}", e);
                    break;
                }
            }
        }
        
        // Cleanup client windows
        for window_id in &client.windows {
            let mut windows = windows.lock().unwrap();
            if let Some(window) = windows.remove(window_id) {
                if let Err(e) = window.cleanup().await {
                    log::error!("Error cleaning up window {}: {}", window_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a specific IPC message
    async fn handle_message(
        message: IpcMessage,
        client: &mut IpcClient,
        windows: &Arc<Mutex<HashMap<WindowId, Window>>>,
        graphics_context: &Arc<GraphicsContext>,
    ) -> Result<Option<IpcMessage>> {
        match message {
            IpcMessage::CreateWindow { app_id, title, width, height } => {
                log::info!("Creating window for app: {}", app_id);
                
                // Create window
                let window_id = WindowId::new();
                let mut window = Window::new(window_id, app_id.clone(), title)?;
                
                // Create surface for the window
                let surface = Arc::new(Mutex::new(Surface::new(
                    width,
                    height,
                    SurfaceFormat::RGBA8888,
                )?));
                
                // Initialize GPU resources for the surface
                {
                    let mut surface_guard = surface.lock().unwrap();
                    surface_guard.init_gpu_resources(graphics_context.device())?;
                }
                
                // Set up shared memory
                let shared_memory_id = {
                    let mut surface_guard = surface.lock().unwrap();
                    surface_guard.setup_shared_memory()?
                };
                
                // Attach surface to window
                window.attach_surface(surface);
                
                // Set input sender for the window
                window.set_input_sender(client.get_input_sender());
                
                // Add window to collection
                {
                    let mut windows = windows.lock().unwrap();
                    windows.insert(window_id, window);
                }
                
                // Add window to client
                client.add_window(window_id);
                client.app_id = app_id;
                
                Ok(Some(IpcMessage::WindowCreated {
                    window_id,
                    shared_memory_id,
                }))
            }
            
            IpcMessage::DestroyWindow { window_id } => {
                log::info!("Destroying window: {}", window_id);
                
                let mut windows = windows.lock().unwrap();
                if let Some(window) = windows.remove(&window_id) {
                    drop(windows); // Release lock before async operation
                    window.cleanup().await?;
                }
                
                client.remove_window(window_id);
                Ok(None)
            }
            
            IpcMessage::UpdateWindow { window_id, title, geometry } => {
                let windows = windows.lock().unwrap();
                if let Some(window) = windows.get(&window_id) {
                    if let Some(title) = title {
                        window.set_title(title);
                    }
                    if let Some((x, y, width, height)) = geometry {
                        window.set_geometry(crate::window::WindowGeometry::new(x, y, width, height));
                    }
                }
                Ok(None)
            }
            
            IpcMessage::CommitSurface { window_id, surface_id: _, damage_regions } => {
                let windows = windows.lock().unwrap();
                if let Some(window) = windows.get(&window_id) {
                    if let Some(surface) = window.get_surface() {
                        // Add damage regions
                        for (x, y, width, height) in damage_regions {
                            surface.lock().unwrap().add_damage(crate::surface::SurfaceDamage {
                                x, y, width, height,
                            });
                        }
                        
                        // Update surface from shared memory
                        drop(windows); // Release lock before async operation
                        let surface_guard = surface.lock().unwrap();
                        drop(surface_guard); // Release surface lock
                        
                        // This would normally update the surface content
                        // surface.update_from_shared_memory(graphics_context.device(), graphics_context.queue()).await?;
                    }
                }
                Ok(None)
            }
            
            IpcMessage::InputAck { window_id: _, timestamp: _ } => {
                // Input acknowledgment - could be used for input flow control
                Ok(None)
            }
            
            _ => {
                log::warn!("Unhandled IPC message: {:?}", message);
                Ok(None)
            }
        }
    }
    
    /// Send input event to a specific window
    pub async fn send_input_event(&self, window_id: WindowId, event: InputEvent) -> Result<()> {
        // This would route the input event to the appropriate client
        // For now, we'll just log it
        log::debug!("Sending input event to window {}: {:?}", window_id, event);
        Ok(())
    }
}

impl Drop for IpcHandler {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipc_message_serialization() {
        let message = IpcMessage::CreateWindow {
            app_id: "test.app".to_string(),
            title: "Test Window".to_string(),
            width: 800,
            height: 600,
        };
        
        let serialized = bincode::serialize(&message).unwrap();
        let deserialized: IpcMessage = bincode::deserialize(&serialized).unwrap();
        
        match deserialized {
            IpcMessage::CreateWindow { app_id, title, width, height } => {
                assert_eq!(app_id, "test.app");
                assert_eq!(title, "Test Window");
                assert_eq!(width, 800);
                assert_eq!(height, 600);
            }
            _ => panic!("Wrong message type"),
        }
    }
}