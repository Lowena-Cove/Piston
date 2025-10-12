//! Window management for the compositor
//! 
//! Handles individual application windows and their properties

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::surface::{Surface, SurfaceId};
use crate::input::InputEvent;

/// Unique identifier for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(u64);

impl WindowId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({})", self.0)
    }
}

/// Window geometry and positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl WindowGeometry {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x 
            && x < self.x + self.width as i32
            && y >= self.y 
            && y < self.y + self.height as i32
    }
}

/// Window state and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub geometry: WindowGeometry,
    pub title: String,
    pub app_id: String,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizable: bool,
    pub decorations: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            geometry: WindowGeometry::new(0, 0, 800, 600),
            title: String::new(),
            app_id: String::new(),
            visible: true,
            focused: false,
            minimized: false,
            maximized: false,
            fullscreen: false,
            resizable: true,
            decorations: true,
        }
    }
}

/// Window role/type for proper handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowRole {
    Normal,
    Dialog,
    Menu,
    Tooltip,
    Notification,
    Splash,
    Utility,
}

/// A window managed by the compositor
pub struct Window {
    pub id: WindowId,
    pub state: Arc<Mutex<WindowState>>,
    pub role: WindowRole,
    
    /// Primary surface for this window
    surface: Option<Arc<Mutex<Surface>>>,
    
    /// Additional surfaces (e.g., for subsurfaces)
    subsurfaces: Vec<Arc<Mutex<Surface>>>,
    
    /// Input event sender to Linux application
    input_sender: Option<mpsc::UnboundedSender<InputEvent>>,
    
