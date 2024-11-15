package core.models;

import java.net.URL;
import java.util.Date;

public record Category(
        int id,
        int gameId,
        String name,
        String slug,
        URL url,
        URL iconUrl,
        Date dateModified,
        boolean isClass,
        int classId,
        int parentCategoryId,
        int displayIndex
) {}
