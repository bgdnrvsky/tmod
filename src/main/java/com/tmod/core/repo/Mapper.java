package com.tmod.core.repo;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.Set;

public class Mapper {

    private final Path repoPath;

    /**
     * names of the repository files
     */
    private static final Path PATH_CONF = Path.of("config.json");
    private static final Path PATH_MODS = Path.of("tmod.json");
    private static final Path PATH_LOCK = Path.of("tmod.lock");

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

        mapper.writeValue(
            repoPath.resolve(PATH_CONF).toFile(),
            repo.getConfig()
        );
        mapper.writeValue(
            repoPath.resolve(PATH_MODS).toFile(),
            repo.getManuallyAdded()
        );
        mapper.writeValue(
            repoPath.resolve(PATH_LOCK).toFile(),
            repo.getLocks()
        );
    }

    /**
     * Deserialize a {@link Repository} object
     *
     * @return {@link Repository}
     */
    public Repository read() throws IOException {
        if (!Files.exists(repoPath)) throw new IOException(
            String.format("Repository at '%s' doesn't exist", repoPath)
        );

        ObjectMapper mapper = new ObjectMapper();

        Configuration config = mapper.readValue(
            repoPath.resolve(PATH_CONF).toFile(),
            Configuration.class
        );
        Set<String> manuallyAdded = mapper.readValue(
            repoPath.resolve(PATH_MODS).toFile(),
            new TypeReference<>() {}
        );
        Map<String, DependencyInfo> locks = mapper.readValue(
            repoPath.resolve(PATH_LOCK).toFile(),
            new TypeReference<>() {}
        );

        return new Repository(config, manuallyAdded, locks);
    }
}
