package com.tmod.core.repo.models;

import com.tmod.core.models.ModLoader;

/**
 *
 * @param gameVersion
 * @param loader
 */
public record Configuration(String gameVersion, ModLoader loader) {}
