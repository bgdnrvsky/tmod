package com.tmod.core.net;

public class CurseForgeModSearchException extends RuntimeException {

    public CurseForgeModSearchException(String slug) {
        super(String.format("No mod found by slug '%s'", slug));
    }

    public CurseForgeModSearchException(int id) {
        super(String.format("No mod found by id %d", id));
    }

    public CurseForgeModSearchException(String slug, Exception e) {
        super(
            String.format(
                "Couldn't find the mod by slug '%s': %s",
                slug,
                e.getMessage()
            )
        );
    }

    public CurseForgeModSearchException(int id, Exception e) {
        super(
            String.format(
                "Couldn't find the mod by id %d: %s",
                id,
                e.getMessage()
            )
        );
    }
}
