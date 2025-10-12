package com.piston.model;

import java.util.List;
import java.util.Map;

public class Distribution {
    private String id;
    private String name;
    private String description;
    private String version;
    private List<String> architecture;
    private String category;
    private String icon;
    private String size;
    private Map<String, String> downloadUrl;
    private String installScript;
    private String postInstallScript;
    private List<String> desktopEnvironments;
    private String defaultUser;
    private List<String> features;
    private String specialType;

    // Constructors
    public Distribution() {}

    public Distribution(String id, String name, String description) {
        this.id = id;
        this.name = name;
        this.description = description;
    }

    // Getters and Setters
    public String getId() { return id; }
    public void setId(String id) { this.id = id; }

    public String getName() { return name; }
    public void setName(String name) { this.name = name; }

    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }

    public String getVersion() { return version; }
    public void setVersion(String version) { this.version = version; }

    public List<String> getArchitecture() { return architecture; }
    public void setArchitecture(List<String> architecture) { this.architecture = architecture; }

    public String getCategory() { return category; }
    public void setCategory(String category) { this.category = category; }

    public String getIcon() { return icon; }
    public void setIcon(String icon) { this.icon = icon; }

    public String getSize() { return size; }
    public void setSize(String size) { this.size = size; }

    public Map<String, String> getDownloadUrl() { return downloadUrl; }
    public void setDownloadUrl(Map<String, String> downloadUrl) { this.downloadUrl = downloadUrl; }

    public String getInstallScript() { return installScript; }
    public void setInstallScript(String installScript) { this.installScript = installScript; }

    public String getPostInstallScript() { return postInstallScript; }
    public void setPostInstallScript(String postInstallScript) { this.postInstallScript = postInstallScript; }

    public List<String> getDesktopEnvironments() { return desktopEnvironments; }
    public void setDesktopEnvironments(List<String> desktopEnvironments) { this.desktopEnvironments = desktopEnvironments; }

    public String getDefaultUser() { return defaultUser; }
    public void setDefaultUser(String defaultUser) { this.defaultUser = defaultUser; }

    public List<String> getFeatures() { return features; }
    public void setFeatures(List<String> features) { this.features = features; }

    public String getSpecialType() { return specialType; }
    public void setSpecialType(String specialType) { this.specialType = specialType; }

    // Utility methods
    public boolean supportsArchitecture(String arch) {
        return architecture != null && architecture.contains(arch);
    }

    public boolean hasFeature(String feature) {
        return features != null && features.contains(feature);
    }

    public String getDownloadUrlForArch(String arch) {
        return downloadUrl != null ? downloadUrl.get(arch) : null;
    }

    public boolean isSpecialType(String type) {
        return specialType != null && specialType.equals(type);
    }

    @Override
    public String toString() {
        return name + " (" + version + ")";
    }
}