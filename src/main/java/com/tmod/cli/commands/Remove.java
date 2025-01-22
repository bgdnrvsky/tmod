package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.File;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
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
        names = { "-r", "--remove" },
        description = "Remove the mod from the folder as well",
        defaultValue = "false",
        showDefaultValue = CommandLine.Help.Visibility.ALWAYS
    )
    private boolean removeFromFolder = false;

    @CommandLine.Parameters(arity = "1..*", paramLabel = "slugs")
    private Set<String> removalTargetMods;

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();
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

                DependencyInfo dependencyInfo = repository
                    .getLocks()
                    .remove(slug);

                if (removeFromFolder) {
                    // Remove the file from the folder
                    Mod mod = TmodClient.searchModBySlug(slug);
                    File modFile = TmodClient.newModFileGetter(mod)
                        .withGameVersion(config.gameVersion())
                        .withModLoader(config.loader())
                        .withTimestamp(dependencyInfo.timestamp())
                        .get();

                    java.io.File actualFile = new java.io.File(
                        targetDirectoryPath.toString(),
                        modFile.fileName()
                    );

                    if (!actualFile.delete()) {
                        System.err.println(
                            String.format(
                                "Couldn't delete the file '%s'",
                                actualFile.getCanonicalPath()
                            )
                        );
                    }
                }
            }

            mapper.write(repository);
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
