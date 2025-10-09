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
import java.io.IOException;
import java.nio.file.Path;
import java.util.Set;
import org.fusesource.jansi.Ansi;
import org.fusesource.jansi.AnsiConsole;
import org.fusesource.jansi.AnsiPrintStream;
import picocli.CommandLine;

@CommandLine.Command(
    name = "remove",
    aliases = { "rm" },
    description = "Remove one or multiple mods from the repo"
)
public class Remove implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Option(
        names = { "-f", "--from" },
        paramLabel = "<Path>",
        description = "The folder with mods",
        defaultValue = "mods/",
        showDefaultValue = CommandLine.Help.Visibility.ALWAYS
    )
    private Path targetDirectoryPath = Path.of("mods/");

    @CommandLine.Option(
        names = { "-k", "--keep-file" },
        description = "Don't remove the mod from the output folder",
        defaultValue = "false",
        showDefaultValue = CommandLine.Help.Visibility.ALWAYS
    )
    private boolean keepTheModFile = false;

    @CommandLine.Parameters(arity = "1..*", paramLabel = "slugs")
    private Set<String> removalTargetMods;

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

        for (String slug : removalTargetMods) {
            boolean removedFromManuallyAdded = repository
                .getManuallyAdded()
                .remove(slug);

            if (!removedFromManuallyAdded) {
                Ansi msg = new Ansi();

                msg
                    .a("The mod ")
                    .fgRed()
                    .a(slug)
                    .fgDefault()
                    .a(" wasn't present in the repo");

                try (AnsiPrintStream stream = AnsiConsole.out()) {
                    stream.println(msg);
                }
                continue;
            }

            DependencyInfo dependencyInfo = repository.getLocks().remove(slug);

            if (!keepTheModFile) {
                // Remove the file from the folder
                Mod mod = TmodClient.searchModBySlug(slug);
                File modFile;
                try {
                    modFile = TmodClient.newModFileGetter(mod)
                        .withGameVersion(config.gameVersion())
                        .withModLoader(config.loader())
                        .withTimestamp(dependencyInfo.timestamp())
                        .get();
                } catch (CurseForgeApiGetException e) {
                    System.err.println(e.getMessage());
                    continue;
                }

                java.io.File actualFile = new java.io.File(
                    targetDirectoryPath.toString(),
                    modFile.fileName()
                );

                if (!actualFile.delete()) {
                    try {
                        System.err.println(
                            String.format(
                                "Couldn't delete the file '%s'",
                                actualFile.getCanonicalPath()
                            )
                        );
                    } catch (IOException e) {
                        System.err.println(e.getMessage());
                    }
                }
            }
        }

        try {
            mapper.write(repository);
        } catch (IOException e) {
            System.err.println(e.getMessage());
        }
    }
}
