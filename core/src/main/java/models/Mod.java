package models;

import java.util.List;

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
        int thumbsUpCount,
        int rating
) {
}


// Nested records outside the main `SearchedMod` record:

record Links(String websiteUrl, String wikiUrl, String issuesUrl, String sourceUrl) {
}

record Author(int id, String name, String url) {
}

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

record SortableGameVersion(
        String gameVersionName,
        String gameVersionPadded,
        String gameVersion,
        String gameVersionReleaseDate,
        int gameVersionTypeId
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
