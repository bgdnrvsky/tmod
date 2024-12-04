package com.tmod.core.net;

import com.tmod.core.models.Mod;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;

import static org.junit.jupiter.api.Assertions.*;

public class TestSearchModBySlug {
    @Test
    void searchExistingMinecraftMod() throws CurseForgeModSearchException {
        Mod jei = TmodClient.searchModBySlug("jei");
        assertEquals(238222, jei.id());

        Mod geckolib = TmodClient.searchModBySlug("geckolib");
        assertEquals(388172, geckolib.id());
    }

    @Test
    void searchExistingNonMinecraftMod() {
        assertThrows(CurseForgeModSearchException.class, () -> TmodClient.searchModBySlug("less-dangerous-cast-spells-group-interactions"));
    }

    @Test
    void searchNonExistingMod() {
        assertThrows(CurseForgeModSearchException.class, () -> TmodClient.searchModBySlug("aaaaaaaaaaabbbbbbbbbbb"));
    }
}
