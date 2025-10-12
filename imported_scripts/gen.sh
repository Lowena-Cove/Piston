#!/bin/sh
# (Imported from Stephensons Rocket, updated for latest SteamOS)
# Enhanced version with support for older and virtualized computers
# See https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage for latest SteamOS images and instructions.
# This script builds custom SteamOS images with mods and optimizations for different system types.

#Basic variables
BUILD="./buildroot"
DISTNAME="steamos-recovery"
ISOPATH="."
ISONAME="steamdeck-recovery-latest.img"
ISOVNAME="SteamOS Recovery (latest)"
# Official SteamOS recovery image URL (as of 2025)
UPSTREAMURL="https://cdn.cloudflare.steamstatic.com/steamdeck-image/steamdeck_recovery"
STEAMINSTALLFILE="steamdeck-recovery-latest.img.bz2"
MD5SUMFILE="MD5SUMS"

KNOWNINSTALLER="316a91157ff200bac44dcc4350a4ccf8" # Update if needed
REPODIR="./archive-mirror/mirror/repo.steampowered.com/steamos"

# New variables for enhanced functionality
BUILD_TYPE="full"  # full, vm, legacy, minimal
ENABLE_VM_OPTIMIZATIONS=0
ENABLE_LEGACY_SUPPORT=0
ENABLE_MINIMAL_MODE=0

usage ( )
{
        cat <<EOF
        $0 [OPTION] [MOD1] [MOD2] [MOD3] ...
        -h                Print this message
        -d        Re-Download ${STEAMINSTALLFILE}
        -n        Set the name for the image
        -t TYPE   Set build type: full, vm, legacy, minimal
        -v        Enable VM optimizations
        -l        Enable legacy hardware support
        -m        Enable minimal mode (for low-resource systems)
        
        Build Types:
          full    - Standard SteamOS with all features (default)
          vm      - Optimized for virtual machines
          legacy  - Compatible with older hardware
          minimal - Minimal installation for resource-constrained systems
        
        (Updated for latest SteamOS: https://help.steampowered.com/en/faqs/view/1B71-EDF2-EB6D-2BB3#reimage)
EOF
}

deps ( ) {
        deps="bzip2 rsync wget 7z realpath guestmount kpartx"
        for dep in ${deps}; do
                if which ${dep} >/dev/null 2>&1; then
                        :
                else
                        echo "Missing dependency: ${dep}"
                        echo "Install with your package manager."
                        # Provide installation hints for common systems
                        case ${dep} in
                                guestmount)
                                        echo "  Ubuntu/Debian: sudo apt install libguestfs-tools"
                                        echo "  CentOS/RHEL: sudo yum install libguestfs-tools"
                                        echo "  Arch: sudo pacman -S libguestfs"
                                        ;;
                                kpartx)
                                        echo "  Ubuntu/Debian: sudo apt install kpartx"
                                        echo "  CentOS/RHEL: sudo yum install kpartx"
                                        echo "  Arch: sudo pacman -S multipath-tools"
                                        ;;
                        esac
                        exit 1
                fi
        done
}

rebuild ( ) {
        if [ -d "${BUILD}" ]; then
                echo "Building ${BUILD} from scratch"
                rm -fr "${BUILD}"
        fi
}

extract ( ) {
        steaminstallerurl="${UPSTREAMURL}/${STEAMINSTALLFILE}"
        if [ ! -f ${STEAMINSTALLFILE} ] || [ -n "${redownload}" ]; then
                if [ -f ${STEAMINSTALLFILE} ];then
                        rm ${STEAMINSTALLFILE}
                fi
                echo "Downloading ${steaminstallerurl} ..."
                if wget -O ${STEAMINSTALLFILE} ${steaminstallerurl}; then
                        :
                else
                        echo "Error downloading ${steaminstallerurl}!"
                        exit 1
                fi
        else
                echo "Using existing ${STEAMINSTALLFILE}"
        fi
        # Decompress the .bz2 image
        if [ ! -f ${ISONAME} ]; then
                echo "Decompressing ${STEAMINSTALLFILE} ..."
                if bzip2 -dk ${STEAMINSTALLFILE}; then
                        :
                else
                        echo "Error decompressing ${STEAMINSTALLFILE}!"
                        exit 1
                fi
        fi
}

