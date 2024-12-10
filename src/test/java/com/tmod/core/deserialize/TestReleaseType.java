package com.tmod.core.deserialize;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.tmod.core.models.ReleaseType;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Iterator;

import static org.junit.jupiter.api.Assertions.*;

public class TestReleaseType {
    @Test
    public void de() throws JsonProcessingException {
        ObjectMapper mapper = new ObjectMapper();
        String json;

        for (Iterator<ReleaseType> it = Arrays.stream(ReleaseType.values()).skip(1).iterator(); it.hasNext(); ) {
            ReleaseType type = it.next();

            json = Integer.toString(type.ordinal());
            assertEquals(type, mapper.readValue(json, ReleaseType.class));
        }
    }
}
