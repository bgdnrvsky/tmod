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
        Date dateCreated,
        Date dateModified,
        Date dateReleased,
        boolean allowModDistribution,
        int gamePopularityRank,
        boolean isAvailable,
        int thumbsUpCount,
        int rating
) {
}

record Links(URL websiteUrl, URL wikiUrl, URL issuesUrl, URL sourceUrl) {}
record Author(int id, String name, URL url) {}

record Logo(
        int id,
        int modId,
        String title,
        String description,
        URL thumbnailUrl,
        URL url
) {
}

record Screenshot(
        int id,
        int modId,
        String title,
        String description,
        URL thumbnailUrl,
        URL url
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