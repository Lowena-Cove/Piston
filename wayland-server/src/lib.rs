//! Wayland server implementation for the Android compositor
//! 
//! Provides Wayland protocol support for Linux applications running in the compositor

use anyhow::Result;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;

use wayland_server::{
    Display, Client, Resource, GlobalManager, Filter, Main,
    protocol::{
        wl_compositor::{self, WlCompositor},
        wl_surface::{self, WlSurface},
        wl_shell::{self, WlShell},
        wl_shell_surface::{self, WlShellSurface},
        wl_seat::{self, WlSeat},
        wl_pointer::{self, WlPointer},
        wl_keyboard::{self, WlKeyboard},
        wl_output::{self, WlOutput},
    },
};

use compositor_core::{
    Compositor,
    surface::{Surface, SurfaceId, SurfaceFormat},
    window::{Window, WindowId},
    input::{InputEvent, PointerEvent, KeyboardEvent},
};

pub mod compositor;
pub mod surface;
pub mod shell;
pub mod seat;
pub mod output;

use compositor::CompositorHandler;
use surface::SurfaceHandler;
use shell::{ShellHandler, ShellSurfaceHandler};
use seat::{SeatHandler, PointerHandler, KeyboardHandler};
use output::OutputHandler;

/// Wayland server state
pub struct WaylandServer {
    display: Display,
    compositor: Arc<Compositor>,
    surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
    windows: Arc<Mutex<HashMap<WindowId, Arc<Mutex<Window>>>>>,
    clients: Arc<Mutex<HashMap<u32, Client>>>,
    input_sender: mpsc::UnboundedSender<InputEvent>,
}

impl WaylandServer {
    /// Create a new Wayland server
    pub fn new(compositor: Arc<Compositor>) -> Result<Self> {
        let display = Display::new();
        let input_sender = compositor.get_input_sender();
        
        Ok(Self {
            display,
            compositor,
            surfaces: Arc::new(Mutex::new(HashMap::new())),
            windows: Arc::new(Mutex::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            input_sender,
        })
    }
    
    /// Initialize Wayland globals and start the server
    pub async fn start(&mut self, socket_name: Option<String>) -> Result<()> {
        info!("Starting Wayland server");
        
        let mut global_manager = GlobalManager::new(&self.display);
        
        // Register compositor global
        let compositor_handler = CompositorHandler::new(
            Arc::clone(&self.surfaces),
            Arc::clone(&self.compositor),
        );
        global_manager.create_global::<WlCompositor, _>(
            4, // Version
            Filter::new(move |new_compositor: (Main<WlCompositor>, _), _, _| {
                compositor_handler.handle_new_compositor(new_compositor.0);
            }),
        );
        
        // Register shell global
        let shell_handler = ShellHandler::new(
            Arc::clone(&self.windows),
            Arc::clone(&self.surfaces),
        );
        global_manager.create_global::<WlShell, _>(
            1, // Version
            Filter::new(move |new_shell: (Main<WlShell>, _), _, _| {
                shell_handler.handle_new_shell(new_shell.0);
            }),
        );
        
        // Register seat global (for input)
        let seat_handler = SeatHandler::new(self.input_sender.clone());
        global_manager.create_global::<WlSeat, _>(
            7, // Version
            Filter::new(move |new_seat: (Main<WlSeat>, _), _, _| {
                seat_handler.handle_new_seat(new_seat.0);
            }),
        );
        
        // Register output global (for display information)
        let output_handler = OutputHandler::new();
        global_manager.create_global::<WlOutput, _>(
            3, // Version
            Filter::new(move |new_output: (Main<WlOutput>, _), _, _| {
                output_handler.handle_new_output(new_output.0);
            }),
        );
        
        // Set up socket
        let socket_name = socket_name.unwrap_or_else(|| "wayland-android".to_string());
        let socket_path = format!("/data/local/tmp/{}", socket_name);
        
        // Remove existing socket
        let _ = std::fs::remove_file(&socket_path);
        
        // Create Unix domain socket
        let listener = std::os::unix::net::UnixListener::bind(&socket_path)?;
        info!("Wayland server listening on {}", socket_path);
        
        // Set up event loop
        let mut event_loop = calloop::EventLoop::try_new()?;
        let handle = event_loop.handle();
        
        // Add display to event loop
        let display_source = wayland_server::sources::ListeningSocketSource::new(
            listener,
            self.display.clone(),
        )?;
        
        handle.insert_source(display_source, |client_stream, _, _| {
            // Handle new client connections
            info!("New Wayland client connected");
        })?;
        
        // Run event loop
        loop {
            event_loop.dispatch(std::time::Duration::from_millis(16), &mut ())?;
            
            // Flush pending events
            self.display.flush_clients()?;
        }
    }
    
    /// Send input event to Wayland clients
    pub fn send_input_event(&self, event: InputEvent) -> Result<()> {
        // This would route input events to appropriate Wayland clients
        // Implementation depends on which surface/window has focus
        info!("Routing input event: {:?}", event);
        Ok(())
    }
    
    /// Get display handle
    pub fn display(&self) -> &Display {
        &self.display
    }
}

/// Initialize and start the Wayland server
pub async fn start_wayland_server(compositor: Arc<Compositor>) -> Result<()> {
    let mut server = WaylandServer::new(compositor)?;
    server.start(None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use compositor_core::init_compositor;
    
    #[tokio::test]
    async fn test_wayland_server_creation() {
        let compositor = init_compositor().await.unwrap();
        let server = WaylandServer::new(compositor);
        assert!(server.is_ok());
    }
}