    /// Window-specific properties
    properties: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Window {
    /// Create a new window
    pub fn new(id: WindowId, app_id: String, title: String) -> Result<Self> {
        let mut state = WindowState::default();
        state.app_id = app_id;
        state.title = title;
        
        Ok(Self {
            id,
            state: Arc::new(Mutex::new(state)),
            role: WindowRole::Normal,
            surface: None,
            subsurfaces: Vec::new(),
            input_sender: None,
            properties: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
    }
    
    /// Attach a surface to this window
    pub fn attach_surface(&mut self, surface: Arc<Mutex<Surface>>) {
        self.surface = Some(surface);
    }
    
    /// Add a subsurface to this window
    pub fn add_subsurface(&mut self, surface: Arc<Mutex<Surface>>) {
        self.subsurfaces.push(surface);
    }
    
    /// Remove a subsurface
    pub fn remove_subsurface(&mut self, surface_id: SurfaceId) {
        self.subsurfaces.retain(|s| {
            s.lock().unwrap().id != surface_id
        });
    }
    
    /// Get the primary surface
    pub fn get_surface(&self) -> Option<Arc<Mutex<Surface>>> {
        self.surface.clone()
    }
    
    /// Get all surfaces (primary + subsurfaces)
    pub fn get_all_surfaces(&self) -> Vec<Arc<Mutex<Surface>>> {
        let mut surfaces = Vec::new();
        
        if let Some(surface) = &self.surface {
            surfaces.push(surface.clone());
        }
        
        surfaces.extend(self.subsurfaces.iter().cloned());
        surfaces
    }
    
    /// Set input event sender for this window
    pub fn set_input_sender(&mut self, sender: mpsc::UnboundedSender<InputEvent>) {
        self.input_sender = Some(sender);
    }
    
    /// Send input event to the Linux application
    pub async fn send_input_event(&self, event: InputEvent) -> Result<()> {
        if let Some(sender) = &self.input_sender {
            sender.send(event)?;
        }
        Ok(())
    }
    
    /// Check if point is within window bounds
    pub fn contains_point(&self, point: (i32, i32)) -> bool {
        let state = self.state.lock().unwrap();
        state.geometry.contains_point(point.0, point.1)
    }
    
    /// Update window geometry
    pub fn set_geometry(&self, geometry: WindowGeometry) {
        let mut state = self.state.lock().unwrap();
        state.geometry = geometry;
    }
    
    /// Update window title
    pub fn set_title(&self, title: String) {
        let mut state = self.state.lock().unwrap();
        state.title = title;
    }
    
    /// Set window visibility
    pub fn set_visible(&self, visible: bool) {
        let mut state = self.state.lock().unwrap();
        state.visible = visible;
    }
    
    /// Set window focus state
    pub fn set_focused(&self, focused: bool) {
        let mut state = self.state.lock().unwrap();
        state.focused = focused;
    }
    
    /// Minimize window
    pub fn minimize(&self) {
        let mut state = self.state.lock().unwrap();
        state.minimized = true;
        state.visible = false;
    }
    
    /// Restore window from minimized state
    pub fn restore(&self) {
        let mut state = self.state.lock().unwrap();
        state.minimized = false;
        state.visible = true;
    }
    
    /// Maximize window
    pub fn maximize(&self) {
        let mut state = self.state.lock().unwrap();
        state.maximized = true;
        // TODO: Set geometry to screen size
    }
    
    /// Unmaximize window
    pub fn unmaximize(&self) {
        let mut state = self.state.lock().unwrap();
        state.maximized = false;
        // TODO: Restore previous geometry
    }
    
    /// Set fullscreen mode
    pub fn set_fullscreen(&self, fullscreen: bool) {
        let mut state = self.state.lock().unwrap();
        state.fullscreen = fullscreen;
        // TODO: Handle fullscreen geometry changes
    }
    
    /// Set window property
    pub fn set_property(&self, key: String, value: String) {
        let mut properties = self.properties.lock().unwrap();
        properties.insert(key, value);
    }
    
    /// Get window property
    pub fn get_property(&self, key: &str) -> Option<String> {
        let properties = self.properties.lock().unwrap();
        properties.get(key).cloned()
    }
    
    /// Get current window state
    pub fn get_state(&self) -> WindowState {
        self.state.lock().unwrap().clone()
    }
    
    /// Update window state with a closure
    pub fn update_state<F>(&self, updater: F) 
    where
        F: FnOnce(&mut WindowState),
    {
        let mut state = self.state.lock().unwrap();
        updater(&mut *state);
    }
    
    /// Check if window needs redraw
    pub fn needs_redraw(&self) -> bool {
        if let Some(surface) = &self.surface {
            if surface.lock().unwrap().needs_redraw() {
                return true;
            }
        }
        
        for subsurface in &self.subsurfaces {
            if subsurface.lock().unwrap().needs_redraw() {
                return true;
            }
        }
        
        false
    }
    
    /// Cleanup window resources
    pub async fn cleanup(&self) -> Result<()> {
        log::info!("Cleaning up window {}", self.id);
        
        // Close input channel
        if let Some(sender) = &self.input_sender {
            // Channel will be closed when sender is dropped
        }
        
        // Surfaces will be cleaned up when dropped
        Ok(())
    }
}

/// Window manager for organizing and managing multiple windows
pub struct WindowManager {
    windows: Arc<Mutex<std::collections::HashMap<WindowId, Arc<Mutex<Window>>>>>,
    focused_window: Arc<Mutex<Option<WindowId>>>,
    window_stack: Arc<Mutex<Vec<WindowId>>>, // Z-order stack
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(std::collections::HashMap::new())),
            focused_window: Arc::new(Mutex::new(None)),
            window_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Add a window to the manager
    pub fn add_window(&self, window: Arc<Mutex<Window>>) {
        let window_id = window.lock().unwrap().id;
        
        {
            let mut windows = self.windows.lock().unwrap();
            windows.insert(window_id, window);
        }
        
        {
            let mut stack = self.window_stack.lock().unwrap();
            stack.push(window_id);
        }
        
        log::info!("Added window {} to manager", window_id);
    }
    
    /// Remove a window from the manager
    pub fn remove_window(&self, window_id: WindowId) -> Option<Arc<Mutex<Window>>> {
        let window = {
            let mut windows = self.windows.lock().unwrap();
            windows.remove(&window_id)
        };
        
        {
            let mut stack = self.window_stack.lock().unwrap();
            stack.retain(|&id| id != window_id);
        }
        
        {
            let mut focused = self.focused_window.lock().unwrap();
            if *focused == Some(window_id) {
                *focused = None;
            }
        }
        
        if window.is_some() {
            log::info!("Removed window {} from manager", window_id);
        }
        
        window
    }
    
    /// Get a window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<Arc<Mutex<Window>>> {
        let windows = self.windows.lock().unwrap();
        windows.get(&window_id).cloned()
    }
    
    /// Get all windows in Z-order (bottom to top)
    pub fn get_windows_in_order(&self) -> Vec<Arc<Mutex<Window>>> {
        let windows = self.windows.lock().unwrap();
        let stack = self.window_stack.lock().unwrap();
        
        stack.iter()
            .filter_map(|&id| windows.get(&id).cloned())
            .collect()
    }
    
    /// Set focused window
    pub fn set_focused_window(&self, window_id: Option<WindowId>) {
        let mut focused = self.focused_window.lock().unwrap();
        
        // Update focus state of previous window
        if let Some(prev_id) = *focused {
            if let Some(window) = self.get_window(prev_id) {
                window.lock().unwrap().set_focused(false);
            }
        }
        
        // Update focus state of new window
        if let Some(new_id) = window_id {
            if let Some(window) = self.get_window(new_id) {
                window.lock().unwrap().set_focused(true);
                self.bring_to_front(new_id);
            }
        }
        
        *focused = window_id;
    }
    
    /// Bring window to front of Z-order
    pub fn bring_to_front(&self, window_id: WindowId) {
        let mut stack = self.window_stack.lock().unwrap();
        stack.retain(|&id| id != window_id);
        stack.push(window_id);
    }
    
    /// Get focused window
    pub fn get_focused_window(&self) -> Option<WindowId> {
        *self.focused_window.lock().unwrap()
    }
}