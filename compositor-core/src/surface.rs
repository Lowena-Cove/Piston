//! Surface management for compositor
//! 
//! Handles graphics surfaces that contain rendered content from Linux applications

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use wgpu::{Buffer, Texture, TextureView};

/// Unique identifier for surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurfaceId(u64);

impl SurfaceId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for SurfaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Surface({})", self.0)
    }
}

/// Surface damage information for partial updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceDamage {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Surface transformation matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub matrix: [[f32; 4]; 4],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// Surface state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceState {
    pub width: u32,
    pub height: u32,
    pub format: SurfaceFormat,
    pub transform: Transform,
    pub opacity: f32,
    pub visible: bool,
}

/// Supported surface pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceFormat {
    RGBA8888,
    RGBX8888,
    RGB565,
    ARGB8888,
}

impl SurfaceFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            SurfaceFormat::RGBA8888 | SurfaceFormat::RGBX8888 | SurfaceFormat::ARGB8888 => 4,
            SurfaceFormat::RGB565 => 2,
        }
    }
    
    pub fn to_wgpu_format(&self) -> wgpu::TextureFormat {
        match self {
            SurfaceFormat::RGBA8888 => wgpu::TextureFormat::Rgba8Unorm,
            SurfaceFormat::RGBX8888 => wgpu::TextureFormat::Rgba8Unorm,
            SurfaceFormat::ARGB8888 => wgpu::TextureFormat::Bgra8Unorm,
            SurfaceFormat::RGB565 => wgpu::TextureFormat::R5g6b5Unorm,
        }
    }
}

/// A graphics surface containing rendered content from a Linux application
pub struct Surface {
    pub id: SurfaceId,
    pub state: Arc<Mutex<SurfaceState>>,
    
    // GPU resources
    texture: Option<Texture>,
    texture_view: Option<TextureView>,
    staging_buffer: Option<Buffer>,
    
    // Shared memory for efficient data transfer
    shared_memory: Option<Arc<shared_memory::Shmem>>,
    
    // Damage tracking for partial updates
    damage_regions: Arc<Mutex<Vec<SurfaceDamage>>>,
}

impl Surface {
    /// Create a new surface with the given dimensions and format
    pub fn new(width: u32, height: u32, format: SurfaceFormat) -> Result<Self> {
        let id = SurfaceId::new();
        let state = SurfaceState {
            width,
            height,
            format,
            transform: Transform::default(),
            opacity: 1.0,
            visible: true,
        };
        
        Ok(Self {
            id,
            state: Arc::new(Mutex::new(state)),
            texture: None,
            texture_view: None,
            staging_buffer: None,
            shared_memory: None,
            damage_regions: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Initialize GPU resources for this surface
    pub fn init_gpu_resources(&mut self, device: &wgpu::Device) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        let texture_desc = wgpu::TextureDescriptor {
            label: Some(&format!("Surface {} Texture", self.id)),
            size: wgpu::Extent3d {
                width: state.width,
                height: state.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: state.format.to_wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create staging buffer for CPU->GPU transfers
        let buffer_size = (state.width * state.height * state.format.bytes_per_pixel()) as u64;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Surface {} Staging Buffer", self.id)),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });
        
        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.staging_buffer = Some(staging_buffer);
        
        Ok(())
    }
    
    /// Set up shared memory for efficient data transfer from Linux applications
    pub fn setup_shared_memory(&mut self) -> Result<String> {
        let state = self.state.lock().unwrap();
        let size = (state.width * state.height * state.format.bytes_per_pixel()) as usize;
        
        let shmem = shared_memory::ShmemConf::new()
            .size(size)
            .create()?;
        
        let shmem_id = format!("surface_{}", self.id.0);
        self.shared_memory = Some(Arc::new(shmem));
        
        Ok(shmem_id)
    }
    
    /// Update surface content from shared memory
    pub async fn update_from_shared_memory(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        if let Some(shared_mem) = &self.shared_memory {
            if let Some(staging_buffer) = &self.staging_buffer {
                if let Some(texture) = &self.texture {
                    let state = self.state.lock().unwrap();
                    
                    // Map staging buffer and copy from shared memory
                    let buffer_slice = staging_buffer.slice(..);
                    let (sender, receiver) = futures::channel::oneshot::channel();
                    buffer_slice.map_async(wgpu::MapMode::Write, move |result| {
                        sender.send(result).unwrap();
                    });
                    
                    device.poll(wgpu::Maintain::Wait);
                    receiver.await??;
                    
                    {
                        let mut buffer_view = buffer_slice.get_mapped_range_mut();
                        let shared_data = unsafe { shared_mem.as_slice() };
                        buffer_view.copy_from_slice(shared_data);
                    }
                    
                    staging_buffer.unmap();
                    
                    // Copy from staging buffer to texture
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Surface Update Encoder"),
                    });
                    
                    encoder.copy_buffer_to_texture(
                        wgpu::ImageCopyBuffer {
                            buffer: staging_buffer,
                            layout: wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(state.width * state.format.bytes_per_pixel()),
                                rows_per_image: Some(state.height),
                            },
                        },
                        wgpu::ImageCopyTexture {
                            texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: state.width,
                            height: state.height,
                            depth_or_array_layers: 1,
                        },
                    );
                    
                    queue.submit(std::iter::once(encoder.finish()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Add damage region for partial updates
    pub fn add_damage(&self, damage: SurfaceDamage) {
        let mut damage_regions = self.damage_regions.lock().unwrap();
        damage_regions.push(damage);
    }
    
    /// Get and clear damage regions
    pub fn take_damage(&self) -> Vec<SurfaceDamage> {
        let mut damage_regions = self.damage_regions.lock().unwrap();
        std::mem::take(&mut *damage_regions)
    }
    
    /// Get the texture view for rendering
    pub fn get_texture_view(&self) -> Option<&TextureView> {
        self.texture_view.as_ref()
    }
    
    /// Update surface state
    pub fn update_state<F>(&self, updater: F) 
    where
        F: FnOnce(&mut SurfaceState),
    {
        let mut state = self.state.lock().unwrap();
        updater(&mut *state);
    }
    
    /// Get current surface state
    pub fn get_state(&self) -> SurfaceState {
        self.state.lock().unwrap().clone()
    }
    
    /// Check if surface needs redraw
    pub fn needs_redraw(&self) -> bool {
        !self.damage_regions.lock().unwrap().is_empty()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        log::debug!("Dropping surface {}", self.id);
    }
}