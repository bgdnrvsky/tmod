package models;

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
