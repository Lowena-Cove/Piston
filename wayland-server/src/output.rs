//! Wayland output protocol handler

use wayland_server::{
    Main, Filter,
    protocol::{
        wl_output::{self, WlOutput, Subpixel, Transform, Mode},
    },
};
use log::{info, debug};

/// Handler for wl_output protocol
pub struct OutputHandler {
    width: i32,
    height: i32,
    refresh: i32,
}

impl OutputHandler {
    pub fn new() -> Self {
        Self {
            width: 1920,  // Default Android display width
            height: 1080, // Default Android display height
            refresh: 60000, // 60 Hz in mHz
        }
    }
    
    pub fn new_with_dimensions(width: i32, height: i32, refresh: i32) -> Self {
        Self {
            width,
            height,
            refresh,
        }
    }
    
    pub fn handle_new_output(&self, output: Main<WlOutput>) {
        info!("New wl_output created");
        
        // Send output geometry
        output.geometry(
            0, // x
            0, // y
            300, // physical_width (mm) - approximate for typical phone
            200, // physical_height (mm)
            Subpixel::Unknown,
            "Android".to_string(), // make
            "Compositor".to_string(), // model
            Transform::Normal,
        );
        
        // Send output mode
        output.mode(
            Mode::Current | Mode::Preferred,
            self.width,
            self.height,
            self.refresh,
        );
        
        // Send scale (for HiDPI displays)
        output.scale(2); // Typical Android phone scale
        
        // Send done event
        output.done();
        
        let width = self.width;
        let height = self.height;
        let refresh = self.refresh;
        
        output.as_ref().assign(Filter::new(move |event, _output| {
            match event {
                wl_output::Event::Release => {
                    debug!("Output released");
                }
            }
        }));
    }
    
    /// Update output dimensions (e.g., on screen rotation)
    pub fn update_dimensions(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
    
    /// Get current output dimensions
    pub fn get_dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}