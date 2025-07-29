package com.tmod.cli;

import com.tmod.cli.commands.*;
import java.nio.file.Path;
import org.fusesource.jansi.AnsiConsole;
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
        names = { "-q", "--quiet" },
        description = "Silence tmod",
        defaultValue = "false",
        showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
    )
    private boolean quiet = false;

    private Path repoPath = Path.of(".tmod");

    public static void main(String[] args) {
        App app = new App();

        AnsiConsole.systemInstall();
        int exitCode = new CommandLine(app).execute(args);
        AnsiConsole.systemUninstall();

        System.exit(exitCode);
    }

    public Path getRepoPath() {
        return repoPath;
    }

    public boolean isQuiet() {
        return quiet;
    }
}
