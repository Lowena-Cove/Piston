//! Input event handling for the compositor
//! 
//! Manages input events from Android and routes them to Linux applications

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Input event types supported by the compositor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Touch/pointer events
    Pointer(PointerEvent),
    /// Keyboard events
    Keyboard(KeyboardEvent),
    /// Scroll/wheel events
    Scroll(ScrollEvent),
    /// Gesture events
    Gesture(GestureEvent),
}

impl InputEvent {
    /// Get the position of the input event if applicable
    pub fn position(&self) -> Option<(i32, i32)> {
        match self {
            InputEvent::Pointer(event) => Some((event.x, event.y)),
            InputEvent::Scroll(event) => Some((event.x, event.y)),
            InputEvent::Gesture(event) => Some((event.x, event.y)),
            InputEvent::Keyboard(_) => None,
        }
    }
    
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> u64 {
        match self {
            InputEvent::Pointer(event) => event.timestamp,
            InputEvent::Keyboard(event) => event.timestamp,
            InputEvent::Scroll(event) => event.timestamp,
            InputEvent::Gesture(event) => event.timestamp,
        }
    }
}

/// Pointer/touch event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerEvent {
    pub timestamp: u64,
    pub x: i32,
    pub y: i32,
    pub event_type: PointerEventType,
    pub button: Option<PointerButton>,
    pub pointer_id: u32, // For multi-touch support
    pub pressure: f32,   // Touch pressure (0.0 - 1.0)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerEventType {
    Down,
    Up,
    Move,
    Enter,
    Leave,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// Keyboard event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub timestamp: u64,
    pub event_type: KeyboardEventType,
    pub key_code: u32,
    pub scan_code: u32,
    pub modifiers: KeyboardModifiers,
    pub text: Option<String>, // For text input
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyboardEventType {
    KeyDown,
    KeyUp,
    KeyRepeat,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyboardModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Windows/Cmd key
    pub caps_lock: bool,
    pub num_lock: bool,
}

/// Scroll/wheel event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub timestamp: u64,
    pub x: i32,
    pub y: i32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub scroll_type: ScrollType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollType {
    Wheel,      // Mouse wheel
    Finger,     // Touch scroll
    Continuous, // Trackpad
}

