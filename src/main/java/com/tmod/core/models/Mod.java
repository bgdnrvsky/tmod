package com.tmod.core.models;

import java.net.URL;
import java.util.Date;
import java.util.List;

/**
 * See the <a href="https://docs.curseforge.com/rest-api/#search-mods">"Search mods"</a> endpoint
 */
public record Mod(
        int id,
        int gameId,
        String name,
        String slug,
        Links links,
        String summary,
        int status,
        int downloadCount,
        boolean isFeatured,
        int primaryCategoryId,
        List<Category> categories,
        int classId,
        List<Author> authors,
        Logo logo,
        List<Screenshot> screenshots,
        int mainFileId,
        List<File> latestFiles,
        List<LatestFileIndex> latestFilesIndexes,
        List<LatestFileIndex> latestEarlyAccessFilesIndexes,
        String dateCreated,
        String dateModified,
        String dateReleased,
        boolean allowModDistribution,
        int gamePopularityRank,
        boolean isAvailable,
        boolean hasCommentsEnabled,
        int thumbsUpCount,
        int rating
) {
}

record Links(String websiteUrl, String wikiUrl, String issuesUrl, String sourceUrl) {}
record Author(int id, String name, String url, String avatarUrl) {}

record Logo(
        int id,
        int modId,
        String title,
        String description,
        String thumbnailUrl,
        String url
) {
}

record Screenshot(
        int id,
        int modId,
        String title,
        String description,
        String thumbnailUrl,
        String url
) {
}

record LatestFileIndex(
        String gameVersion,
        int fileId,
        String filename,
        int releaseType,
        int gameVersionTypeId,
        int modLoader
) {
}