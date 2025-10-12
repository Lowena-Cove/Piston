//! Wayland shell protocol handlers

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use wayland_server::{
    Main, Filter,
    protocol::{
        wl_shell::{self, WlShell},
        wl_shell_surface::{self, WlShellSurface},
        wl_surface::WlSurface,
    },
};
use log::{info, debug};

use compositor_core::{
    surface::{Surface, SurfaceId},
    window::{Window, WindowId, WindowGeometry},
};

/// Handler for wl_shell protocol
pub struct ShellHandler {
    windows: Arc<Mutex<HashMap<WindowId, Arc<Mutex<Window>>>>>,
    surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
}

impl ShellHandler {
    pub fn new(
        windows: Arc<Mutex<HashMap<WindowId, Arc<Mutex<Window>>>>>,
        surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
    ) -> Self {
        Self {
            windows,
            surfaces,
        }
    }
    
    pub fn handle_new_shell(&self, shell: Main<WlShell>) {
        info!("New wl_shell created");
        
        let windows = Arc::clone(&self.windows);
        let surfaces = Arc::clone(&self.surfaces);
        
        shell.as_ref().assign(Filter::new(move |event, _shell| {
            match event {
                wl_shell::Event::GetShellSurface { id, surface } => {
                    debug!("Creating shell surface");
                    
                    let shell_surface_handler = ShellSurfaceHandler::new(
                        Arc::clone(&windows),
                        Arc::clone(&surfaces),
                        surface,
                    );
                    
                    id.as_ref().assign(Filter::new(move |event, shell_surface| {
                        shell_surface_handler.handle_shell_surface_event(event, shell_surface);
                    }));
                }
            }
        }));
    }
}

/// Handler for wl_shell_surface protocol
pub struct ShellSurfaceHandler {
    windows: Arc<Mutex<HashMap<WindowId, Arc<Mutex<Window>>>>>,
    surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
    surface: Main<WlSurface>,
    window_id: Option<WindowId>,
}

impl ShellSurfaceHandler {
    pub fn new(
        windows: Arc<Mutex<HashMap<WindowId, Arc<Mutex<Window>>>>>,
        surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
        surface: Main<WlSurface>,
    ) -> Self {
        Self {
            windows,
            surfaces,
            surface,
            window_id: None,
        }
    }
    
    pub fn handle_shell_surface_event(
        &mut self,
        event: wl_shell_surface::Event,
        shell_surface: Main<WlShellSurface>,
    ) {
        match event {
            wl_shell_surface::Event::Pong { serial: _ } => {
                debug!("Shell surface pong");
            }
            
            wl_shell_surface::Event::Move { seat: _, serial: _ } => {
                debug!("Shell surface move request");
                // Handle window move
            }
            
            wl_shell_surface::Event::Resize { seat: _, serial: _, edges: _ } => {
                debug!("Shell surface resize request");
                // Handle window resize
            }
            
            wl_shell_surface::Event::SetToplevel => {
                debug!("Shell surface set toplevel");
                
                // Create a new window for this shell surface
                let window_id = WindowId::new();
                let window = Arc::new(Mutex::new(
                    Window::new(window_id, "wayland-app".to_string(), "Wayland Window".to_string())
                        .unwrap()
                ));
                
                // Store window
                {
                    let mut windows = self.windows.lock().unwrap();
                    windows.insert(window_id, Arc::clone(&window));
                }
                
                self.window_id = Some(window_id);
                
                // Send configure event
                shell_surface.configure(wl_shell_surface::Resize::None, 800, 600);
            }
            
            wl_shell_surface::Event::SetTransient { parent: _, x, y, flags: _ } => {
                debug!("Shell surface set transient at ({}, {})", x, y);
                
                if let Some(window_id) = self.window_id {
                    let windows = self.windows.lock().unwrap();
                    if let Some(window) = windows.get(&window_id) {
                        let window = window.lock().unwrap();
                        window.set_geometry(WindowGeometry::new(x, y, 400, 300));
                    }
                }
            }
            
            wl_shell_surface::Event::SetFullscreen { method: _, framerate: _, output: _ } => {
                debug!("Shell surface set fullscreen");
                
                if let Some(window_id) = self.window_id {
                    let windows = self.windows.lock().unwrap();
                    if let Some(window) = windows.get(&window_id) {
                        let window = window.lock().unwrap();
                        window.set_fullscreen(true);
                    }
                }
                
                // Send configure for fullscreen
                shell_surface.configure(wl_shell_surface::Resize::None, 1920, 1080);
            }
            
            wl_shell_surface::Event::SetPopup { 
                seat: _, 
                serial: _, 
                parent: _, 
                x, 
                y, 
                flags: _ 
            } => {
                debug!("Shell surface set popup at ({}, {})", x, y);
                
                // Create popup window
                let window_id = WindowId::new();
                let window = Arc::new(Mutex::new(
                    Window::new(window_id, "wayland-popup".to_string(), "Popup".to_string())
                        .unwrap()
                ));
                
                {
                    let window_guard = window.lock().unwrap();
                    window_guard.set_geometry(WindowGeometry::new(x, y, 200, 100));
                }
                
                {
                    let mut windows = self.windows.lock().unwrap();
                    windows.insert(window_id, window);
                }
                
                self.window_id = Some(window_id);
            }
            
            wl_shell_surface::Event::SetMaximized { output: _ } => {
                debug!("Shell surface set maximized");
                
                if let Some(window_id) = self.window_id {
                    let windows = self.windows.lock().unwrap();
                    if let Some(window) = windows.get(&window_id) {
                        let window = window.lock().unwrap();
                        window.maximize();
                    }
                }
                
                // Send configure for maximized
                shell_surface.configure(wl_shell_surface::Resize::None, 1920, 1080);
            }
            
            wl_shell_surface::Event::SetTitle { title } => {
                debug!("Shell surface set title: {}", title);
                
                if let Some(window_id) = self.window_id {
                    let windows = self.windows.lock().unwrap();
                    if let Some(window) = windows.get(&window_id) {
                        let window = window.lock().unwrap();
                        window.set_title(title);
                    }
                }
            }
            
            wl_shell_surface::Event::SetClass { class_ } => {
                debug!("Shell surface set class: {}", class_);
                
                if let Some(window_id) = self.window_id {
                    let windows = self.windows.lock().unwrap();
                    if let Some(window) = windows.get(&window_id) {
                        let window = window.lock().unwrap();
                        window.set_property("class".to_string(), class_);
                    }
                }
            }
        }
    }
}