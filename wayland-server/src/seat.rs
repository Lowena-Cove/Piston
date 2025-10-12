//! Wayland seat protocol handlers for input

use std::sync::{Arc, Mutex};
use wayland_server::{
    Main, Filter,
    protocol::{
        wl_seat::{self, WlSeat, Capability},
        wl_pointer::{self, WlPointer},
        wl_keyboard::{self, WlKeyboard},
    },
};
use tokio::sync::mpsc;
use log::{info, debug};

use compositor_core::input::{InputEvent, PointerEvent, KeyboardEvent};

/// Handler for wl_seat protocol
pub struct SeatHandler {
    input_sender: mpsc::UnboundedSender<InputEvent>,
    pointer_handlers: Arc<Mutex<Vec<Arc<PointerHandler>>>>,
    keyboard_handlers: Arc<Mutex<Vec<Arc<KeyboardHandler>>>>,
}

impl SeatHandler {
    pub fn new(input_sender: mpsc::UnboundedSender<InputEvent>) -> Self {
        Self {
            input_sender,
            pointer_handlers: Arc::new(Mutex::new(Vec::new())),
            keyboard_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn handle_new_seat(&self, seat: Main<WlSeat>) {
        info!("New wl_seat created");
        
        // Announce capabilities
        seat.capabilities(Capability::Pointer | Capability::Keyboard);
        
        let input_sender = self.input_sender.clone();
        let pointer_handlers = Arc::clone(&self.pointer_handlers);
        let keyboard_handlers = Arc::clone(&self.keyboard_handlers);
        
        seat.as_ref().assign(Filter::new(move |event, _seat| {
            match event {
                wl_seat::Event::GetPointer { id } => {
                    debug!("Creating pointer");
                    
                    let pointer_handler = Arc::new(PointerHandler::new(input_sender.clone()));
                    
                    {
                        let mut handlers = pointer_handlers.lock().unwrap();
                        handlers.push(Arc::clone(&pointer_handler));
                    }
                    
                    id.as_ref().assign(Filter::new(move |event, pointer| {
                        pointer_handler.handle_pointer_event(event, pointer);
                    }));
                }
                
                wl_seat::Event::GetKeyboard { id } => {
                    debug!("Creating keyboard");
                    
                    let keyboard_handler = Arc::new(KeyboardHandler::new(input_sender.clone()));
                    
                    {
                        let mut handlers = keyboard_handlers.lock().unwrap();
                        handlers.push(Arc::clone(&keyboard_handler));
                    }
                    
                    id.as_ref().assign(Filter::new(move |event, keyboard| {
                        keyboard_handler.handle_keyboard_event(event, keyboard);
                    }));
                }
                
                wl_seat::Event::GetTouch { id: _ } => {
                    debug!("Creating touch (not implemented)");
                    // Touch support would be implemented here
                }
                
                wl_seat::Event::Release => {
                    debug!("Seat released");
                }
            }
        }));
    }
    
    /// Send pointer event to all pointer handlers
    pub fn send_pointer_event(&self, event: PointerEvent) {
        let handlers = self.pointer_handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler.send_event(event.clone());
        }
    }
    
    /// Send keyboard event to all keyboard handlers
    pub fn send_keyboard_event(&self, event: KeyboardEvent) {
        let handlers = self.keyboard_handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler.send_event(event.clone());
        }
    }
}

/// Handler for wl_pointer protocol
pub struct PointerHandler {
    input_sender: mpsc::UnboundedSender<InputEvent>,
    pointer: Option<Main<WlPointer>>,
}

impl PointerHandler {
    pub fn new(input_sender: mpsc::UnboundedSender<InputEvent>) -> Self {
        Self {
            input_sender,
            pointer: None,
        }
    }
    
