// Native entry point for launching SteamOS/Deck ISO environment
// Enhanced version with support for older and virtualized computers
// This version includes hardware detection, virtualization support, and compatibility features

#include <jni.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <string.h>
#include <sys/utsname.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <android/log.h>

#define LOG_TAG "SteamOSNative"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// System capability flags
typedef struct {
    int is_virtualized;
    int has_64bit_support;
    int memory_mb;
    int cpu_cores;
    char vm_type[32];
    char cpu_arch[16];
    int supports_chroot;
    int supports_proot;
} system_caps_t;

// Function declarations
static int detect_system_capabilities(system_caps_t *caps);
static int detect_virtualization(system_caps_t *caps);
static int get_memory_info(void);
static int get_cpu_info(system_caps_t *caps);
static int check_chroot_support(void);
static int check_proot_support(void);
static const char* get_optimal_launch_method(const system_caps_t *caps);
static int create_launch_script_optimized(const char *iso_path, const system_caps_t *caps);
static const char* get_compatibility_assessment(const system_caps_t *caps);

// Android NDK does not support fork/chroot/waitpid directly. Use shell commands as a workaround.
// This enhanced version detects system capabilities and optimizes for virtualized/older systems.

// Detect system capabilities including virtualization and hardware specs
static int detect_system_capabilities(system_caps_t *caps) {
    memset(caps, 0, sizeof(system_caps_t));
    
    // Get basic system info
    struct utsname uname_info;
    if (uname(&uname_info) == 0) {
        strncpy(caps->cpu_arch, uname_info.machine, sizeof(caps->cpu_arch) - 1);
        caps->has_64bit_support = (strstr(uname_info.machine, "64") != NULL);
    }
    
    // Detect virtualization
    detect_virtualization(caps);
    
    // Get memory and CPU info
    caps->memory_mb = get_memory_info();
    get_cpu_info(caps);
    
    // Check container support
    caps->supports_chroot = check_chroot_support();
    caps->supports_proot = check_proot_support();
    
    LOGI("System capabilities: arch=%s, 64bit=%d, vm=%s, mem=%dMB, cores=%d", 
         caps->cpu_arch, caps->has_64bit_support, caps->vm_type, 
         caps->memory_mb, caps->cpu_cores);
    
    return 0;
}

// Detect if running in a virtualized environment
static int detect_virtualization(system_caps_t *caps) {
    FILE *fp;
    char buffer[256];
    
    // Check /proc/cpuinfo for hypervisor flags
    fp = fopen("/proc/cpuinfo", "r");
    if (fp) {
        while (fgets(buffer, sizeof(buffer), fp)) {
            if (strstr(buffer, "hypervisor")) {
                caps->is_virtualized = 1;
                strcpy(caps->vm_type, "generic");
            }
            // Check for specific VM signatures
            if (strstr(buffer, "VMware")) {
                strcpy(caps->vm_type, "vmware");
            } else if (strstr(buffer, "VirtualBox")) {
                strcpy(caps->vm_type, "virtualbox");
            } else if (strstr(buffer, "QEMU")) {
                strcpy(caps->vm_type, "qemu");
            } else if (strstr(buffer, "Microsoft")) {
                strcpy(caps->vm_type, "hyperv");
            }
        }
        fclose(fp);
    }
    
    // Check DMI information if available
    fp = fopen("/sys/class/dmi/id/sys_vendor", "r");
    if (fp) {
        if (fgets(buffer, sizeof(buffer), fp)) {
            if (strstr(buffer, "VMware")) {
                caps->is_virtualized = 1;
                strcpy(caps->vm_type, "vmware");
            } else if (strstr(buffer, "innotek") || strstr(buffer, "Oracle")) {
                caps->is_virtualized = 1;
                strcpy(caps->vm_type, "virtualbox");
            } else if (strstr(buffer, "QEMU")) {
                caps->is_virtualized = 1;
                strcpy(caps->vm_type, "qemu");
            } else if (strstr(buffer, "Microsoft")) {
                caps->is_virtualized = 1;
                strcpy(caps->vm_type, "hyperv");
            }
        }
        fclose(fp);
    }
    
    return caps->is_virtualized;
}

// Get available memory in MB
static int get_memory_info(void) {
    FILE *fp = fopen("/proc/meminfo", "r");
    if (!fp) return 0;
    
    char buffer[256];
    int memory_kb = 0;
    
    while (fgets(buffer, sizeof(buffer), fp)) {
        if (sscanf(buffer, "MemTotal: %d kB", &memory_kb) == 1) {
            break;
        }
    }
    fclose(fp);
    
    return memory_kb / 1024; // Convert to MB
}

// Get CPU information
static int get_cpu_info(system_caps_t *caps) {
    FILE *fp = fopen("/proc/cpuinfo", "r");
    if (!fp) return -1;
    
    char buffer[256];
    caps->cpu_cores = 0;
    
    while (fgets(buffer, sizeof(buffer), fp)) {
        if (strstr(buffer, "processor")) {
            caps->cpu_cores++;
        }
    }
    fclose(fp);
    
    return 0;
}

