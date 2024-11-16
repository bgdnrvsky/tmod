package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;

import java.nio.file.Path;


public class Options {

    @Parameter(names="--repo")
    private String repository = ".tmod";

    @Parameter(names={"-q","--quiet"}, description="Do not print Tmod log messages")
    private boolean quiet = false;

    @Parameter(names={"-h", "--help"}, description="Print help", help=true)
    private boolean help;

    public Path getRepositoryPath() {
        return Path.of(repository);
    }

    public boolean isQuiet() {
        return quiet;
    }

    public boolean requestedHelp() {
        return help;
    }
}
