package com.tmod.core.net;

import com.fasterxml.jackson.databind.JavaType;

public class DeserializationException extends RuntimeException {

    public DeserializationException(JavaType type, Exception exception) {
        super(
            String.format(
                "Error deserializing type %s: %s",
                type.toString(),
                exception.getMessage()
            )
        );
    }
}
