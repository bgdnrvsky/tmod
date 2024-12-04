package com.tmod.core.net;

import com.tmod.core.models.Mod;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;

import static org.junit.jupiter.api.Assertions.*;

class TestSearchModById {
    @Test
    void searchExistingMinecraftMod() throws CurseForgeModSearchException {
        Mod jei = TmodClient.searchModById(238222);
        assertEquals("jei", jei.slug());

        Mod geckolib = TmodClient.searchModById(388172);
        assertEquals("geckolib", geckolib.slug());
    }

    @Test
    void searchExistingNonMinecraftMod()  {
        assertThrows(CurseForgeModSearchException.class, () -> TmodClient.searchModById(694283));
    }
}