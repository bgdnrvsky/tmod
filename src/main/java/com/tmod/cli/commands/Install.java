package com.tmod.cli.commands;

import picocli.CommandLine;

@CommandLine.Command(
        name = "install",
        description = "Download all the mods to a target folder"
)
public class Install implements Runnable {
    @CommandLine.Option(
            names = { "-s", "--server" },
            description = "Do not install client only mods",
            defaultValue = "false"
    )
    private boolean server = false;

    @CommandLine.Option(
            names = { "-o", "--out-dir" },
            paramLabel = "<Path>",
            description = "The target folder",
            defaultValue = "mods/",
            showDefaultValue = CommandLine.Help.Visibility.ALWAYS
    )
    private String targetDirectoryPath = "mods/";

    @Override
    public void run() {
        System.out.println("Installing added mods");
        System.out.println(server);
        System.out.println(targetDirectoryPath);
    }
}
