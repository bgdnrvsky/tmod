package com.tmod.core.models;

/**
 * See <a href="https://docs.curseforge.com/rest-api/#tocS_Game">Schemas > Game</a>
 */
public record Game(
    int id,
    String name,
    String slug,
    String dateModified,
    Assets assets
// Will never be used I guess
//    int status,
//    int apiStatus,
) {
}

/**
 * See <a href="https://docs.curseforge.com/rest-api/#tocS_GameAssets">Schemas > GameAssets</a>
 */
record Assets(
        String iconUrl,
        String tileUrl,
        String coverUrl
) {}
