package com.tmod.core.net;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JavaType;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.type.TypeFactory;
import com.tmod.core.models.Category;
import com.tmod.core.models.File;
import com.tmod.core.models.Game;
import com.tmod.core.models.Mod;
import com.tmod.core.models.ModLoader;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

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
    private static final String API_KEY =
        "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";

    private static final HttpClient client = HttpClient.newHttpClient();

    /**
     * Searches for a mod by its ID using the CurseForge API.
     * <p>
     *     This method sends a GET request to the `/mods/{id}` endpoint of the CurseForge API
     * </p>
     *
     * @param id the unique numeric identifier of the mod to search for
     * @return a {@link Mod} object containing mod details
     * id is not a Minecraft mod (e.g. a mod for Sims 4)
     * @throws CurseForgeModSearchException couldn't find the Minecraft mod
     */
    public static Mod searchModById(int id)
        throws CurseForgeModSearchException {
        try {
            URI uri = URIBuilder.newBuilder()
                .endpoint(API_BASE_URL + "mods/" + id)
                .build();
            Mod mod = CurseForgeGet(
                uri,
                TypeFactory.defaultInstance().constructType(Mod.class)
            );

            if (mod.gameId() == TmodClient.getCurseForgeMinecraftId()) {
                return mod;
            }
        } catch (CurseForgeApiGetException e) {
            throw new CurseForgeModSearchException(id, e);
        }

        throw new CurseForgeModSearchException(id);
    }

    /**
     * Searches for a mod by its SLUG using the CurseForge API.
     * <p>
     *     This method sends a GET request to the `/mods/search` endpoint of the CurseForge API
     * </p>
     *
     * @param slug the unique identifier of the mod to search for
     * @return a {@link Mod} object containing mod details
     * id is not a Minecraft mod (e.g. a mod for Sims 4)
     * @throws CurseForgeModSearchException couldn't find the Minecraft mod
     */
    public static Mod searchModBySlug(String slug)
        throws CurseForgeModSearchException {
        try {
            int minecraftId = TmodClient.getCurseForgeMinecraftId();

            List<Category> categories = TmodClient.getCurseForgeCategories();

            int modsClassId = categories
                .stream()
                .filter(Category::isClass)
                .filter(category -> Objects.equals(category.name(), "Mods"))
                .findFirst()
                .map(Category::classId)
                .orElse(0);

            URI uri = URIBuilder.newBuilder()
                .endpoint(API_BASE_URL + "mods/search")
                .appendPair("gameId", String.valueOf(minecraftId))
                .appendPair("classId", String.valueOf(modsClassId))
                .appendPair("slug", slug)
                .appendPair("pageSize", "1") // slug coupled with classId will result in a unique result
                .build();

            List<Mod> mods = CurseForgeGet(
                uri,
                TypeFactory.defaultInstance()
                    .constructCollectionType(ArrayList.class, Mod.class)
            );

            Mod mod;

            if (!mods.isEmpty()) {
                mod = mods.getFirst();

                if (mod.gameId() == minecraftId) {
                    return mod;
                }
            }
        } catch (CurseForgeApiGetException e) {
            throw new CurseForgeModSearchException(slug, e);
        }

        throw new CurseForgeModSearchException(slug);
    }

    /**
     * Obtains all the games available on the CurseForge platform
     * <p>
     *     Sends a GET request to the `/games` endpoint of the CurseForge API
     * </p>
     *
     * @return the list of all the {@link Game}s available on the CurseForge platform
     * @throws CurseForgeApiGetException error while performing GET request
     */
    private static List<Game> getCurseForgeGames()
        throws CurseForgeApiGetException {
        URI uri = URIBuilder.newBuilder()
            .endpoint(API_BASE_URL + "games")
            .build();

        return CurseForgeGet(
            uri,
            TypeFactory.defaultInstance()
                .constructCollectionType(ArrayList.class, Game.class)
        );
    }

    /**
     * Obtains all the categories available on the CurseForge platform
     * <p>
     *     Sends a GET request to the `/categories` endpoint of the CurseForge API
     * </p>
     *
     * @return the list of all {@link Category}s available on the CurseForge platform
     * @throws CurseForgeApiGetException error while performing GET request
     */
    private static List<Category> getCurseForgeCategories()
        throws CurseForgeApiGetException {
        int minecraftId = TmodClient.getCurseForgeMinecraftId();

        URI uri = URIBuilder.newBuilder()
            .endpoint(API_BASE_URL + "categories")
            .appendPair("gameId", String.valueOf(minecraftId))
            .build();

        return CurseForgeGet(
            uri,
            TypeFactory.defaultInstance()
                .constructCollectionType(ArrayList.class, Category.class)
        );
    }

    /**
     * Obtains the inner id of Minecraft on CurseForge platform
     * <p>
     *     Firstly fetches all the games via {@link TmodClient#getCurseForgeGames()},
     *     and then searches for the Minecraft and maps the value to get the id
     * </p>
     *
     * @return the inner id of Minecraft on the CurseForge platform, or 432 if failed,
     * or if Minecraft wasn't present on the game list
     * @throws CurseForgeApiGetException error while performing GET request
     */
    private static int getCurseForgeMinecraftId()
        throws CurseForgeApiGetException {
        List<Game> games = TmodClient.getCurseForgeGames();

        return games
            .stream()
            .filter(
                game ->
                    Objects.equals(game.slug(), "minecraft") ||
                    Objects.equals(game.name(), "Minecraft")
            )
            .findFirst()
            .map(Game::id)
            .orElse(432);
    }

    public static ModFileGetter newModFileGetter(Mod mod) {
        return new ModFileGetter(mod);
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
     * @return an object of type {@code T}
     * @throws CurseForgeApiGetException error while performing GET request
     */
    private static <T> T CurseForgeGet(URI endpoint, JavaType type)
        throws CurseForgeApiGetException {
        HttpRequest request = HttpRequest.newBuilder()
            .uri(endpoint)
            .header("Accept", "application/json")
            .header("x-api-key", API_KEY)
            .GET()
            .build();

        CurseForgeResponse<T> model;

        try {
            model = HttpGet(
                request,
                TypeFactory.defaultInstance()
                    .constructParametricType(CurseForgeResponse.class, type)
            );

            return model.getData();
        } catch (Exception e) {
            throw new CurseForgeApiGetException(e);
        }
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
     * @return an object of type {@code T}
     * @throws HttpGetException error while sending request or status code is not 200
     * @throws DeserializationException error deserializing response
     */
    private static <T> T HttpGet(HttpRequest request, JavaType type)
        throws HttpGetException, DeserializationException {
        HttpResponse<String> response;
        try {
            response = client.send(
                request,
                HttpResponse.BodyHandlers.ofString()
            );
        } catch (Exception e) {
            throw new HttpGetException(request.uri(), e);
        }

        if (response.statusCode() != 200) {
            throw new HttpGetException(request.uri(), response.statusCode());
        }

        // Deserialize the response
        ObjectMapper mapper = new ObjectMapper();
        try {
            return mapper.readValue(response.body(), type);
        } catch (JsonProcessingException e) {
            throw new DeserializationException(type, e);
        }
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

    /**
     * Helper class to parametrize file fetch for a mod
     */
    public static class ModFileGetter {

        private Mod mod;
        private Optional<String> gameVersion;
        private Optional<ModLoader> modLoader;
        private Optional<String> timestamp;

        public ModFileGetter(Mod mod) {
            this.mod = mod;
            this.gameVersion = Optional.empty();
            this.modLoader = Optional.empty();
            this.timestamp = Optional.empty();
        }

        /**
         * Search by Minecraft version
         */
        public ModFileGetter withGameVersion(String gameVersion) {
            this.gameVersion = Optional.of(gameVersion);
            return this;
        }

        /**
         * Search by Minecraft mod loader
         */
        public ModFileGetter withModLoader(ModLoader modLoader) {
            this.modLoader = Optional.of(modLoader);
            return this;
        }

        /**
         * Search by file timestamp
         */
        public ModFileGetter withTimestamp(String timestamp) {
            this.timestamp = Optional.of(timestamp);
            return this;
        }

        /**
         * Performs the GET request to https://api.curseforge.com/v1/mods/{modId}/files endpoint
         *
         * <p>
         *      <a href="https://docs.curseforge.com/rest-api/#get-mod-files">Documentation</a>
         * </p>
         *
         * @return the target file
         * @throws CurseForgeApiGetException couldn't perform the GET request
         * @throws NoSuchModFileException couldn't find the file with given timestamp
         * @throws NoFilesFetchedException no files were fetched from the server
         */
        public File get()
            throws CurseForgeApiGetException, NoSuchModFileException, NoFilesFetchedException {
            URIBuilder builder = URIBuilder.newBuilder()
                .endpoint(API_BASE_URL + "mods/" + mod.id() + "/files");

            if (this.gameVersion.isPresent()) {
                String gameVersion = this.gameVersion.get();
                builder = builder.appendPair("gameVersion", gameVersion);
            }

            if (this.modLoader.isPresent()) {
                ModLoader modLoader = this.modLoader.get();
                builder = builder.appendPair(
                    "modLoaderType",
                    String.valueOf(modLoader.getId())
                );
            }

            List<File> files = CurseForgeGet(
                builder.build(),
                TypeFactory.defaultInstance()
                    .constructCollectionLikeType(List.class, File.class)
            );

            if (files.isEmpty()) {
                throw new NoFilesFetchedException(this.mod);
            }

            // Sort files by timestamp in reverse order
            files.sort(Comparator.comparing(File::fileDate).reversed());

            if (this.timestamp.isPresent()) {
                // We need to search for a file with the specific timestamp
                String targetTimestamp = this.timestamp.get();

                Optional<File> targetFile = files
                    .stream()
                    .filter(file ->
                        Objects.equals(targetTimestamp, file.fileDate())
                    )
                    .findFirst();

                if (targetFile.isEmpty()) {
                    throw new NoSuchModFileException(mod, targetTimestamp);
                } else {
                    return targetFile.get();
                }
            } else {
                // We need the latest file
                return files.getFirst();
            }
        }
    }
}
