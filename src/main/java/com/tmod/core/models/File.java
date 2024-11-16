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
        int releaseType,
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

record Dependency(int modId, int relationType) {}

record Hash(String value, int algo) {}

record Module(String name, int fingerprint) {}
