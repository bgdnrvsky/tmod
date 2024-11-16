package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;
import com.beust.jcommander.Parameters;

@Parameters(commandNames="install", commandDescription="Download all the mods to the folder")
public class CInstall implements ICommand {

    @Parameter(names={ "-s", "--server" }, description="Do not install client only mods (and dependencies)")
    private boolean server;

    @Parameter(names={ "-o", "--out-dir" })
    private String targetDirectory = "mods";

    @Override
    public void onExecute(Options options) {
        System.out.println("Installing added mods");
        System.out.println(server);
        System.out.println(targetDirectory);
    }
}
