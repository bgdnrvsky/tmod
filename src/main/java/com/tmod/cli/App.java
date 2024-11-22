package com.tmod.cli;

import com.tmod.cli.commands.*;
import picocli.CommandLine;

@CommandLine.Command(
        name="tmod",
        description = "Takes care of your mods",
        subcommands = {
                CommandLine.HelpCommand.class,
                Add.class,
                Info.class,
                Init.class,
                Install.class,
                List.class,
                Remove.class,
                Tree.class
        }
)
public class App {
    public static void main(String[] args) {
        int exitCode = new CommandLine(new App()).execute(args);
        System.exit(exitCode);
    }
}