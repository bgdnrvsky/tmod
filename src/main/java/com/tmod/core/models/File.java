package com.tmod.core.models;

import com.fasterxml.jackson.annotation.JsonProperty;

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
    String fileDate,
    int fileLength,
    int downloadCount,
    int fileSizeOnDisk,
    String downloadUrl,
    List<String> gameVersions,
    List<SortableVersion> sortableGameVersions,
    @JsonProperty("dependencies")
    List<Relation> relations,
    boolean exposeAsAlternative,
    int parentProjectFileId,
    int alternateFileId,
    boolean isServerPack,
    int serverPackFileId,
    boolean isEarlyAccessContent,
    String earlyAccessEndDate,
    String fileFingerprint,
    List<Module> modules
) {}

record SortableVersion(
    String gameVersionName,
    String gameVersionPadded,
    String gameVersion,
    String gameVersionReleaseDate,
    int gameVersionTypeId
) {}

record Hash(String value, int algo) {}

record Module(String name, String fingerprint) {}