/// Gesture event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEvent {
    pub timestamp: u64,
    pub x: i32,
    pub y: i32,
    pub gesture_type: GestureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureType {
    Pinch { scale: f32, rotation: f32 },
    Swipe { direction: SwipeDirection, velocity: f32 },
    Tap { finger_count: u32 },
    LongPress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Input event builder for creating events from Android input
pub struct InputEventBuilder;

impl InputEventBuilder {
    /// Create a pointer event from Android MotionEvent
    pub fn pointer_event(
        action: i32,
        x: f32,
        y: f32,
        pointer_id: i32,
        pressure: f32,
        timestamp: i64,
    ) -> InputEvent {
        let event_type = match action {
            0 => PointerEventType::Down,   // ACTION_DOWN
            1 => PointerEventType::Up,     // ACTION_UP
            2 => PointerEventType::Move,   // ACTION_MOVE
            3 => PointerEventType::Cancel, // ACTION_CANCEL
            _ => PointerEventType::Move,
        };
        
        InputEvent::Pointer(PointerEvent {
            timestamp: timestamp as u64,
            x: x as i32,
            y: y as i32,
            event_type,
            button: None, // Touch events don't have buttons
            pointer_id: pointer_id as u32,
            pressure,
        })
    }
    
    /// Create a keyboard event from Android KeyEvent
    pub fn keyboard_event(
        action: i32,
        key_code: i32,
        scan_code: i32,
        meta_state: i32,
        timestamp: i64,
    ) -> InputEvent {
        let event_type = match action {
            0 => KeyboardEventType::KeyDown,   // ACTION_DOWN
            1 => KeyboardEventType::KeyUp,     // ACTION_UP
            2 => KeyboardEventType::KeyRepeat, // ACTION_MULTIPLE
            _ => KeyboardEventType::KeyDown,
        };
        
        let modifiers = KeyboardModifiers {
            ctrl: (meta_state & 0x1000) != 0,  // META_CTRL_ON
            alt: (meta_state & 0x02) != 0,     // META_ALT_ON
            shift: (meta_state & 0x01) != 0,   // META_SHIFT_ON
            meta: (meta_state & 0x10000) != 0, // META_META_ON
            caps_lock: (meta_state & 0x100000) != 0, // META_CAPS_LOCK_ON
            num_lock: (meta_state & 0x200000) != 0,  // META_NUM_LOCK_ON
        };
        
        InputEvent::Keyboard(KeyboardEvent {
            timestamp: timestamp as u64,
            event_type,
            key_code: key_code as u32,
            scan_code: scan_code as u32,
            modifiers,
            text: None,
        })
    }
    
    /// Create a scroll event
    pub fn scroll_event(
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        timestamp: i64,
    ) -> InputEvent {
        InputEvent::Scroll(ScrollEvent {
            timestamp: timestamp as u64,
            x: x as i32,
            y: y as i32,
            delta_x,
            delta_y,
            scroll_type: ScrollType::Finger, // Assume touch scroll from Android
        })
    }
    
    /// Get current timestamp in milliseconds
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Input state tracker for managing input device states
pub struct InputState {
    /// Currently pressed pointer buttons
    pressed_buttons: HashMap<u32, PointerButton>,
    /// Current pointer positions
    pointer_positions: HashMap<u32, (i32, i32)>,
    /// Currently pressed keys
    pressed_keys: HashMap<u32, bool>,
    /// Current keyboard modifiers
    keyboard_modifiers: KeyboardModifiers,
    /// Last input timestamp
    last_timestamp: u64,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_buttons: HashMap::new(),
            pointer_positions: HashMap::new(),
            pressed_keys: HashMap::new(),
            keyboard_modifiers: KeyboardModifiers::default(),
            last_timestamp: 0,
        }
    }
    
    /// Update state with new input event
    pub fn update(&mut self, event: &InputEvent) {
        self.last_timestamp = event.timestamp();
        
        match event {
            InputEvent::Pointer(pointer_event) => {
                self.pointer_positions.insert(
                    pointer_event.pointer_id,
                    (pointer_event.x, pointer_event.y),
                );
                
                match pointer_event.event_type {
                    PointerEventType::Down => {
                        if let Some(button) = &pointer_event.button {
                            self.pressed_buttons.insert(pointer_event.pointer_id, button.clone());
                        }
                    }
                    PointerEventType::Up | PointerEventType::Cancel => {
                        self.pressed_buttons.remove(&pointer_event.pointer_id);
                    }
                    _ => {}
                }
            }
            InputEvent::Keyboard(keyboard_event) => {
                self.keyboard_modifiers = keyboard_event.modifiers.clone();
                
                match keyboard_event.event_type {
                    KeyboardEventType::KeyDown => {
                        self.pressed_keys.insert(keyboard_event.key_code, true);
                    }
                    KeyboardEventType::KeyUp => {
                        self.pressed_keys.remove(&keyboard_event.key_code);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.pressed_keys.contains_key(&key_code)
    }
    
    /// Check if a pointer button is currently pressed
    pub fn is_button_pressed(&self, pointer_id: u32) -> bool {
        self.pressed_buttons.contains_key(&pointer_id)
    }
    
    /// Get current pointer position
    pub fn get_pointer_position(&self, pointer_id: u32) -> Option<(i32, i32)> {
        self.pointer_positions.get(&pointer_id).copied()
    }
    
    /// Get current keyboard modifiers
    pub fn get_modifiers(&self) -> &KeyboardModifiers {
        &self.keyboard_modifiers
    }
}

/// Input event router for distributing events to appropriate handlers
pub struct InputRouter {
    /// Event handlers for different input types
    handlers: HashMap<String, Box<dyn InputHandler + Send + Sync>>,
}

impl InputRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register an input handler
    pub fn register_handler(&mut self, name: String, handler: Box<dyn InputHandler + Send + Sync>) {
        self.handlers.insert(name, handler);
    }
    
    /// Route an input event to appropriate handlers
    pub async fn route_event(&self, event: InputEvent) -> Result<()> {
        for handler in self.handlers.values() {
            handler.handle_input(event.clone()).await?;
        }
        Ok(())
    }
}

/// Trait for input event handlers
#[async_trait::async_trait]
pub trait InputHandler {
    async fn handle_input(&self, event: InputEvent) -> Result<()>;
}

/// Convert Android key codes to Linux key codes
pub fn android_to_linux_keycode(android_keycode: u32) -> u32 {
    // This is a simplified mapping - a full implementation would need
    // a comprehensive lookup table
    match android_keycode {
        // Common keys
        7..=16 => android_keycode - 7 + 2,  // 0-9 keys
        29 => 30,  // A
        30 => 48,  // B
        31 => 46,  // C
        32 => 32,  // D
        33 => 18,  // E
        // ... more mappings would be needed
        _ => android_keycode, // Fallback to same code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_event_builder() {
        let event = InputEventBuilder::pointer_event(0, 100.0, 200.0, 0, 1.0, 1000);
        
        if let InputEvent::Pointer(pointer_event) = event {
            assert_eq!(pointer_event.x, 100);
            assert_eq!(pointer_event.y, 200);
            assert_eq!(pointer_event.event_type, PointerEventType::Down);
        } else {
            panic!("Expected pointer event");
        }
    }
    
    #[test]
    fn test_input_state() {
        let mut state = InputState::new();
        let event = InputEventBuilder::pointer_event(0, 100.0, 200.0, 0, 1.0, 1000);
        
        state.update(&event);
        assert_eq!(state.get_pointer_position(0), Some((100, 200)));
    }
}