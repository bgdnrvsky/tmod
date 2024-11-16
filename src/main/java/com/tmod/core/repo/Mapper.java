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

    /**
     * names of the repository files
     */
    private final static String PATH_CONF = "config.json";
    private final static String PATH_MODS = "tmod.json";
    private final static String PATH_LOCK = "tmod.lock";

    /**
     * Serialize a {@link Repository} object.
     * <p>
     * This method overwrites any pre-existing directory and files
     *
     * @param repo {@link Repository}
     * @param directory {@link Path}
     */
    public static void write(Repository repo, Path directory) throws IOException {
        Files.createDirectories(directory);

        ObjectMapper mapper = new ObjectMapper()
                .enable(SerializationFeature.INDENT_OUTPUT);

        mapper.writeValue(directory.resolve(PATH_CONF).toFile(), repo.getConfig());
        mapper.writeValue(directory.resolve(PATH_MODS).toFile(), repo.getManuallyAdded());
        mapper.writeValue(directory.resolve(PATH_LOCK).toFile(), repo.getLocks());
    }

    /**
     * Deserialize a {@link Repository} object
     *
     * @param directory {@link Path}
     * @return {@link Repository}
     */
    public static Repository read(Path directory) throws IOException {
        ObjectMapper mapper = new ObjectMapper();

        Configuration config = mapper.readValue(directory.resolve(PATH_CONF).toFile(), Configuration.class);
        Set<String> manuallyAdded = mapper.readValue(directory.resolve(PATH_MODS).toFile(), new TypeReference<>() {
        });
        Map<String, DependencyInfo> locks = mapper.readValue(directory.resolve(PATH_LOCK).toFile(), new TypeReference<>() {
        });

        return new Repository(config, manuallyAdded, locks);
    }
}
