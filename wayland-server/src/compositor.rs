//! Wayland compositor protocol handler

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use wayland_server::{
    Main, Filter,
    protocol::{
        wl_compositor::{self, WlCompositor},
        wl_surface::{self, WlSurface},
    },
};
use log::{info, debug};

use compositor_core::{
    Compositor,
    surface::{Surface, SurfaceId, SurfaceFormat},
};

/// Handler for wl_compositor protocol
pub struct CompositorHandler {
    surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
    compositor: Arc<Compositor>,
}

impl CompositorHandler {
    pub fn new(
        surfaces: Arc<Mutex<HashMap<SurfaceId, Arc<Mutex<Surface>>>>>,
        compositor: Arc<Compositor>,
    ) -> Self {
        Self {
            surfaces,
            compositor,
        }
    }
    
    pub fn handle_new_compositor(&self, compositor: Main<WlCompositor>) {
        info!("New wl_compositor created");
        
        let surfaces = Arc::clone(&self.surfaces);
        let comp = Arc::clone(&self.compositor);
        
        compositor.as_ref().assign(Filter::new(move |event, compositor| {
            match event {
                wl_compositor::Event::CreateSurface { id } => {
                    debug!("Creating new surface");
                    
                    let surface_id = SurfaceId::new();
                    let surface = Arc::new(Mutex::new(
                        Surface::new(800, 600, SurfaceFormat::RGBA8888).unwrap()
                    ));
                    
                    // Store surface
                    {
                        let mut surfaces = surfaces.lock().unwrap();
                        surfaces.insert(surface_id, Arc::clone(&surface));
                    }
                    
                    // Set up surface handlers
                    let surface_handler = SurfaceHandler::new(surface_id, Arc::clone(&surface));
                    id.as_ref().assign(Filter::new(move |event, surface| {
                        surface_handler.handle_surface_event(event, surface);
                    }));
                }
                
                wl_compositor::Event::CreateRegion { id: _ } => {
                    debug!("Creating new region (not implemented)");
                    // Region creation would be implemented here
                }
            }
        }));
    }
}

/// Handler for wl_surface protocol
pub struct SurfaceHandler {
    surface_id: SurfaceId,
    surface: Arc<Mutex<Surface>>,
}

impl SurfaceHandler {
    pub fn new(surface_id: SurfaceId, surface: Arc<Mutex<Surface>>) -> Self {
        Self {
            surface_id,
            surface,
        }
    }
    
    pub fn handle_surface_event(&self, event: wl_surface::Event, _surface: Main<WlSurface>) {
        match event {
            wl_surface::Event::Destroy => {
                debug!("Surface {} destroyed", self.surface_id);
            }
            
            wl_surface::Event::Attach { buffer, x, y } => {
                debug!("Surface {} attach buffer at ({}, {})", self.surface_id, x, y);
                // Buffer attachment would be handled here
                if let Some(_buffer) = buffer {
                    // Process buffer attachment
                }
            }
            
            wl_surface::Event::Damage { x, y, width, height } => {
                debug!("Surface {} damage: ({}, {}) {}x{}", self.surface_id, x, y, width, height);
                
                let surface = self.surface.lock().unwrap();
                surface.add_damage(compositor_core::surface::SurfaceDamage {
                    x, y, width: width as u32, height: height as u32,
                });
            }
            
            wl_surface::Event::Frame { callback } => {
                debug!("Surface {} frame callback", self.surface_id);
                // Frame callback would be handled here
                callback.done(0); // Send frame done immediately for now
            }
            
            wl_surface::Event::SetOpaqueRegion { region: _ } => {
                debug!("Surface {} set opaque region", self.surface_id);
                // Opaque region handling
            }
            
            wl_surface::Event::SetInputRegion { region: _ } => {
                debug!("Surface {} set input region", self.surface_id);
                // Input region handling
            }
            
            wl_surface::Event::Commit => {
                debug!("Surface {} commit", self.surface_id);
                // Surface commit - apply pending state
                // This would trigger a redraw
            }
            
            wl_surface::Event::SetBufferTransform { transform: _ } => {
                debug!("Surface {} set buffer transform", self.surface_id);
                // Buffer transform handling
            }
            
            wl_surface::Event::SetBufferScale { scale: _ } => {
                debug!("Surface {} set buffer scale", self.surface_id);
                // Buffer scale handling
            }
            
            wl_surface::Event::DamageBuffer { x, y, width, height } => {
                debug!("Surface {} damage buffer: ({}, {}) {}x{}", self.surface_id, x, y, width, height);
                
                let surface = self.surface.lock().unwrap();
                surface.add_damage(compositor_core::surface::SurfaceDamage {
                    x, y, width: width as u32, height: height as u32,
                });
            }
        }
    }
}