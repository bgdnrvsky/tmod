package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.File;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import java.io.FileOutputStream;
import java.net.URI;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.file.Path;
import java.util.Map;
import picocli.CommandLine;

@CommandLine.Command(
    name = "install",
    description = "Download all the mods to a target folder"
)
public class Install implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Option(
        names = { "-s", "--server" },
        description = "Do not install client only mods",
        defaultValue = "false"
    )
    private boolean server = false;

    @CommandLine.Option(
        names = { "-o", "--out-dir" },
        paramLabel = "<Path>",
        description = "The target folder",
        defaultValue = "mods/",
        showDefaultValue = CommandLine.Help.Visibility.ALWAYS
    )
    private Path targetDirectoryPath = Path.of("mods/");

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();
            Configuration config = repository.getConfig();

            // Create the target repository
            java.io.File targetDirectoryFile = new java.io.File(
                targetDirectoryPath.toString()
            );
            targetDirectoryFile.mkdir();

            for (Map.Entry<String, DependencyInfo> entry : repository
                .getLocks()
                .entrySet()) {
                // Don't install the mod if it's client only and installing for server
                if (server) if (entry.getValue().clientOnly()) continue;

                Mod mod = TmodClient.searchModBySlug(entry.getKey());
                File file = TmodClient.newModFileGetter(mod)
                    .withModLoader(config.loader())
                    .withGameVersion(config.gameVersion())
                    .withTimestamp(entry.getValue().timestamp())
                    .get();

                // Create the file
                java.io.File outputFile = new java.io.File(
                    targetDirectoryFile,
                    file.fileName()
                );

                boolean alreadyExists = !outputFile.createNewFile();

                if (alreadyExists) continue;

                // Download the file
                HttpRequest request = HttpRequest.newBuilder(
                    new URI(file.downloadUrl())
                ).build();
                HttpResponse<String> downloadedFile = TmodClient.HttpGet(
                    request
                );

                FileOutputStream stream = new FileOutputStream(outputFile);
                FileChannel channel = stream.getChannel();

                ByteBuffer buffer = ByteBuffer.wrap(
                    downloadedFile.body().getBytes()
                );

                channel.write(buffer);

                channel.close();
                stream.close();
            }
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