    pub fn handle_pointer_event(&mut self, event: wl_pointer::Event, pointer: Main<WlPointer>) {
        if self.pointer.is_none() {
            self.pointer = Some(pointer.clone());
        }
        
        match event {
            wl_pointer::Event::SetCursor { serial: _, surface: _, hotspot_x: _, hotspot_y: _ } => {
                debug!("Pointer set cursor");
                // Cursor handling would be implemented here
            }
            
            wl_pointer::Event::Release => {
                debug!("Pointer released");
                self.pointer = None;
            }
        }
    }
    
    /// Send pointer event from Android to Wayland client
    pub fn send_event(&self, event: PointerEvent) {
        if let Some(pointer) = &self.pointer {
            match event.event_type {
                compositor_core::input::PointerEventType::Enter => {
                    // pointer.enter(serial, surface, surface_x, surface_y);
                }
                compositor_core::input::PointerEventType::Leave => {
                    // pointer.leave(serial, surface);
                }
                compositor_core::input::PointerEventType::Move => {
                    pointer.motion(
                        event.timestamp as u32,
                        (event.x as f64).into(),
                        (event.y as f64).into(),
                    );
                }
                compositor_core::input::PointerEventType::Down => {
                    if let Some(button) = &event.button {
                        let button_code = match button {
                            compositor_core::input::PointerButton::Left => 0x110,
                            compositor_core::input::PointerButton::Right => 0x111,
                            compositor_core::input::PointerButton::Middle => 0x112,
                            _ => 0x110,
                        };
                        pointer.button(
                            event.timestamp as u32,
                            event.timestamp as u32,
                            button_code,
                            wl_pointer::ButtonState::Pressed,
                        );
                    }
                }
                compositor_core::input::PointerEventType::Up => {
                    if let Some(button) = &event.button {
                        let button_code = match button {
                            compositor_core::input::PointerButton::Left => 0x110,
                            compositor_core::input::PointerButton::Right => 0x111,
                            compositor_core::input::PointerButton::Middle => 0x112,
                            _ => 0x110,
                        };
                        pointer.button(
                            event.timestamp as u32,
                            event.timestamp as u32,
                            button_code,
                            wl_pointer::ButtonState::Released,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// Handler for wl_keyboard protocol
pub struct KeyboardHandler {
    input_sender: mpsc::UnboundedSender<InputEvent>,
    keyboard: Option<Main<WlKeyboard>>,
}

impl KeyboardHandler {
    pub fn new(input_sender: mpsc::UnboundedSender<InputEvent>) -> Self {
        Self {
            input_sender,
            keyboard: None,
        }
    }
    
    pub fn handle_keyboard_event(&mut self, event: wl_keyboard::Event, keyboard: Main<WlKeyboard>) {
        if self.keyboard.is_none() {
            self.keyboard = Some(keyboard.clone());
            
            // Send keymap
            let keymap = include_bytes!("../assets/keymap.xkb");
            keyboard.keymap(
                wl_keyboard::KeymapFormat::XkbV1,
                keymap.len() as i32,
            );
        }
        
        match event {
            wl_keyboard::Event::Release => {
                debug!("Keyboard released");
                self.keyboard = None;
            }
        }
    }
    
    /// Send keyboard event from Android to Wayland client
    pub fn send_event(&self, event: KeyboardEvent) {
        if let Some(keyboard) = &self.keyboard {
            match event.event_type {
                compositor_core::input::KeyboardEventType::KeyDown => {
                    keyboard.key(
                        event.timestamp as u32,
                        event.timestamp as u32,
                        event.key_code,
                        wl_keyboard::KeyState::Pressed,
                    );
                }
                compositor_core::input::KeyboardEventType::KeyUp => {
                    keyboard.key(
                        event.timestamp as u32,
                        event.timestamp as u32,
                        event.key_code,
                        wl_keyboard::KeyState::Released,
                    );
                }
                _ => {}
            }
            
            // Send modifiers
            keyboard.modifiers(
                event.timestamp as u32,
                0, // mods_depressed
                0, // mods_latched
                0, // mods_locked
                0, // group
            );
        }
    }
}