# Piston - SteamOS Launcher (Legacy Edition)

Enhanced version of Stephenson's Rocket with support for older and virtualized computers.

## Overview

This is a modified SteamOS installer that automatically detects your system capabilities and optimizes the installation for:
- **Older hardware** with limited resources
- **Virtual machines** (VMware, VirtualBox, QEMU, Hyper-V)
- **Low-memory systems** (< 4GB RAM)
- **Single-core processors**
- **Legacy graphics hardware**

## Features

### Automatic System Detection
- CPU architecture and core count detection
- Memory availability assessment
- Virtualization environment detection (VMware, VirtualBox, QEMU, Hyper-V)
- Hardware acceleration capability checking
- Container support verification (chroot/proot)

### Adaptive Launch Methods
- **Full Mode**: Standard SteamOS for capable systems
- **VM-Optimized Mode**: Optimized for virtual machines with guest tools integration
- **Legacy Mode**: Compatible with older hardware using software rendering
- **Minimal Mode**: Lightweight version for resource-constrained systems

### Enhanced Build System
- Support for older Android versions (API 21+)
- Extended architecture support (including MIPS)
- Legacy build variants with compatibility flags
- Hardware acceleration can be disabled for older systems

## Installation

### Prerequisites
- Android device or emulator
- SteamOS recovery image (download from Steam)
- Root access recommended (but not required - will use proot as fallback)

### Building the App
```bash
# Standard build
./gradlew assembleRelease

# Legacy build for older devices
./gradlew assembleLegacy
```

### Creating Custom SteamOS Images

The enhanced `gen.sh` script can create optimized SteamOS images:

```bash
# Create VM-optimized image
./imported_scripts/gen.sh -t vm

# Create legacy-compatible image
./imported_scripts/gen.sh -t legacy

# Create minimal image for low-resource systems
./imported_scripts/gen.sh -t minimal

# Combine multiple optimizations
./imported_scripts/gen.sh -v -l -m -n "SteamOS-Custom"
```

#### Build Types
- `full`: Standard SteamOS with all features (default)
- `vm`: Optimized for virtual machines
- `legacy`: Compatible with older hardware
- `minimal`: Minimal installation for resource-constrained systems

#### Options
- `-v`: Enable VM optimizations
- `-l`: Enable legacy hardware support
- `-m`: Enable minimal mode
- `-d`: Re-download SteamOS image
- `-n NAME`: Set custom image name
- `-t TYPE`: Set build type

## Usage

1. **Install the Piston app** on your Android device
2. **Download a SteamOS recovery image** from Steam's official site
3. **Launch Piston** and select your ISO file
4. **Check system compatibility** using the built-in system analyzer
5. **Launch SteamOS** - the app will automatically choose the best method

### System Compatibility Check

The app provides detailed system information including:
- Hardware specifications
- Virtualization status
- Available launch methods
- Compatibility assessment
- Performance recommendations

## Optimizations by System Type

### Virtual Machines
- Disables unnecessary services (Bluetooth, CUPS, Avahi)
- Configures VM-specific environment variables
- Enables guest tools detection and integration
- Optimizes graphics settings for VM environments
- Reduces compositor overhead

### Legacy Hardware
- Enables software rendering fallbacks
- Installs VESA and framebuffer drivers
- Configures X11 for older graphics cards
- Disables modern CPU features
- Provides firmware for older hardware

### Low-Memory Systems
- Removes unnecessary packages
- Configures lightweight desktop environment
- Reduces memory usage and swappiness
- Disables animations and effects
- Sets memory limits and optimizations

## Technical Details

### System Detection
The native code detects system capabilities by examining:
- `/proc/cpuinfo` for CPU features and virtualization flags
- `/proc/meminfo` for memory information
- `/sys/class/dmi/id/sys_vendor` for hardware vendor information
- Available container technologies (chroot, proot)

### Launch Methods
1. **Chroot** (requires root): Full container isolation
2. **PRoot** (user mode): Userspace container emulation
3. **Direct execution**: Fallback for systems without container support

### VM Detection
Supports detection of:
- VMware (via hypervisor flags and DMI information)
- VirtualBox (via vendor strings)
- QEMU/KVM (via device signatures)
- Microsoft Hyper-V (via vendor identification)

## Compatibility

### Supported Systems
- **Android**: API 21+ (Android 5.0+)
- **Architectures**: ARM, ARM64, x86, x86_64, MIPS, MIPS64
- **Memory**: Minimum 1GB (2GB+ recommended)
- **Storage**: 8GB+ free space for SteamOS image

### Tested Environments
- Physical hardware (various generations)
- VMware Workstation/Player
- VirtualBox
- QEMU/KVM
- Microsoft Hyper-V
- Android emulators

## Troubleshooting

### Common Issues

**"System detection failed"**
- Check if `/proc/cpuinfo` and `/proc/meminfo` are accessible
- Ensure the app has necessary permissions

**"Script creation failed"**
- Verify write access to `/data/local/tmp/`
- Check available storage space

**"No container support available"**
- Install proot: `pkg install proot` (Termux)
- Consider using a rooted device for chroot support

**"ISO file not found"**
- Verify the file path is correct
- Ensure the file is accessible to the app
- Check file permissions

### Performance Tips

1. **For VMs**: Allocate at least 4GB RAM and enable hardware acceleration
2. **For older hardware**: Use legacy build type and disable hardware acceleration
3. **For low memory**: Use minimal build type and close other applications
4. **For better performance**: Use SSD storage and enable VM guest tools

## Contributing

This project builds upon the original Stephenson's Rocket. Contributions are welcome for:
- Additional hardware compatibility
- New virtualization platform support
- Performance optimizations
- Bug fixes and improvements

## License

This project maintains the same license as the original Stephenson's Rocket project.

## Acknowledgments

- Original Stephenson's Rocket project and contributors
- SteamOS and Steam Deck teams at Valve
- Android NDK and build tools teams
- Open source virtualization communities

## Support

For issues specific to this enhanced version:
1. Check the system compatibility report in the app
2. Review the generated launch script in `/data/local/tmp/launch_steamos.sh`
3. Check Android logs for detailed error messages
4. Try different build types based on your system capabilities

---

**Note**: This is an unofficial modification. For official SteamOS support, please refer to Valve's documentation.