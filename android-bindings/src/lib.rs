//! Android JNI bindings for the SteamOS emulator
//! 
//! This module provides the bridge between the Android Java frontend
//! and the Rust-based SteamOS emulator core.

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use std::ffi::CString;
use log::{info, error, warn};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;

mod system_info;
mod emulator_bridge;
mod error_handling;

use system_info::SystemCapabilities;
use emulator_bridge::EmulatorInstance;
use error_handling::AndroidResult;

/// Global emulator instance manager
static EMULATOR_INSTANCES: Lazy<Mutex<HashMap<u32, EmulatorInstance>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize logging for the native library
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Debug)
            .with_tag("SteamOSNative")
    );
}

/// JNI entry point for launching SteamOS
/// 
/// # Arguments
/// * `env` - JNI environment
/// * `_class` - Java class (unused)
/// * `iso_path` - Path to the SteamOS ISO file
/// 
/// # Returns
/// * `0` - Success
/// * `-1` - System capability detection failed
/// * `-2` - ISO file validation failed
/// * `-3` - Emulator initialization failed
/// * `-4` - Emulator launch failed
#[no_mangle]
pub extern "system" fn Java_com_piston_SteamOSNative_launchSteamOS(
    env: JNIEnv,
    _class: JClass,
    iso_path: JString,
) -> jint {
    init_logging();
    info!("Native launchSteamOS called");
    
    // Convert Java string to Rust string
    let iso_path_str = match env.get_string(iso_path) {
        Ok(path) => match path.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                error!("Failed to convert ISO path to UTF-8: {}", e);
                return -2;
            }
        },
        Err(e) => {
            error!("Failed to get ISO path from JNI: {}", e);
            return -2;
        }
    };
    
    info!("Launching SteamOS with ISO: {}", iso_path_str);
    
    // Check system capabilities
    let capabilities = match SystemCapabilities::detect() {
        Ok(caps) => caps,
        Err(e) => {
            error!("Failed to detect system capabilities: {}", e);
            return -1;
        }
    };
    
    info!("System capabilities: {:?}", capabilities);
    
    // Validate ISO file
    if !std::path::Path::new(&iso_path_str).exists() {
        error!("ISO file does not exist: {}", iso_path_str);
        return -2;
    }
    
    // Create emulator instance
    let instance_id = generate_instance_id();
    let mut emulator = match EmulatorInstance::new(capabilities) {
        Ok(emu) => emu,
        Err(e) => {
            error!("Failed to create emulator instance: {}", e);
            return -3;
        }
    };
    
    // Load and launch the ISO
    match emulator.load_and_launch(&iso_path_str) {
        Ok(()) => {
            info!("SteamOS launched successfully");
            
            // Store the instance for potential future operations
            EMULATOR_INSTANCES.lock().insert(instance_id, emulator);
            
            0 // Success
        }
        Err(e) => {
            error!("Failed to launch SteamOS: {}", e);
            -4
        }
    }
}

/// JNI entry point for getting system information
/// 
/// # Arguments
/// * `env` - JNI environment
/// * `_class` - Java class (unused)
/// 
/// # Returns
/// Java string containing system information
#[no_mangle]
pub extern "system" fn Java_com_piston_SteamOSNative_getSystemInfo(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    init_logging();
    info!("Native getSystemInfo called");
    
    let system_info = match SystemCapabilities::detect() {
        Ok(capabilities) => {
            format!(
                "System Compatibility Report\n\
                ===========================\n\
                Architecture: {}\n\
                CPU Cores: {}\n\
                Available Memory: {} MB\n\
                Virtualization Support: {}\n\
                GPU Acceleration: {}\n\
                OpenGL ES Version: {}\n\
                Vulkan Support: {}\n\
                \n\
                Emulation Capabilities:\n\
                - x86_64 Emulation: {}\n\
                - Hardware Acceleration: {}\n\
                - Container Support: {}\n\
                \n\
                Recommended Settings:\n\
                - Memory Allocation: {} MB\n\
                - CPU Cores: {}\n\
                - Graphics Mode: {}\n\
                \n\
                Status: {}",
                capabilities.architecture,
                capabilities.cpu_cores,
                capabilities.available_memory_mb,
                if capabilities.has_virtualization { "Yes" } else { "No" },
                if capabilities.has_gpu_acceleration { "Yes" } else { "No" },
                capabilities.opengl_version.as_deref().unwrap_or("Unknown"),
                if capabilities.has_vulkan { "Yes" } else { "No" },
                if capabilities.can_emulate_x86_64 { "Supported" } else { "Not Supported" },
                if capabilities.has_hardware_acceleration { "Available" } else { "Software Only" },
                if capabilities.has_container_support { "Available" } else { "Limited" },
                capabilities.recommended_memory_mb,
                capabilities.recommended_cpu_cores,
                capabilities.recommended_graphics_mode,
                if capabilities.is_compatible() { "Compatible" } else { "Limited Compatibility" }
            )
        }
        Err(e) => {
            error!("Failed to detect system capabilities: {}", e);
            format!(
                "System Detection Failed\n\
                =======================\n\
                Error: {}\n\
                \n\
                This device may not be compatible with SteamOS emulation.\n\
                Please check that your device meets the minimum requirements:\n\
                - Android API 28+\n\
                - ARM64 or x86_64 architecture\n\
                - At least 4GB RAM\n\
                - OpenGL ES 3.0+",
                e
            )
        }
    };
    
    // Convert Rust string to Java string
    match env.new_string(&system_info) {
        Ok(jstring) => jstring.into_inner(),
        Err(e) => {
            error!("Failed to create Java string: {}", e);
            // Return empty string on error
            match env.new_string("") {
                Ok(empty) => empty.into_inner(),
                Err(_) => std::ptr::null_mut(),
            }
        }
    }
}

/// Generate a unique instance ID for emulator instances
fn generate_instance_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Cleanup function called when the library is unloaded
#[no_mangle]
pub extern "system" fn JNI_OnUnload() {
    info!("SteamOS Native library unloading, cleaning up emulator instances");
    
    let mut instances = EMULATOR_INSTANCES.lock();
    for (id, mut instance) in instances.drain() {
        info!("Shutting down emulator instance {}", id);
        if let Err(e) = instance.shutdown() {
            error!("Failed to shutdown emulator instance {}: {}", id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instance_id_generation() {
        let id1 = generate_instance_id();
        let id2 = generate_instance_id();
        assert_ne!(id1, id2);
        assert!(id2 > id1);
    }
}