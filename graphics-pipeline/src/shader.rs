//! Shader management

/// Shader types
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

/// Shader resource
pub struct Shader {
    shader_type: ShaderType,
    source: String,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source: String) -> Self {
        Self {
            shader_type,
            source,
        }
    }
}