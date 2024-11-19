package com.tmod.core.net;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;

import static org.junit.jupiter.api.Assertions.*;

public class TestGetCurseForgeMinecraftId {
    @Test
    void getCurseForgeMinecraftId() throws URISyntaxException, IOException, InterruptedException {
        int minecraftId = TmodClient.getCurseForgeMinecraftId();
        assertEquals(432, minecraftId);
    }
}
