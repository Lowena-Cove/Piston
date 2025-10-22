//! Buffer management for graphics resources

/// Graphics buffer types
pub enum BufferType {
    Vertex,
    Index,
    Uniform,
    Storage,
}

/// Graphics buffer
pub struct Buffer {
    buffer_type: BufferType,
    size: usize,
    data: Vec<u8>,
}

impl Buffer {
    pub fn new(buffer_type: BufferType, size: usize) -> Self {
        Self {
            buffer_type,
            size,
            data: vec![0u8; size],
        }
    }
}