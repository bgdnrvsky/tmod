package com.tmod.core.net;

import com.tmod.core.models.Mod;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;

import static org.junit.jupiter.api.Assertions.*;

class TestSearchModById {
    @Test
    void searchExistingMinecraftMod() throws URISyntaxException, IOException, InterruptedException {
        Mod jei = TmodClient.searchModById(238222);
        assertEquals("jei", jei.slug());

        Mod geckolib = TmodClient.searchModById(388172);
        assertEquals("geckolib", geckolib.slug());
    }

    @Test
    void searchExistingNonMinecraftMod() throws URISyntaxException, IOException, InterruptedException {
        Mod sims_mod = TmodClient.searchModById(694283);
        assertNull(sims_mod);
    }
}