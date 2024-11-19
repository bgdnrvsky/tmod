package com.tmod.core.net;

import com.tmod.core.models.Game;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class TestGetCurseForgeGames {
    @Test
    void getCurseForgeGames() throws URISyntaxException, IOException, InterruptedException {
        List<Game> games = TmodClient.getCurseForgeGames();
        assertNotNull(games);
    }
}
