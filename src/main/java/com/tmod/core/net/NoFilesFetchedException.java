package com.tmod.core.net;

import com.tmod.core.models.Mod;

public class NoFilesFetchedException extends CurseForgeApiGetException {

    public NoFilesFetchedException(Mod targetMod) {
        super(
            new Exception(
                String.format(
                    "No files fetched while searching for mod '%s'",
                    targetMod.slug()
                )
            )
        );
    }
}
