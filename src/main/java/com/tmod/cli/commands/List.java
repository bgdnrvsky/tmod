package com.tmod.cli.commands;

import picocli.CommandLine;

@CommandLine.Command(
        name = "list",
        description = "List the mods in the repo"
)
public class List implements Runnable  {
    @Override
    public void run() {
        System.out.println("Listing mods in the repository");
    }
}
