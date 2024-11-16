package com.tmod.core.models;

import java.net.URL;
import java.util.Date;
import java.util.List;

/**
 * See the <a href="https://docs.curseforge.com/rest-api/#get-mod-file">"Get mod file"</a> endpoint
 */
public record File(
        int id,
        int gameId,
        int modId,
        boolean isAvailable,
        String displayName,
        String fileName,
        ReleaseType releaseType,
        int fileStatus,
        List<Hash> hashes,
        Date fileDate,
        int fileLength,
        int downloadCount,
        int fileSizeOnDisk,
        URL downloadUrl,
        List<String> gameVersions,
        List<SortableVersion> sortableGameVersions,
        List<Dependency> dependencies,
        boolean exposeAsAlternative,
        int parentProjectFileId,
        int alternativeFileId,
        boolean isServerPack,
        int serverPackFileId,
        boolean isEarlyAccessContent,
        Date earlyAccessEndDate,
        int fileFingerprint,
        List<Module> modules
) {
}

record SortableVersion(
        String gameVersionName,
        String gameVersionPadded,
        String gameVersion,
        Date gameVersionReleaseDate,
        int gameVersionTypeId
) {
}

/**
 * Sent as an integer by the server
 * <p>
 * <table>
 *     <tr>
 *         <td>Type</td>
 *         <td>Value</td>
 *     </tr>
 *     <tr>
 *         <td>EmbeddedLibrary</td>
 *         <td>1</td>
 *     </tr>
 *     <tr>
 *         <td>OptionalDependency</td>
 *         <td>2</td>
 *     </tr>
 *     <tr>
 *         <td>RequiredDependency</td>
 *         <td>3</td>
 *     </tr>
 *     <tr>
 *         <td>Tool</td>
 *         <td>4</td>
 *     </tr>
 *     <tr>
 *         <td>Incompatible</td>
 *         <td>5</td>
 *     </tr>
 *     <tr>
 *         <td>Include</td>
 *         <td>6</td>
 *     </tr>
 * </table>
 */
enum RelationType {
    __SKIP,
    EmbeddedLibrary,
    OptionalDependency,
    RequiredDependency,
    Tool,
    Incompatible,
    Include
}

record Dependency(int modId, RelationType relationType) {}

record Hash(String value, int algo) {}

record Module(String name, int fingerprint) {}
