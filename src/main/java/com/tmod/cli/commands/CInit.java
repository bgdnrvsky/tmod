package com.tmod.cli.commands;

import com.beust.jcommander.Parameters;

@Parameters(commandNames="init", commandDescription="Initialize a new repo")
public class CInit implements ICommand {

    @Override
    public void onExecute(Options options) {
        System.out.println("Initializing empty tmod repository");
    }
}
