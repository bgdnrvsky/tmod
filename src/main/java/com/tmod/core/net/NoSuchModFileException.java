package com.tmod.core.net;

import com.tmod.core.models.Mod;

public class NoSuchModFileException extends CurseForgeApiGetException {

    public NoSuchModFileException(Mod targetMod, String targetTimestamp) {
        super(
            new Exception(
                String.format(
                    "No mod file for the mod '%s' with timestamp=%s",
                    targetMod.slug(),
                    targetTimestamp
                )
            )
        );
    }
}
