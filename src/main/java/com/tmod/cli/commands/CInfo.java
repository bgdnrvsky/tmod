package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;
import com.beust.jcommander.Parameters;

@Parameters(commandNames="info", commandDescription="Search a remote mod and print its info")
public class CInfo implements ICommand {

    @Parameter(description="<mod id/slug>", required=true)
    private String target;

    @Override
    public void onExecute(Options options) {
        System.out.println("Printing mod info");
        System.out.println(target);
    }
}
