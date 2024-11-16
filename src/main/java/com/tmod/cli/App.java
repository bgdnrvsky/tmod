package com.tmod.cli;

import com.beust.jcommander.JCommander;
import com.beust.jcommander.ParameterException;
import com.tmod.cli.commands.*;

/**
 * Main class for the command line interface version of tmod
 */
public class App {

    /**
     * Entry point for the CLI version of tmod
     *
     * @param argv command line parameters
     */
    public static void main(String[] argv) {

        Options options = new Options();

        // Create argument parser
        JCommander commander = JCommander.newBuilder()
                .addObject(options)
                .addCommand(new CList())
                .addCommand(new CInit())
                .addCommand(new CAdd())
                .addCommand(new CRemove())
                .addCommand(new CInfo())
                .addCommand(new CInstall())
                .addCommand(new CTree())
                .build();

        commander.setProgramName("tmod");

        // Parse and print error if unknown command/option is given
        try {
            commander.parse(argv);
        } catch (ParameterException e) {
            System.err.println(e.getMessage());
            commander.usage();
            return;
        }

        if (options.requestedHelp()) {
            commander.usage();
        }

        // Execute corresponding command
        dispatchCommands(commander);
    }

    /**
     * Execute the appropriate command according to the user arguments
     *
     * @param commander {@link JCommander} contains all command objects and parsed arguments
     */
    public static void dispatchCommands(JCommander commander) {
        String parsedCommand = commander.getParsedCommand();

        if (parsedCommand == null) {
            commander.usage();
            return;
        }

        ICommand command = (ICommand) commander
                .getCommands()
                .get(parsedCommand)
                .getObjects()
                .getFirst();

        command.onExecute((Options) commander.getObjects().getFirst());
    }
}
