package com.tmod.cli.commands;

import com.beust.jcommander.Parameters;

@Parameters(commandNames="list", commandDescription="List the mods in the repo")
public class CList implements ICommand {

    @Override
    public void onExecute(Options options) {
        System.out.println("Listing mods in the repository");
    }
}
