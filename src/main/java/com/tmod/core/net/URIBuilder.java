package com.tmod.core.net;

import java.net.URI;
import java.net.URISyntaxException;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;


/**
 * A utility class for building URIs with query parameters
 */
class URIBuilder {

    private String baseUrl;
    private final StringBuilder queryParams;

    /**
     * Private constructor to enforce the use of the {@link #newBuilder()} factory method.
     */
    private URIBuilder() {
        this.queryParams = new StringBuilder();
    }

    /**
     * Creates a new instance of {@code URIBuilder}.
     *
     * @return a new {@code URIBuilder} instance
     */
    public static URIBuilder newBuilder() {
        return new URIBuilder();
    }

    /**
     * Sets the base endpoint URL for the URI.
     *
     * @param endpoint the base URL (e.g., "https://example.com/api")
     * @return the current {@code URIBuilder} instance for method chaining
     */
    public URIBuilder endpoint(String endpoint) {
        this.baseUrl = endpoint;
        return this;
    }

    /**
     * Appends a query parameter to the URI.
     * <p>
     * The key and value will be URL-encoded using UTF-8 to ensure they are safe for inclusion
     * in the URI.
     * </p>
     *
     * @param key   the parameter name
     * @param value the parameter value
     * @return the current {@code URIBuilder} instance for method chaining
     */
    public URIBuilder appendPair(String key, String value) {
        if (!queryParams.isEmpty()) {
            queryParams.append("&");
        }

        queryParams
                .append(URLEncoder.encode(key, StandardCharsets.UTF_8))
                .append("=")
                .append(URLEncoder.encode(value, StandardCharsets.UTF_8));

        return this;
    }

    /**
     * Builds the final {@link URI} object.
     * <p>
     * If query parameters have been appended, they will be included in the URI.
     * Otherwise, only the base URL will be used.
     * </p>
     *
     * @return the constructed {@link URI}
     * @throws URISyntaxException if the resulting URI is not syntactically valid
     */
    public URI build() throws URISyntaxException {
        StringBuilder builder = new StringBuilder(baseUrl);

        if (!queryParams.isEmpty()) {
            builder
                .append("?")
                .append(queryParams);
        }

        return new URI(builder.toString());
    }
}
