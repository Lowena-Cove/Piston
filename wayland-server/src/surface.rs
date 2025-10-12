//! Wayland surface management

use std::sync::{Arc, Mutex};
use wayland_server::{
    Main,
    protocol::wl_surface::WlSurface,
};

use compositor_core::surface::{Surface, SurfaceId};

/// Extended surface handler with Wayland-specific functionality
pub struct WaylandSurface {
    pub id: SurfaceId,
    pub surface: Arc<Mutex<Surface>>,
    pub wayland_surface: Option<Main<WlSurface>>,
    pub pending_buffer: Option<WaylandBuffer>,
    pub current_buffer: Option<WaylandBuffer>,
}

impl WaylandSurface {
    pub fn new(id: SurfaceId, surface: Arc<Mutex<Surface>>) -> Self {
        Self {
            id,
            surface,
            wayland_surface: None,
            pending_buffer: None,
            current_buffer: None,
        }
    }
    
    pub fn set_wayland_surface(&mut self, wayland_surface: Main<WlSurface>) {
        self.wayland_surface = Some(wayland_surface);
    }
    
    pub fn attach_buffer(&mut self, buffer: WaylandBuffer) {
        self.pending_buffer = Some(buffer);
    }
    
    pub fn commit(&mut self) {
        // Move pending buffer to current
        if let Some(buffer) = self.pending_buffer.take() {
            self.current_buffer = Some(buffer);
            // Update the actual surface with buffer data
            self.update_surface_from_buffer();
        }
    }
    
    fn update_surface_from_buffer(&self) {
        if let Some(buffer) = &self.current_buffer {
            // This would copy buffer data to the surface
            log::debug!("Updating surface {} from buffer", self.id);
            // Implementation would depend on buffer type (shm, dmabuf, etc.)
        }
    }
}

/// Wayland buffer representation
pub struct WaylandBuffer {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,
    pub buffer_type: BufferType,
}

pub enum BufferType {
    Shm {
        data: Vec<u8>,
    },
    DmaBuf {
        fd: i32,
        offset: u32,
        modifier: u64,
    },
}

impl WaylandBuffer {
    pub fn new_shm(width: u32, height: u32, stride: u32, format: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            stride,
            format,
            buffer_type: BufferType::Shm { data },
        }
    }
    
    pub fn new_dmabuf(
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
        fd: i32,
        offset: u32,
        modifier: u64,
    ) -> Self {
        Self {
            width,
            height,
            stride,
            format,
            buffer_type: BufferType::DmaBuf { fd, offset, modifier },
        }
    }
}