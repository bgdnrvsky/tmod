package com.tmod.core.net;

import java.net.URI;

public class HttpGetException extends RuntimeException {

    public HttpGetException(URI uri, Exception e) {
        super(
            String.format(
                "Error doing a GET request to '%s': %s",
                uri.toString(),
                e.getMessage()
            )
        );
    }

    public HttpGetException(URI uri, int statusCode) {
        super(
            String.format(
                "Invalid response(%d) from '%s'",
                statusCode,
                uri.toString()
            )
        );
    }
}
