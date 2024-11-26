package com.tmod.core.models;

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
    ModStatus status,
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
) {}

record Links(
    String websiteUrl,
    String wikiUrl,
    String issuesUrl,
    String sourceUrl
) {}

record Author(int id, String name, String url, String avatarUrl) {}

/**
 * Sent as an integer by the server
 * <p>
 * <table>
 *     <tr>
 *         <td>Status</td>
 *         <td>Value</td>
 *     </tr>
 *     <tr>
 *         <td>New</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>ChangesRequired</td>
 *         <td>2</td>
 *     </tr>
 *     <tr>
 *         <td>UnderSoftReview</td>
 *         <td>3</td>
 *     </tr>
 *     <tr>
 *         <td>Approved</td>
 *         <td>4</td>
 *     </tr>
 *     <tr>
 *         <td>Rejected</td>
 *         <td>5</td>
 *     </tr>
 *     <tr>
 *         <td>ChangesMade</td>
 *         <td>6</td>
 *     </tr>
 *     <tr>
 *         <td>Inactive</td>
 *         <td>7</td>
 *     </tr>
 *     <tr>
 *         <td>Abandoned</td>
 *         <td>8</td>
 *     </tr>
 *     <tr>
 *         <td>Deleted</td>
 *         <td>9</td>
 *     </tr>
 *     <tr>
 *         <td>UnderReview</td>
 *         <td>10</td>
 *     </tr>
 * </table>
 */
enum ModStatus {
    __SKIP,
    New,
    ChangesRequired,
    UnderSoftReview,
    Approved,
    Rejected,
    ChangesMade,
    Inactive,
    Abandoned,
    Deleted,
    UnderReview,
}

record Logo(
    int id,
    int modId,
    String title,
    String description,
    String thumbnailUrl,
    String url
) {}

record Screenshot(
    int id,
    int modId,
    String title,
    String description,
    String thumbnailUrl,
    String url
) {}

record LatestFileIndex(
    String gameVersion,
    int fileId,
    String filename,
    ReleaseType releaseType,
    int gameVersionTypeId,
    ModLoader modLoader
) {}
