package com.tmod.core.net;

public abstract class ApiGetException extends Exception {
    public ApiGetException(String apiName, Exception getException) {
        super(
            String.format(
                "%s API error: %s",
                apiName,
                getException.getMessage()
            )
        );
    }
}
