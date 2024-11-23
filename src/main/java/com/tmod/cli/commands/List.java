package com.tmod.cli.commands;

import com.tmod.cli.App;
import picocli.CommandLine;

@CommandLine.Command(
        name = "list",
        description = "List the mods in the repo"
)
public class List implements Runnable  {
    @CommandLine.ParentCommand
    private App parent;

    @Override
    public void run() {
        System.out.println("Listing mods in the repository");
    }
}
