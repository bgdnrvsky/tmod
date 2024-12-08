package com.tmod.core.repo;

import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import com.tmod.core.models.ModLoader;

import java.util.Map;
import java.util.Set;
import java.util.HashMap;
import java.util.HashSet;

/**
 * This class represents a repository (.tmod directory)
 */
public class Repository {

    /**
     * Holds target minecraft version and mod loader
     */
    private final Configuration config;

    /**
     * Collections of manually added mod
     */
    private final Set<String> manuallyAdded;

    /**
     *
     */
    private final Map<String, DependencyInfo> locks;

    /**
     * Default constructor
     */
    public Repository(String gameVersion, ModLoader loader) {
        this.config = new Configuration(gameVersion, loader);
        this.manuallyAdded = new HashSet<>();
        this.locks = new HashMap<>();
    }

    public Repository(Configuration config, Set<String> manuallyAdded, Map<String, DependencyInfo> locks) {
        this.config = config;
        this.manuallyAdded = manuallyAdded;
        this.locks = locks;
    }

    public Configuration getConfig() {
        return config;
    }

    public Set<String> getManuallyAdded() {
        return manuallyAdded;
    }

    public Map<String, DependencyInfo> getLocks() {
        return locks;
    }
}
