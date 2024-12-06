package com.tmod.core.repo;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.tmod.core.models.Mod;
import com.tmod.core.net.CurseForgeModSearchException;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.AbstractMap;
import java.util.AbstractMap.SimpleEntry;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

public class Mapper {
    private final Path repoPath;

    /**
     * names of the repository files
     */
    private final static String PATH_CONF = "config.json";
    private final static String PATH_MODS = "tmod.json";
    private final static String PATH_LOCK = "tmod.lock";

    public Mapper(Path repoPath) {
        this.repoPath = repoPath;
    }

    /**
     * Serialize a {@link Repository} object.
     * <p>
     * This method overwrites any pre-existing directory and files
     *
     * @param repo {@link Repository}
     */
    public void write(Repository repo) throws IOException {
        Files.createDirectories(repoPath);

        ObjectMapper mapper = new ObjectMapper()
                .enable(SerializationFeature.INDENT_OUTPUT);

        mapper.writeValue(repoPath.resolve(PATH_CONF).toFile(), repo.getConfig());
        mapper.writeValue(repoPath.resolve(PATH_MODS).toFile(), repo.getManuallyAdded());
        mapper.writeValue(repoPath.resolve(PATH_LOCK).toFile(), repo.getLocks());
    }

    /**
     * Deserialize a {@link Repository} object
     *
     * @return {@link Repository}
     */
    public Repository read() throws IOException, CurseForgeModSearchException {
        ObjectMapper mapper = new ObjectMapper();

        Configuration config = mapper.readValue(repoPath.resolve(PATH_CONF).toFile(), Configuration.class);
        Set<Mod> manuallyAdded = mapper.readValue(
                repoPath.resolve(PATH_MODS).toFile(),
                new TypeReference<Set<String>>() {}
        )
                .stream()
                .map(TmodClient::searchModBySlug)
                .collect(Collectors.toSet());
        Map<Mod, DependencyInfo> locks = mapper.readValue(
                repoPath.resolve(PATH_LOCK).toFile(),
                new TypeReference<Map<String, DependencyInfo>>() {}
        )
                .entrySet()
                .stream()
                .collect(Collectors.toMap(
                        entry -> TmodClient.searchModBySlug(entry.getKey()),
                        Map.Entry::getValue)
                );

        return new Repository(config, manuallyAdded, locks);
    }
}
