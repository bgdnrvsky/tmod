package com.tmod.core.net;


import com.fasterxml.jackson.databind.JavaType;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.type.TypeFactory;
import com.tmod.core.models.Mod;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;


/**
 * Utility class that expose method to interact with the CurseForge REST API.
 *
 * <p>
 * This class provides methods to query CurseForge's API, such as retrieving mod details by ID.
 * It uses Java's {@link HttpClient} to send HTTP requests and Jackson for JSON parsing.
 * </p>
 */
public class TmodClient {

    /**
     * CurseForge's REST API
     */
    private static final String API_BASE_URL = "https://api.curseforge.com/v1/";

    /**
     * CurseForge API key
     * cf: <a href="https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround">CurseForge Workaround</a>
     */
    private static final String API_KEY = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";

    private static final HttpClient client = HttpClient.newHttpClient();

    /**
     * Searches for a mod by its ID using the CurseForge API.
     * <p>
     * This method sends a GET request to the `/mods/{id}` endpoint of the CurseForge API
     * and retrieves the mod details if available.
     * </p>
     *
     * @param id the unique numeric identifier of the mod to search for
     * @return a {@link Mod} object containing mod details, or {@code null} if the mod is not found
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    public static Mod searchModById(int id) throws URISyntaxException, IOException, InterruptedException {
        return CurseForgeGet(new URI(API_BASE_URL + "mods/" + Integer.toString(id)), Mod.class);
    }

    /**
     * Sends an HTTP GET request to the specified endpoint and parses the JSON response into a specified model type.
     * <p>
     * This method is a generic utility for interacting with the CurseForge API. It sends a request
     * to the given URI, handles response parsing, and maps the JSON response body to an object
     * of the specified type.
     * </p>
     *
     * @param endpoint the URI of the API endpoint to send the request to
     * @param clazz    the class type to map the JSON response to
     * @param <T>      the generic type of the response data
     * @return an object of type {@code T}, or {@code null} if the response status code is not 200
     * @throws IOException          if an I/O error occurs during the request
     * @throws InterruptedException if the operation is interrupted
     */
    private static <T> T CurseForgeGet(URI endpoint, Class<T> clazz) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(endpoint)
                .header("Accept", "application/json")
                .header("x-api-key", API_KEY)
                .GET()
                .build();

        JavaType type = TypeFactory.defaultInstance().constructParametricType(CurseForgeResponse.class, clazz);
        CurseForgeResponse<T> model = HTTPGet(request, type);
        return model != null ? model.getData() : null;
    }

    /**
     * Sends an HTTP GET request to the specified endpoint and parses the JSON response into a specified model type.
     * <p>
     * This method is a generic utility. It sends a request to the given URI,
     * handles response parsing, and maps the JSON response body to an object of the specified type.
     * </p>
     *
     * @param request  the {@link HttpRequest} instance
     * @param type     the type to deserialize the JSON response to
     * @param <T>      the generic type of the response data
     * @return an object of type {@code T}, or {@code null} if the response status code is not 200
     * @throws IOException          if an I/O error occurs during the request
     * @throws InterruptedException if the operation is interrupted
     */
    private static <T> T HTTPGet(HttpRequest request, JavaType type) throws IOException, InterruptedException {
        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());

        if (response.statusCode() != 200) {
            return null;
        }

        // Deserialize the response
        ObjectMapper mapper = new ObjectMapper();
        return mapper.readValue(response.body(), type);
    }

    /**
     * This class is just a model to easily unwrap inner model from CurseForge API responses,
     * because JSON object returned by the API always contains a "data" key.
     * <p>
     * {@code
     *     {
     *         data: ...
     *     }
     * }
     *
     * @param <T> Inner model contained by the response
     */
    private static class CurseForgeResponse<T> {
        private T data;

        public T getData() {
            return data;
        }
    }
}
