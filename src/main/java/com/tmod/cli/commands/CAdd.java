package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;
import com.beust.jcommander.Parameters;

@Parameters(commandNames="add", commandDescription="Add minecraft mod to the repo")
public class CAdd implements ICommand {

    @Parameter(description="<mod id/slug>", required=true)
    private String target;

    @Parameter(names={"-c", "--client-only"}, description="Mark mod as client only, it (and it's dependencies) will be ignored when installing with '--server'")
    private boolean clientOnly = false;

    @Override
    public void onExecute(Options options) {
        System.out.println("Adding to repository");
        System.out.println(target);
        System.out.println(clientOnly);
    }
}
