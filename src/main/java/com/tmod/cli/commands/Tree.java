package com.tmod.cli.commands;

import com.tmod.cli.App;
import picocli.CommandLine;

@CommandLine.Command(
        name = "tree",
        description = "Print the tree of added mods and dependencies"
)
public class Tree implements Runnable {
    @CommandLine.ParentCommand
    private App parent;

    @Override
    public void run() {
        System.out.println("Printing mods tree");
    }
}
