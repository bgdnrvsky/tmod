package com.tmod.core.repo.models;

public enum ModLoader {
    Forge("1"),
    Fabric("4"),
    Quilt("5"),
    NeoForge("6");

    /**
     * API id
     */
    private final String id;

    ModLoader(String id) {
        this.id = id;
    }

    public String getId() {
        return id;
    }
}
