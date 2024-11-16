package com.tmod.core.repo.models;

import java.util.List;

/**
 *
 * @param timestamp
 * @param clientOnly
 * @param dependencies
 */
public record DependencyInfo(String timestamp, boolean clientOnly, List<String> dependencies) {}