// Check if chroot is supported (requires root)
static int check_chroot_support(void) {
    return (geteuid() == 0) ? 1 : 0;
}

// Check if proot is available
static int check_proot_support(void) {
    return (system("which proot >/dev/null 2>&1") == 0) ? 1 : 0;
}

// Determine optimal launch method based on system capabilities
static const char* get_optimal_launch_method(const system_caps_t *caps) {
    // For virtualized environments, prefer lighter methods
    if (caps->is_virtualized) {
        if (caps->memory_mb < 2048) {
            return "minimal"; // Minimal environment for low-memory VMs
        } else {
            return "vm_optimized"; // VM-optimized environment
        }
    }
    
    // For older hardware (less than 4GB RAM or single core)
    if (caps->memory_mb < 4096 || caps->cpu_cores <= 1) {
        return "legacy"; // Legacy-compatible environment
    }
    
    // Default full environment
    return "full";
}

// Create optimized launch script based on system capabilities
static int create_launch_script_optimized(const char *iso_path, const system_caps_t *caps) {
    FILE *script = fopen("/data/local/tmp/launch_steamos.sh", "w");
    if (!script) {
        LOGE("Failed to create shell script");
        return -1;
    }
    
    const char *launch_method = get_optimal_launch_method(caps);
    
    fprintf(script, "#!/system/bin/sh\n");
    fprintf(script, "# Auto-generated SteamOS launch script\n");
    fprintf(script, "# System: %s, Method: %s, VM: %s\n", 
            caps->cpu_arch, launch_method, caps->vm_type);
    fprintf(script, "\n");
    
    // Create root directory
    fprintf(script, "mkdir -p /data/local/tmp/steamos_root\n");
    
    // VM-specific optimizations
    if (caps->is_virtualized) {
        fprintf(script, "# VM optimizations\n");
        fprintf(script, "echo 'Running in %s environment'\n", caps->vm_type);
        
        // Disable unnecessary services for VMs
        fprintf(script, "export STEAMOS_VM_MODE=1\n");
        fprintf(script, "export STEAMOS_DISABLE_COMPOSITOR=1\n");
        
        if (strcmp(caps->vm_type, "qemu") == 0) {
            fprintf(script, "export STEAMOS_QEMU_GUEST=1\n");
        } else if (strcmp(caps->vm_type, "vmware") == 0) {
            fprintf(script, "export STEAMOS_VMWARE_GUEST=1\n");
        }
    }
    
    // Memory-based optimizations
    if (caps->memory_mb < 2048) {
        fprintf(script, "# Low memory optimizations\n");
        fprintf(script, "export STEAMOS_LOW_MEMORY=1\n");
        fprintf(script, "export STEAMOS_DISABLE_ANIMATIONS=1\n");
        fprintf(script, "ulimit -v %d\n", caps->memory_mb * 1024 * 800); // 80% of available memory
    }
    
    // Mount/extract ISO with fallbacks
    fprintf(script, "# Mount or extract ISO with fallbacks\n");
    fprintf(script, "if busybox mount -o loop,ro '%s' /data/local/tmp/steamos_root 2>/dev/null; then\n", iso_path);
    fprintf(script, "  echo 'ISO mounted successfully'\n");
    fprintf(script, "  MOUNTED=1\n");
    fprintf(script, "elif busybox tar -tf '%s' >/dev/null 2>&1; then\n", iso_path);
    fprintf(script, "  echo 'Extracting tar archive...'\n");
    fprintf(script, "  busybox tar -xf '%s' -C /data/local/tmp/steamos_root\n", iso_path);
    fprintf(script, "  MOUNTED=0\n");
    fprintf(script, "elif 7z x '%s' -o/data/local/tmp/steamos_root >/dev/null 2>&1; then\n", iso_path);
    fprintf(script, "  echo 'Extracted with 7z'\n");
    fprintf(script, "  MOUNTED=0\n");
    fprintf(script, "else\n");
    fprintf(script, "  echo 'Failed to mount or extract ISO'\n");
    fprintf(script, "  exit 1\n");
    fprintf(script, "fi\n\n");
    
    // Launch environment with appropriate method
    fprintf(script, "# Launch SteamOS environment\n");
    fprintf(script, "if [ -x /data/local/tmp/steamos_root/bin/bash ]; then\n");
    
    if (caps->supports_chroot) {
        fprintf(script, "  echo 'Using chroot (root mode)'\n");
        fprintf(script, "  chroot /data/local/tmp/steamos_root /bin/bash\n");
    } else if (caps->supports_proot) {
        fprintf(script, "  echo 'Using proot (user mode)'\n");
        if (caps->is_virtualized) {
            // Use proot with VM-friendly options
            fprintf(script, "  proot -S /data/local/tmp/steamos_root -b /dev -b /proc -b /sys /bin/bash\n");
        } else {
            fprintf(script, "  proot -S /data/local/tmp/steamos_root /bin/bash\n");
        }
    } else {
        fprintf(script, "  echo 'No container support available, trying direct execution'\n");
        fprintf(script, "  cd /data/local/tmp/steamos_root\n");
        fprintf(script, "  ./bin/bash || /system/bin/sh\n");
    }
    
    fprintf(script, "else\n");
    fprintf(script, "  echo 'No bash found in rootfs, trying alternatives'\n");
    fprintf(script, "  if [ -x /data/local/tmp/steamos_root/bin/sh ]; then\n");
    fprintf(script, "    /data/local/tmp/steamos_root/bin/sh\n");
    fprintf(script, "  else\n");
    fprintf(script, "    echo 'No shell found in SteamOS image'\n");
    fprintf(script, "    exit 127\n");
    fprintf(script, "  fi\n");
    fprintf(script, "fi\n\n");
    
    // Cleanup
    fprintf(script, "# Cleanup\n");
    fprintf(script, "if [ \"$MOUNTED\" = \"1\" ]; then\n");
    fprintf(script, "  busybox umount /data/local/tmp/steamos_root 2>/dev/null\n");
    fprintf(script, "fi\n");
    
    fclose(script);
    system("chmod 755 /data/local/tmp/launch_steamos.sh");
    
    return 0;
}

