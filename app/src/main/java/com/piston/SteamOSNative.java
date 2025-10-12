package com.piston;

public class LinuxContainerNative {
    static {
        System.loadLibrary("LinuxContainerNative");
    }

    // Container lifecycle operations
    public static native int createContainer(String containerId, String rootfsPath, String distributionId);
    public static native int startContainer(String containerId, String rootfsPath, String username);
    public static native int stopContainer(String containerId);
    public static native int removeContainer(String containerId, String rootfsPath);
    
    // Installation operations
    public static native int extractRootfs(String archivePath, String extractPath, String progressCallback);
    public static native int mountImage(String imagePath, String mountPath);
    public static native int unmountImage(String mountPath);
    
    // System operations
    public static native int executeCommand(String containerId, String command, String workingDir);
    public static native int installPackages(String containerId, String packages);
    public static native int setupDesktopEnvironment(String containerId, String desktopEnv);
    
    // Network and services
    public static native int startVncServer(String containerId, int port, String password);
    public static native int stopVncServer(String containerId);
    public static native int startSshServer(String containerId, int port);
    public static native int stopSshServer(String containerId);
    
    // System information
    public static native String getContainerStatus(String containerId);
    public static native long getContainerDiskUsage(String rootfsPath);
    public static native boolean isContainerRunning(String containerId);
    public static native String[] getRunningProcesses(String containerId);
    
    // Utility functions
    public static native boolean hasRootAccess();
    public static native String getSystemArchitecture();
    public static native boolean checkDependencies();
    
    // Legacy SteamOS support (for backward compatibility)
    public static native int launchSteamOS(String isoPath);
}
