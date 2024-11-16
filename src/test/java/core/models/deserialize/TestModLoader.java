package core.models.deserialize;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.tmod.core.models.ModLoader;
import org.junit.jupiter.api.Test;

import java.io.IOException;

import static org.junit.jupiter.api.Assertions.*;

public class TestModLoader {
    @Test
    public void de() throws IOException {
        ObjectMapper mapper = new ObjectMapper();
        String json;

        for (ModLoader loader : ModLoader.values()) {
            json = Integer.toString(loader.ordinal());
            assertEquals(loader, mapper.readValue(json, ModLoader.class));
        }
    }
}
