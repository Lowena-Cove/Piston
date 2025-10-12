package com.piston.manager;

import android.content.Context;
import android.content.res.AssetManager;
import com.google.gson.Gson;
import com.google.gson.reflect.TypeToken;
import com.piston.model.Distribution;
import com.piston.model.Category;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class DistributionManager {
    private static DistributionManager instance;
    private List<Distribution> distributions;
    private List<Category> categories;
    private Map<String, Distribution> distributionMap;
    private Map<String, Category> categoryMap;

    private DistributionManager() {
        distributions = new ArrayList<>();
        categories = new ArrayList<>();
        distributionMap = new HashMap<>();
        categoryMap = new HashMap<>();
    }

    public static synchronized DistributionManager getInstance() {
        if (instance == null) {
            instance = new DistributionManager();
        }
        return instance;
    }

    public void loadDistributions(Context context) throws IOException {
        AssetManager assetManager = context.getAssets();
        InputStream inputStream = assetManager.open("distributions.json");
        InputStreamReader reader = new InputStreamReader(inputStream);

        Gson gson = new Gson();
        Type type = new TypeToken<DistributionConfig>(){}.getType();
        DistributionConfig config = gson.fromJson(reader, type);

        distributions.clear();
        categories.clear();
        distributionMap.clear();
        categoryMap.clear();

        if (config.distributions != null) {
            distributions.addAll(config.distributions);
            for (Distribution dist : config.distributions) {
                distributionMap.put(dist.getId(), dist);
            }
        }

        if (config.categories != null) {
            categories.addAll(config.categories);
            for (Category cat : config.categories) {
                categoryMap.put(cat.getId(), cat);
            }
        }

        reader.close();
        inputStream.close();
    }

    public List<Distribution> getAllDistributions() {
        return new ArrayList<>(distributions);
    }

    public List<Distribution> getDistributionsByCategory(String categoryId) {
        List<Distribution> result = new ArrayList<>();
        for (Distribution dist : distributions) {
            if (categoryId.equals(dist.getCategory())) {
                result.add(dist);
            }
        }
        return result;
    }

    public List<Distribution> getDistributionsByArchitecture(String architecture) {
        List<Distribution> result = new ArrayList<>();
        for (Distribution dist : distributions) {
            if (dist.supportsArchitecture(architecture)) {
                result.add(dist);
            }
        }
        return result;
    }

    public Distribution getDistributionById(String id) {
        return distributionMap.get(id);
    }

    public List<Category> getAllCategories() {
        return new ArrayList<>(categories);
    }

    public Category getCategoryById(String id) {
        return categoryMap.get(id);
    }

    public List<Distribution> searchDistributions(String query) {
        List<Distribution> result = new ArrayList<>();
        String lowerQuery = query.toLowerCase();
        
        for (Distribution dist : distributions) {
            if (dist.getName().toLowerCase().contains(lowerQuery) ||
                dist.getDescription().toLowerCase().contains(lowerQuery) ||
                dist.getId().toLowerCase().contains(lowerQuery)) {
                result.add(dist);
            }
        }
        return result;
    }

    public String getCurrentArchitecture() {
        String arch = System.getProperty("os.arch");
        if (arch == null) return "arm64";
        
        // Map Android architectures to distribution architectures
        switch (arch.toLowerCase()) {
            case "aarch64":
            case "arm64-v8a":
                return "arm64";
            case "arm":
            case "armeabi-v7a":
                return "armhf";
            case "x86_64":
            case "amd64":
                return "amd64";
            case "i386":
            case "x86":
                return "i386";
            default:
                return "arm64"; // Default fallback
        }
    }

    // Inner class for JSON parsing
    private static class DistributionConfig {
        List<Distribution> distributions;
        List<Category> categories;
    }
}