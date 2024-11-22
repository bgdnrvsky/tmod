package com.tmod.cli.commands;

import picocli.CommandLine;

@CommandLine.Command(
        name = "tree",
        description = "Print the tree of added mods and dependencies"
)
public class Tree implements Runnable {
    @Override
    public void run() {
        System.out.println("Printing mods tree");
    }
}
