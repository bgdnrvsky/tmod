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
        String fileDate,
        int fileLength,
        int downloadCount,
        int fileSizeOnDisk,
        String downloadUrl,
        List<String> gameVersions,
        List<SortableVersion> sortableGameVersions,
        List<Dependency> dependencies,
        boolean exposeAsAlternative,
        int parentProjectFileId,
        int alternateFileId,
        boolean isServerPack,
        int serverPackFileId,
        boolean isEarlyAccessContent,
        String earlyAccessEndDate,
        String fileFingerprint,
        List<Module> modules
) {
}

record SortableVersion(
        String gameVersionName,
        String gameVersionPadded,
        String gameVersion,
        String gameVersionReleaseDate,
        int gameVersionTypeId
) {
}

record Dependency(int modId, int relationType) {}

record Hash(String value, int algo) {}

record Module(String name, String fingerprint) {}
