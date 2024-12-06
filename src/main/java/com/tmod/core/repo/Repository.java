package com.tmod.core.repo;

import com.tmod.core.models.Mod;
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
    private final Set<Mod> manuallyAdded;

    /**
     *
     */
    private final Map<Mod, DependencyInfo> locks;

    /**
     * Default constructor
     */
    public Repository(String gameVersion, ModLoader loader) {
        this.config = new Configuration(gameVersion, loader);
        this.manuallyAdded = new HashSet<>();
        this.locks = new HashMap<>();
    }

    public Repository(Configuration config, Set<Mod> manuallyAdded, Map<Mod, DependencyInfo> locks) {
        this.config = config;
        this.manuallyAdded = manuallyAdded;
        this.locks = locks;
    }

    public Configuration getConfig() {
        return config;
    }

    public Set<Mod> getManuallyAdded() {
        return manuallyAdded;
    }

    public Map<Mod, DependencyInfo> getLocks() {
        return locks;
    }
}
