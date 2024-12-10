package com.tmod.cli;

import com.tmod.cli.commands.*;
import java.nio.file.Path;
import picocli.CommandLine;

@CommandLine.Command(
    name = "tmod",
    description = "Takes care of your mods",
    subcommands = {
        CommandLine.HelpCommand.class,
        Add.class,
        Info.class,
        Init.class,
        Install.class,
        List.class,
        Remove.class,
        Tree.class,
    }
)
public class App {

    @CommandLine.Option(
        names = { "-r", "--repo" },
        paramLabel = "<Path>",
        description = "Change the default repository path",
        defaultValue = ".tmod",
        showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
    )
    private Path repoPath = Path.of(".tmod");

    @CommandLine.Option(
        names = { "-q", "--quiet" },
        description = "Silence tmod",
        defaultValue = "false",
        showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
    )
    private boolean quiet = false;

    public static void main(String[] args) {
        int exitCode = new CommandLine(new App()).execute(args);
        System.exit(exitCode);
    }

    public Path getRepoPath() {
        return repoPath;
    }

    public boolean isQuiet() {
        return quiet;
    }
}
