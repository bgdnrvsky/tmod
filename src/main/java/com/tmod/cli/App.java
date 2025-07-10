package com.tmod.cli;

import com.tmod.cli.commands.*;
import java.nio.file.Path;
import java.util.Arrays;

import com.tmod.gui.TModGui;
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

    @CommandLine.Option(
            names = { "--gui" },
            description = "Launch the graphical user interface instead of the command line interface",
            defaultValue = "false",
            showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
    )
    private boolean gui = false;

    private Path repoPath = Path.of(".tmod");

    public static void main(String[] args) {
        App app = new App();

        AnsiConsole.systemInstall();
        int exitCode = new CommandLine(app).execute(args);
        AnsiConsole.systemUninstall();

        if (app.gui) {
            TModGui.launch(TModGui.class, args);
            return;
        }

        System.exit(exitCode);
    }



    public Path getRepoPath() {
        return repoPath;
    }

    public boolean isQuiet() {
        return quiet;
    }
}
