package com.tmod.core.net;


import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.databind.JavaType;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.type.TypeFactory;
import com.tmod.core.models.Category;
import com.tmod.core.models.Game;
import com.tmod.core.models.Mod;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.ArrayList;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.Objects;


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
     *     This method sends a GET request to the `/mods/{id}` endpoint of the CurseForge API
     * </p>
     *
     * @param id the unique numeric identifier of the mod to search for
     * @return a {@link Mod} object containing mod details, or {@code null} if the mod is not found, or if the searched
     * id is not a Minecraft mod (e.g. a mod for Sims 4)
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    public static Mod searchModById(int id) throws URISyntaxException, IOException, InterruptedException {
        Mod mod = CurseForgeGet(new URI(API_BASE_URL + "mods/" + id), TypeFactory.defaultInstance().constructType(Mod.class));

        return modForMinecraft(mod);
    }

    public static Mod searchModBySlug(String slug) throws URISyntaxException, IOException, InterruptedException {
        int minecraftId = TmodClient.getCurseForgeMinecraftId();
        int modsClassId = TmodClient.getCurseForgeCategories()
                .stream()
                .filter(Category::isClass)
                .filter(category -> Objects.equals(category.name(), "Mods"))
                .findFirst()
                .map(Category::classId)
                .orElse(-1);

        URI uri = URIBuilder.newBuilder()
                .endpoint(API_BASE_URL + "mods/search/")
                .appendPair("gameId", String.valueOf(minecraftId))
                .appendPair("classId", String.valueOf(modsClassId))
                .appendPair("slug", slug)
                .appendPair("pageSize", "1") // slug coupled with classId will result in a unique result
                .build();

        List<Mod> mods = CurseForgeGet(uri, TypeFactory.defaultInstance().constructCollectionType(ArrayList.class, Mod.class));

        if (mods == null || mods.isEmpty()) {
            return null;
        }

        Mod mod;

        try {
            mod = mods.getFirst();
        } catch (NoSuchElementException e) {
            // The mod with such slug doesn't exist
            return null;
        }

        return modForMinecraft(mod);
    }

    /**
     * Checks if the {@link Mod} is for Minecraft, and not any other game, by comparing its {@code gameId}
     * <p>
     * @param mod the mod that we are checking
     * @return the same mod if it is for Minecraft or {@code null} if it is not
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    private static Mod modForMinecraft(Mod mod) throws URISyntaxException, IOException, InterruptedException {
        if (mod == null) {
            // The mod doesn't exist
            return null;
        }

        int minecraftId = TmodClient.getCurseForgeMinecraftId();

        if (minecraftId != -1) {
            // Only check if we actually got the id
            if (mod.gameId() != minecraftId) {
                // The mod is not for Minecraft
                return null;
            }
        }

        return mod;
    }

    /**
     * Obtains all the games available on the CurseForge platform
     * <p>
     *     Sends a GET request to the `/games` endpoint of the CurseForge API
     * </p>
     *
     * @return the list of all the {@link Game}s available on the CurseForge platform, or {@code null} if status code is not 200
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    private static List<Game> getCurseForgeGames() throws URISyntaxException, IOException, InterruptedException {
        return CurseForgeGet(new URI(API_BASE_URL + "games/"), TypeFactory.defaultInstance().constructCollectionType(ArrayList.class, Game.class));
    }

    /**
     * Obtains all the categories available on the CurseForge platform
     * <p>
     *     Sends a GET request to the `/categories` endpoint of the CurseForge API
     * </p>
     *
     * @return the list of all {@link Category}s available on the CurseForge platform, or {@code null} if status code is not 200
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    private static List<Category> getCurseForgeCategories() throws URISyntaxException, IOException, InterruptedException {
        int minecraftId = TmodClient.getCurseForgeMinecraftId();
        URI uri = URIBuilder.newBuilder()
                            .endpoint(API_BASE_URL + "categories")
                            .appendPair("gameId", String.valueOf(minecraftId))
                            .build();

        return CurseForgeGet(uri, TypeFactory.defaultInstance().constructCollectionType(ArrayList.class, Category.class));
    }

    /**
     * Obtains the inner id of Minecraft on CurseForge platform
     * <p>
     *     Firstly fetches all the games via {@link TmodClient#getCurseForgeGames()},
     *     and then searches for the Minecraft and maps the value to get the id
     * </p>
     *
     * @return the inner id of Minecraft on the CurseForge platform, or -1 if status code is not 200,
     * or if Minecraft wasn't present on the game list
     * @throws URISyntaxException    if the constructed URI is invalid
     * @throws IOException           if an I/O error occurs during the request
     * @throws InterruptedException  if the operation is interrupted
     */
    private static int getCurseForgeMinecraftId() throws URISyntaxException, IOException, InterruptedException {
        List<Game> games = TmodClient.getCurseForgeGames();

        if (games == null) {
            return -1;
        }

        return games.stream()
                    .filter((game) -> Objects.equals(game.slug(), "minecraft") || Objects.equals(game.name(), "Minecraft"))
                    .findFirst()
                    .map(Game::id)
                    .orElse(-1);
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
     * @param type     the class type to deserialize the JSON response to
     * @param <T>      the generic type of the response data
     * @return an object of type {@code T}, or {@code null} if the response status code is not 200
     * @throws IOException          if an I/O error occurs during the request
     * @throws InterruptedException if the operation is interrupted
     */
    private static <T> T CurseForgeGet(URI endpoint, JavaType type) throws IOException, InterruptedException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(endpoint)
                .header("Accept", "application/json")
                .header("x-api-key", API_KEY)
                .GET()
                .build();

        CurseForgeResponse<T> model = HttpGet(request, TypeFactory.defaultInstance().constructParametricType(CurseForgeResponse.class, type));
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
    private static <T> T HttpGet(HttpRequest request, JavaType type) throws IOException, InterruptedException {
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
    @JsonIgnoreProperties(ignoreUnknown = true)
    private static class CurseForgeResponse<T> {
        private T data;

        public T getData() {
            return data;
        }
    }
}
