package com.piston.model;

import java.util.Date;

public class LinuxContainer {
    public enum Status {
        INSTALLING,
        STOPPED,
        RUNNING,
        ERROR,
        UNINSTALLED
    }

    private String id;
    private String name;
    private Distribution distribution;
    private String rootfsPath;
    private Status status;
    private Date createdDate;
    private Date lastStarted;
    private String username;
    private String desktopEnvironment;
    private int vncPort;
    private int sshPort;
    private boolean autoStart;
    private long diskUsage;
    private String errorMessage;

    // Constructors
    public LinuxContainer() {
        this.status = Status.STOPPED;
        this.createdDate = new Date();
    }

    public LinuxContainer(String id, String name, Distribution distribution) {
        this();
        this.id = id;
        this.name = name;
        this.distribution = distribution;
    }

    // Getters and Setters
    public String getId() { return id; }
    public void setId(String id) { this.id = id; }

    public String getName() { return name; }
    public void setName(String name) { this.name = name; }

    public Distribution getDistribution() { return distribution; }
    public void setDistribution(Distribution distribution) { this.distribution = distribution; }

    public String getRootfsPath() { return rootfsPath; }
    public void setRootfsPath(String rootfsPath) { this.rootfsPath = rootfsPath; }

    public Status getStatus() { return status; }
    public void setStatus(Status status) { this.status = status; }

    public Date getCreatedDate() { return createdDate; }
    public void setCreatedDate(Date createdDate) { this.createdDate = createdDate; }

    public Date getLastStarted() { return lastStarted; }
    public void setLastStarted(Date lastStarted) { this.lastStarted = lastStarted; }

    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }

    public String getDesktopEnvironment() { return desktopEnvironment; }
    public void setDesktopEnvironment(String desktopEnvironment) { this.desktopEnvironment = desktopEnvironment; }

    public int getVncPort() { return vncPort; }
    public void setVncPort(int vncPort) { this.vncPort = vncPort; }

    public int getSshPort() { return sshPort; }
    public void setSshPort(int sshPort) { this.sshPort = sshPort; }

    public boolean isAutoStart() { return autoStart; }
    public void setAutoStart(boolean autoStart) { this.autoStart = autoStart; }

    public long getDiskUsage() { return diskUsage; }
    public void setDiskUsage(long diskUsage) { this.diskUsage = diskUsage; }

    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { this.errorMessage = errorMessage; }

    // Utility methods
    public boolean isRunning() {
        return status == Status.RUNNING;
    }

    public boolean isInstalled() {
        return status != Status.INSTALLING && status != Status.UNINSTALLED;
    }

    public boolean hasError() {
        return status == Status.ERROR;
    }

    public String getStatusText() {
        switch (status) {
            case INSTALLING: return "Installing...";
            case STOPPED: return "Stopped";
            case RUNNING: return "Running";
            case ERROR: return "Error";
            case UNINSTALLED: return "Uninstalled";
            default: return "Unknown";
        }
    }

    public String getFormattedDiskUsage() {
        if (diskUsage < 1024) return diskUsage + " B";
        if (diskUsage < 1024 * 1024) return String.format("%.1f KB", diskUsage / 1024.0);
        if (diskUsage < 1024 * 1024 * 1024) return String.format("%.1f MB", diskUsage / (1024.0 * 1024));
        return String.format("%.1f GB", diskUsage / (1024.0 * 1024 * 1024));
    }

    @Override
    public String toString() {
        return name + " (" + distribution.getName() + ") - " + getStatusText();
    }
}