package com.tmod.core.net;


import com.fasterxml.jackson.databind.JavaType;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.tmod.core.models.JsonResponse;
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
public class CurseForgeClient {

    /**
     * CurseForge's REST API
     */
    private static final String API_BASE_URL = "https://api.curseforge.com/v1/";

    /**
     * CurseForge API key
     * cf: <a href="https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround">CurseForge Workaround</a>
     */
    private static final String API_KEY = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";


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
    public static Mod searchModById(String id) throws URISyntaxException, IOException, InterruptedException {
        return httpGet(new URI(API_BASE_URL + "mods/" + id), Mod.class);
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
    private static <T> T httpGet(URI endpoint, Class<T> clazz) throws IOException, InterruptedException {
        HttpClient client = HttpClient.newHttpClient();

        HttpRequest request = HttpRequest.newBuilder()
                .uri(endpoint)
                .header("Accept", "application/json")
                .header("x-api-key", API_KEY)
                .GET()
                .build();

        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());

        ObjectMapper mapper = new ObjectMapper();

        if (response.statusCode() == 200) {
            JavaType type = mapper.getTypeFactory().constructParametricType(JsonResponse.class, clazz);
            JsonResponse<T> model = mapper.readValue(response.body(), type);
            return model.getData();
        }

        return null;
    }
}
