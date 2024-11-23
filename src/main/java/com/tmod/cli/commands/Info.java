package com.tmod.cli.commands;

import com.tmod.cli.App;
import picocli.CommandLine;

@CommandLine.Command(
        name = "info",
        description = "Search a remote mod and print its info"
)
public class Info implements Runnable {
    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(
            paramLabel="<mod id/slug>",
            description = "Search using mod id, or mod's 'slug' (slug is not always the same as the mod name)"
    )
    private String target;

    @Override
    public void run() {
        System.out.println("Printing mod info");
        System.out.println(target);
    }
}
