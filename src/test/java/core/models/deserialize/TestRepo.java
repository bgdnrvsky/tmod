package core.models.deserialize;

import com.tmod.core.models.Mod;
import com.tmod.core.models.ModLoader;
import com.tmod.core.net.CurseForgeModSearchException;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.*;

public class TestRepo {
    @Test
    void de() throws IOException, CurseForgeModSearchException {
        Mapper mapper = new Mapper(Path.of("src/test/java/core/models/deserialize/test-repo"));
        Repository repository = mapper.read();

        assertEquals(new Configuration("1.20.1", ModLoader.Forge), repository.getConfig());

        Set<String> slugsInManuallyAdded = repository.getManuallyAdded().stream().map(Mod::slug).collect(Collectors.toSet());
        Set<String> slugsInLocks = repository.getLocks().keySet().stream().map(Mod::slug).collect(Collectors.toSet());

        assertTrue(slugsInLocks.contains("jei"));
        assertTrue(slugsInLocks.contains("waystones"));

        // Dependencies
        assertTrue(slugsInLocks.contains("balm")); // 'waystones' dependency
        assertFalse(slugsInManuallyAdded.contains("balm")); // wasn't manually added

        assertTrue(slugsInManuallyAdded.contains("jei"));
        assertTrue(slugsInManuallyAdded.contains("waystones"));
    }
}
