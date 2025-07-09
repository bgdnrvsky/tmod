package com.tmod.cli;

import com.tmod.cli.commands.*;
import java.nio.file.Path;

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
        names = { "-r", "--repo" },
        paramLabel = "<Path>",
        description = "Change the default repository path",
        defaultValue = ".tmod",
        showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
    )
    private Path repoPath = Path.of(".tmod");

    @CommandLine.Option(
//        names = { "-q", "--quiet" },
//        description = "Silence tmod",
//        defaultValue = "false",
//        showDefaultValue = CommandLine.Help.Visibility.ON_DEMAND
            names = {"--gui"},
        description = "Launch the graphical user interface instead of the command line interface",
        defaultValue = "false"
    )

    private boolean quiet = false;
    private boolean gui= false;

    public static void main(String[] args) {
        App app = new App();

        // 2 Options of running the application:
        /// GUI
        if (args.length > 0 && args[0].equals("--gui"))
            TModGui.launch(TModGui.class, args);

        /// CLI
         else {
            AnsiConsole.systemInstall();
            int exitCode = new CommandLine(app).execute(args);
            AnsiConsole.systemUninstall();
            System.exit(exitCode);
        }
    }

    public Path getRepoPath() {
        return repoPath;
    }

    public boolean isQuiet() {
        return quiet;
    }
}