# Enhanced modding logic with support for different build types
mod_image ( ) {
        echo "Modding image: ${ISONAME} for build type: ${BUILD_TYPE}"
        
        # Create mount point
        MOUNT_POINT="/tmp/steamos_mount_$$"
        mkdir -p "${MOUNT_POINT}"
        
        # Mount the image
        if guestmount -a "${ISONAME}" -i --rw "${MOUNT_POINT}"; then
                echo "Image mounted successfully at ${MOUNT_POINT}"
        else
                echo "Failed to mount image with guestmount, trying kpartx..."
                LOOP_DEV=$(sudo losetup -f --show "${ISONAME}")
                sudo kpartx -av "${LOOP_DEV}"
                # Find the root partition (usually the largest one)
                ROOT_PART=$(sudo kpartx -l "${LOOP_DEV}" | awk '{print $1}' | head -1)
                sudo mkdir -p "${MOUNT_POINT}"
                if sudo mount "/dev/mapper/${ROOT_PART}" "${MOUNT_POINT}"; then
                        echo "Image mounted with kpartx at ${MOUNT_POINT}"
                else
                        echo "Failed to mount image"
                        cleanup_mount
                        return 1
                fi
        fi
        
        # Apply modifications based on build type
        case "${BUILD_TYPE}" in
                "vm")
                        apply_vm_optimizations "${MOUNT_POINT}"
                        ;;
                "legacy")
                        apply_legacy_support "${MOUNT_POINT}"
                        ;;
                "minimal")
                        apply_minimal_optimizations "${MOUNT_POINT}"
                        ;;
                "full")
                        echo "Using full build - no special optimizations"
                        ;;
        esac
        
        # Apply general optimizations if flags are set
        if [ "${ENABLE_VM_OPTIMIZATIONS}" = "1" ]; then
                apply_vm_optimizations "${MOUNT_POINT}"
        fi
        
        if [ "${ENABLE_LEGACY_SUPPORT}" = "1" ]; then
                apply_legacy_support "${MOUNT_POINT}"
        fi
        
        if [ "${ENABLE_MINIMAL_MODE}" = "1" ]; then
                apply_minimal_optimizations "${MOUNT_POINT}"
        fi
        
        # Apply any custom mods
        if [ -n "${MODSTOADD}" ]; then
                echo "Applying custom mods: ${MODSTOADD}"
                for mod in ${MODSTOADD}; do
                        if [ -d "${mod}" ]; then
                                echo "Copying mod directory: ${mod}"
                                sudo rsync -av "${mod}/" "${MOUNT_POINT}/"
                        elif [ -f "${mod}" ]; then
                                echo "Applying mod script: ${mod}"
                                sudo bash "${mod}" "${MOUNT_POINT}"
                        fi
                done
        fi
        
        # Cleanup
        cleanup_mount
}

