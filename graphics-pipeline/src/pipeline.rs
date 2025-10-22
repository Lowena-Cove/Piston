//! Graphics pipeline state management

use crate::shader::Shader;

/// Graphics pipeline configuration
pub struct PipelineConfig {
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
}

/// Graphics pipeline
pub struct Pipeline {
    config: PipelineConfig,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }
}