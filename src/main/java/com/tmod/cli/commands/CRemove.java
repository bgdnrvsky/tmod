package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;
import com.beust.jcommander.Parameters;

import java.util.Set;

@Parameters(commandNames="remove", commandDescription="Remove one or multiple mods from the repo")
public class CRemove implements ICommand {

    @Parameter(description="<mod id/slug> <mod id/slug> ...", required=true)
    private Set<String> mods;


    @Override
    public void onExecute(Options options) {
        System.out.println("Removing mods from repository");
        System.out.println(mods);
    }
}