# VM-specific optimizations
apply_vm_optimizations ( ) {
        local mount_point="$1"
        echo "Applying VM optimizations..."
        
        # Disable unnecessary services
        sudo chroot "${mount_point}" systemctl disable bluetooth.service 2>/dev/null || true
        sudo chroot "${mount_point}" systemctl disable cups.service 2>/dev/null || true
        sudo chroot "${mount_point}" systemctl disable avahi-daemon.service 2>/dev/null || true
        
        # Configure for VM environments
        cat << 'EOF' | sudo tee "${mount_point}/etc/steamos-vm-config" > /dev/null
# SteamOS VM Configuration
export STEAMOS_VM_MODE=1
export STEAMOS_DISABLE_COMPOSITOR=1
export STEAMOS_DISABLE_ANIMATIONS=1
export MESA_GL_VERSION_OVERRIDE=3.3
export MESA_GLSL_VERSION_OVERRIDE=330
EOF
        
        # Add VM guest tools detection
        cat << 'EOF' | sudo tee "${mount_point}/usr/local/bin/detect-vm-guest-tools" > /dev/null
#!/bin/bash
# Detect and configure VM guest tools
if lspci | grep -i vmware; then
    echo "VMware detected"
    systemctl enable vmware-tools || true
elif lspci | grep -i virtualbox; then
    echo "VirtualBox detected"
    systemctl enable vboxservice || true
elif lspci | grep -i qemu; then
    echo "QEMU detected"
    systemctl enable qemu-guest-agent || true
fi
EOF
        sudo chmod +x "${mount_point}/usr/local/bin/detect-vm-guest-tools"
        
        # Add to startup
        echo "/usr/local/bin/detect-vm-guest-tools" | sudo tee -a "${mount_point}/etc/rc.local" > /dev/null
}

# Legacy hardware support
apply_legacy_support ( ) {
        local mount_point="$1"
        echo "Applying legacy hardware support..."
        
        # Disable modern CPU features that might not be available
        cat << 'EOF' | sudo tee "${mount_point}/etc/steamos-legacy-config" > /dev/null
# SteamOS Legacy Hardware Configuration
export STEAMOS_LEGACY_MODE=1
export STEAMOS_DISABLE_HARDWARE_ACCELERATION=1
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
EOF
        
        # Install additional drivers for older hardware
        if [ -f "${mount_point}/usr/bin/apt-get" ]; then
                sudo chroot "${mount_point}" apt-get update
                sudo chroot "${mount_point}" apt-get install -y \
                        xserver-xorg-video-vesa \
                        xserver-xorg-video-fbdev \
                        firmware-linux-free \
                        firmware-linux-nonfree 2>/dev/null || true
        fi
        
        # Configure X11 for older graphics cards
        cat << 'EOF' | sudo tee "${mount_point}/etc/X11/xorg.conf.d/99-legacy-graphics.conf" > /dev/null
Section "Device"
    Identifier "Legacy Graphics"
    Driver "vesa"
    Option "UseFBDev" "true"
EndSection

Section "Screen"
    Identifier "Legacy Screen"
    Device "Legacy Graphics"
    DefaultDepth 16
EndSection
EOF
}

