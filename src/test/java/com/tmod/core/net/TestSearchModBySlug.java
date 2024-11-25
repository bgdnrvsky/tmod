package com.tmod.core.net;

import com.tmod.core.models.Mod;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;

import static org.junit.jupiter.api.Assertions.*;

public class TestSearchModBySlug {
    @Test
    void searchExistingMinecraftMod() throws URISyntaxException, IOException, InterruptedException {
        Mod jei = TmodClient.searchModBySlug("jei");
        assertNotNull(jei);
        assertEquals(238222, jei.id());

        Mod geckolib = TmodClient.searchModBySlug("geckolib");
        assertNotNull(geckolib);
        assertEquals(388172, geckolib.id());
    }

    @Test
    void searchExistingNonMinecraftMod() throws URISyntaxException, IOException, InterruptedException {
        Mod simsMod = TmodClient.searchModBySlug("less-dangerous-cast-spells-group-interactions");
        assertNull(simsMod);
    }

    @Test
    void searchNonExistingMod() throws URISyntaxException, IOException, InterruptedException {
        Mod nonExisting = TmodClient.searchModBySlug("aaaaaaaaaaabbbbbbbbbbb");
        assertNull(nonExisting);
    }
}
