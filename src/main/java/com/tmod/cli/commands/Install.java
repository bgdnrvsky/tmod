package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.File;
import com.tmod.core.models.Mod;
import com.tmod.core.net.CurseForgeApiGetException;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import java.io.FileOutputStream;
import java.io.IOException;
import java.net.URI;
import java.net.URL;
import java.net.URISyntaxException;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.ByteBuffer;
import java.nio.channels.Channels;
import java.nio.channels.FileChannel;
import java.nio.channels.ReadableByteChannel;
import java.nio.file.Path;
import java.util.Map;
import picocli.CommandLine;

@CommandLine.Command(
    name = "install",
    aliases = { "i" },
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

    private void installMod(
        String slug,
        Configuration config,
        Map<String, DependencyInfo> locks
    ) throws IOException, CurseForgeApiGetException, URISyntaxException {
        Mod mod = TmodClient.searchModBySlug(slug);
        File file = TmodClient.newModFileGetter(mod)
            .withModLoader(config.loader())
            .withGameVersion(config.gameVersion())
            .withTimestamp(locks.get(slug).timestamp())
            .get();

        // Create the file
        java.io.File outputFile = new java.io.File(
            targetDirectoryPath.toString(),
            file.fileName()
        );

        boolean alreadyExists = !outputFile.createNewFile();

        if (!alreadyExists) {
            try
                (
                    ReadableByteChannel sourceChannel = Channels.newChannel(new URL(file.downloadUrl()).openStream());
                    FileOutputStream fileStream = new FileOutputStream(outputFile);
                    FileChannel fileChannel = fileStream.getChannel()
                )
            {
                fileChannel.transferFrom(sourceChannel, 0, file.fileLength());
            }
        }

        for (String dependencySlug : locks.get(slug).dependencies()) {
            installMod(dependencySlug, config, locks);
        }
    }

    @Override
    public void run() {
        Mapper mapper = new Mapper(parent.getRepoPath());
        Repository repository;

        try {
            repository = mapper.read();
        } catch (IOException e) {
            System.err.println(e.getMessage());
            return;
        }

        Configuration config = repository.getConfig();

        // Create the target repository
        java.io.File targetDirectoryFile = new java.io.File(
            targetDirectoryPath.toString()
        );
        targetDirectoryFile.mkdir();

        for (String slug : repository.getManuallyAdded()) {
            // Don't install the mod if it's client only and installing for server
            if (server) if (
                repository.getLocks().get(slug).clientOnly()
            ) continue;

            try {
                installMod(slug, config, repository.getLocks());
            } catch (
                CurseForgeApiGetException | IOException | URISyntaxException e
            ) {
                System.err.println(e.getMessage());
            }
        }
    }
}