# Minimal system optimizations
apply_minimal_optimizations ( ) {
        local mount_point="$1"
        echo "Applying minimal system optimizations..."
        
        # Remove unnecessary packages to save space
        if [ -f "${mount_point}/usr/bin/apt-get" ]; then
                sudo chroot "${mount_point}" apt-get remove -y \
                        libreoffice* \
                        firefox* \
                        thunderbird* \
                        games-* \
                        gnome-games* 2>/dev/null || true
                sudo chroot "${mount_point}" apt-get autoremove -y
                sudo chroot "${mount_point}" apt-get clean
        fi
        
        # Configure for low memory usage
        cat << 'EOF' | sudo tee "${mount_point}/etc/steamos-minimal-config" > /dev/null
# SteamOS Minimal Configuration
export STEAMOS_MINIMAL_MODE=1
export STEAMOS_LOW_MEMORY=1
export STEAMOS_DISABLE_ANIMATIONS=1
export STEAMOS_DISABLE_EFFECTS=1
ulimit -v 1048576  # Limit virtual memory to 1GB
EOF
        
        # Reduce swappiness for better performance on low-memory systems
        echo "vm.swappiness=10" | sudo tee -a "${mount_point}/etc/sysctl.conf" > /dev/null
        
        # Configure lightweight desktop environment
        if [ -d "${mount_point}/etc/xdg/autostart" ]; then
                # Disable heavy startup applications
                for app in "${mount_point}"/etc/xdg/autostart/*.desktop; do
                        if [ -f "$app" ]; then
                                echo "Hidden=true" | sudo tee -a "$app" > /dev/null
                        fi
                done
        fi
}

# Cleanup mount points
cleanup_mount ( ) {
        echo "Cleaning up mount points..."
        if mountpoint -q "${MOUNT_POINT}"; then
                sudo umount "${MOUNT_POINT}" || sudo umount -l "${MOUNT_POINT}"
        fi
        if [ -n "${LOOP_DEV}" ]; then
                sudo kpartx -dv "${LOOP_DEV}" 2>/dev/null || true
                sudo losetup -d "${LOOP_DEV}" 2>/dev/null || true
        fi
        sudo rmdir "${MOUNT_POINT}" 2>/dev/null || true
}

mkchecksum ( ) {
        echo "Generating checksum..."
        md5sum ${ISONAME} > "${ISONAME}.md5"
        if [ -f ${ISONAME}.md5 ]; then
                echo "Checksum saved in ${ISONAME}.md5"
        else
                echo "Failed to save checksum"
        fi
}

while getopts "hdvlmn:t:" OPTION; do
        case ${OPTION} in
      h)
                usage
                exit 1
        ;;
        d)
            redownload="1"
        ;;
        v)
            ENABLE_VM_OPTIMIZATIONS=1
            echo "VM optimizations enabled"
        ;;
        l)
            ENABLE_LEGACY_SUPPORT=1
            echo "Legacy hardware support enabled"
        ;;
        m)
            ENABLE_MINIMAL_MODE=1
            echo "Minimal mode enabled"
        ;;
        n)
                ISOVNAME="${OPTARG}"
        ISONAME=$(echo "${OPTARG}.img"|tr '[:upper:]' '[:lower:]'|tr "\ " "-")
       ;;
        t)
            BUILD_TYPE="${OPTARG}"
            case "${BUILD_TYPE}" in
                full|vm|legacy|minimal)
                    echo "Build type set to: ${BUILD_TYPE}"
                    # Auto-enable corresponding flags
                    case "${BUILD_TYPE}" in
                        vm) ENABLE_VM_OPTIMIZATIONS=1 ;;
                        legacy) ENABLE_LEGACY_SUPPORT=1 ;;
                        minimal) ENABLE_MINIMAL_MODE=1 ;;
                    esac
                    ;;
                *)
                    echo "Invalid build type: ${BUILD_TYPE}"
                    echo "Valid types: full, vm, legacy, minimal"
                    exit 1
                    ;;
            esac
        ;;
        *)
                echo "${OPTION} - Unrecognized option"
             usage
                exit 1
        ;;
        esac
done
shift $((OPTIND-1))
MODSTOADD="$@"

# Update image name based on build type
if [ "${BUILD_TYPE}" != "full" ]; then
        ISONAME=$(echo "${ISONAME}" | sed "s/\.img$/-${BUILD_TYPE}.img/")
        ISOVNAME="${ISOVNAME} (${BUILD_TYPE})"
fi

echo "Building SteamOS image: ${ISOVNAME}"
echo "Output file: ${ISONAME}"
echo "Build type: ${BUILD_TYPE}"

deps
rebuild
extract
mod_image
mkchecksum

echo "Build completed successfully!"
echo "Image: ${ISONAME}"
echo "Type: ${BUILD_TYPE}"
echo "Optimizations applied:"
[ "${ENABLE_VM_OPTIMIZATIONS}" = "1" ] && echo "  - VM optimizations"
[ "${ENABLE_LEGACY_SUPPORT}" = "1" ] && echo "  - Legacy hardware support"
[ "${ENABLE_MINIMAL_MODE}" = "1" ] && echo "  - Minimal mode"
echo ""
echo "You can now use this image with the Piston launcher."
# End of script
