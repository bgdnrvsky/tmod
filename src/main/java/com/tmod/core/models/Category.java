package com.tmod.core.models;

/**
 * See the <a href="https://docs.curseforge.com/rest-api/#get-categories">"Get categories"</a> endpoint
 */
public record Category(
    int id,
    int gameId,
    String name,
    String slug,
    String url,
    String iconUrl,
    String dateModified,
    boolean isClass,
    int classId,
    int parentCategoryId,
    int displayIndex
) {}
