package core.models.deserialize;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.tmod.core.models.ModLoader;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class TestModLoader {
    @Test
    public void de() throws JsonProcessingException {
        ObjectMapper mapper = new ObjectMapper();
        String json;

        for (ModLoader loader : ModLoader.values()) {
            json = Integer.toString(loader.ordinal());
            assertEquals(loader, mapper.readValue(json, ModLoader.class));
        }
    }
}
