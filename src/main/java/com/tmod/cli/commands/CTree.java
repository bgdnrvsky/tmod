package com.tmod.cli.commands;

import com.beust.jcommander.Parameters;

@Parameters(commandNames="tree", commandDescription="Print the tree of added mods and dependencies")
public class CTree implements ICommand {

    @Override
    public void onExecute(Options options) {
        System.out.println("Printing mods tree");
    }
}
