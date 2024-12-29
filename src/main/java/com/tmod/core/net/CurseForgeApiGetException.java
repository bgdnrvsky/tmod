package com.tmod.core.net;

public class CurseForgeApiGetException extends ApiGetException {

    public CurseForgeApiGetException(Exception exception) {
        super("CurseForge", exception);
    }
}
