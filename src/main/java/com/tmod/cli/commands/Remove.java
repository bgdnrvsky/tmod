package com.tmod.cli.commands;

import com.tmod.cli.App;
import picocli.CommandLine;

import java.util.Set;

@CommandLine.Command(
        name = "remove",
        description = "Remove one or multiple mods from the repo"
)
public class Remove implements Runnable {
    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(
        arity = "1..*",
        paramLabel = "slugs"
    )
    private Set<String> removalTargetMods;

    @Override
    public void run() {
        System.out.println("Removing mods from repo");
        System.out.println(removalTargetMods);
    }
}
