#!/bin/bash
# Build script for Piston SteamOS Launcher variants
# This script builds different variants optimized for different system types

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build variants of Piston SteamOS Launcher:

OPTIONS:
    -a, --all           Build all variants
    -r, --release       Build release variant (default)
    -l, --legacy        Build legacy variant for older devices
    -d, --debug         Build debug variant
    -c, --clean         Clean build directory first
    -h, --help          Show this help message

EXAMPLES:
    $0 --all            # Build all variants
    $0 --legacy         # Build only legacy variant
    $0 --clean --release # Clean and build release variant

EOF
}

clean_build() {
    echo "🧹 Cleaning build directory..."
    ./gradlew clean
}

build_release() {
    echo "🚀 Building release variant..."
    ./gradlew assembleRelease
    echo "✅ Release build completed: app/build/outputs/apk/release/"
}

build_legacy() {
    echo "🏛️ Building legacy variant..."
    ./gradlew assembleLegacy
    echo "✅ Legacy build completed: app/build/outputs/apk/legacy/"
}

build_debug() {
    echo "🐛 Building debug variant..."
    ./gradlew assembleDebug
    echo "✅ Debug build completed: app/build/outputs/apk/debug/"
}

build_all() {
    echo "🔨 Building all variants..."
    build_release
    build_legacy
    build_debug
    echo ""
    echo "📦 All builds completed!"
    echo "Output directory: app/build/outputs/apk/"
    ls -la app/build/outputs/apk/*/
}

# Check if gradlew exists
if [ ! -f "./gradlew" ]; then
    echo "❌ Error: gradlew not found. Make sure you're in the project root directory."
    exit 1
fi

# Make gradlew executable
chmod +x ./gradlew

# Parse command line arguments
CLEAN=false
BUILD_ALL=false
BUILD_RELEASE=false
BUILD_LEGACY=false
BUILD_DEBUG=false

if [ $# -eq 0 ]; then
    BUILD_RELEASE=true
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--all)
            BUILD_ALL=true
            shift
            ;;
        -r|--release)
            BUILD_RELEASE=true
            shift
            ;;
        -l|--legacy)
            BUILD_LEGACY=true
            shift
            ;;
        -d|--debug)
            BUILD_DEBUG=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

echo "🎯 Piston SteamOS Launcher Build Script"
echo "========================================"

# Clean if requested
if [ "$CLEAN" = true ]; then
    clean_build
fi

# Build variants
if [ "$BUILD_ALL" = true ]; then
    build_all
else
    if [ "$BUILD_RELEASE" = true ]; then
        build_release
    fi
    
    if [ "$BUILD_LEGACY" = true ]; then
        build_legacy
    fi
    
    if [ "$BUILD_DEBUG" = true ]; then
        build_debug
    fi
fi

echo ""
echo "🎉 Build process completed!"
echo ""
echo "📋 Next steps:"
echo "1. Install the APK on your Android device"
echo "2. Download a SteamOS recovery image"
echo "3. Use the app to check system compatibility"
echo "4. Launch SteamOS with optimized settings"
echo ""
echo "💡 Tip: Use the legacy variant for older devices or VMs"