package core.models.deserialize;

import static org.junit.jupiter.api.Assertions.*;

import com.tmod.core.models.ModLoader;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import java.io.IOException;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;

public class TestRepo {

    @Test
    void de() throws IOException {
        Mapper mapper = new Mapper(
            Path.of("src/test/java/core/models/deserialize/test-repo")
        );
        Repository repository = mapper.read();

        assertEquals(
            new Configuration("1.20.1", ModLoader.Forge),
            repository.getConfig()
        );

        // Manually added mods
        assertTrue(repository.getManuallyAdded().contains("jei"));
        assertTrue(repository.getManuallyAdded().contains("waystones"));
        assertFalse(repository.getManuallyAdded().contains("balm")); // wasn't manually added

        // Manually added mods are also present in locks
        assertTrue(repository.getLocks().containsKey("jei"));
        assertTrue(repository.getLocks().containsKey("waystones"));

        // Dependencies have their own entry in locks
        assertTrue(repository.getLocks().containsKey("balm"));

        // 'balm' is 'waystones' dependency
        assertTrue(
            repository
                .getLocks()
                .get("waystones")
                .dependencies()
                .contains("balm")
        );
    }
}