JNIEXPORT jint JNICALL
Java_com_piston_SteamOSNative_launchSteamOS(JNIEnv *env, jobject thiz, jstring isoPath) {
    const char *path = (*env)->GetStringUTFChars(env, isoPath, 0);
    LOGI("Launching SteamOS from: %s", path);
    
    // Detect system capabilities
    system_caps_t caps;
    if (detect_system_capabilities(&caps) != 0) {
        LOGE("Failed to detect system capabilities");
        (*env)->ReleaseStringUTFChars(env, isoPath, path);
        return -1;
    }
    
    // Create optimized launch script
    if (create_launch_script_optimized(path, &caps) != 0) {
        LOGE("Failed to create launch script");
        (*env)->ReleaseStringUTFChars(env, isoPath, path);
        return -2;
    }
    
    // Run the script
    LOGI("Executing launch script with method: %s", get_optimal_launch_method(&caps));
    int ret = system("sh /data/local/tmp/launch_steamos.sh");
    
    (*env)->ReleaseStringUTFChars(env, isoPath, path);
    return ret;
}

JNIEXPORT jstring JNICALL
Java_com_piston_SteamOSNative_getSystemInfo(JNIEnv *env, jobject thiz) {
    system_caps_t caps;
    char info_buffer[2048];
    
    // Detect system capabilities
    if (detect_system_capabilities(&caps) != 0) {
        strcpy(info_buffer, "Failed to detect system capabilities");
    } else {
        snprintf(info_buffer, sizeof(info_buffer),
            "System Compatibility Report\n\n"
            "Architecture: %s\n"
            "64-bit Support: %s\n"
            "CPU Cores: %d\n"
            "Memory: %d MB\n"
            "Virtualized: %s\n"
            "VM Type: %s\n"
            "Chroot Support: %s\n"
            "PRoot Support: %s\n\n"
            "Recommended Launch Method: %s\n\n"
            "Compatibility Assessment:\n"
            "%s",
            caps.cpu_arch,
            caps.has_64bit_support ? "Yes" : "No",
            caps.cpu_cores,
            caps.memory_mb,
            caps.is_virtualized ? "Yes" : "No",
            caps.is_virtualized ? caps.vm_type : "N/A",
            caps.supports_chroot ? "Yes (Root)" : "No",
            caps.supports_proot ? "Yes" : "No",
            get_optimal_launch_method(&caps),
            get_compatibility_assessment(&caps)
        );
    }
    
    return (*env)->NewStringUTF(env, info_buffer);
}

// Get compatibility assessment text
static const char* get_compatibility_assessment(const system_caps_t *caps) {
    static char assessment[512];
    
    if (caps->memory_mb < 1024) {
        strcpy(assessment, "⚠️ Very low memory. SteamOS may not run properly.\nConsider using a minimal Linux distribution instead.");
    } else if (caps->memory_mb < 2048) {
        strcpy(assessment, "⚠️ Low memory detected. Will use minimal mode.\nSome features may be disabled for performance.");
    } else if (caps->is_virtualized && caps->memory_mb < 4096) {
        strcpy(assessment, "✅ VM detected with adequate memory.\nWill use VM-optimized mode for better performance.");
    } else if (caps->cpu_cores <= 1) {
        strcpy(assessment, "⚠️ Single-core CPU detected.\nPerformance may be limited. Consider upgrading hardware.");
    } else if (caps->is_virtualized) {
        strcpy(assessment, "✅ Good VM environment detected.\nShould run well with VM optimizations enabled.");
    } else if (caps->memory_mb >= 4096 && caps->cpu_cores >= 2) {
        strcpy(assessment, "✅ Excellent compatibility.\nAll features should work properly.");
    } else {
        strcpy(assessment, "✅ Good compatibility.\nShould run with minor optimizations.");
    }
    
    return assessment;
}
