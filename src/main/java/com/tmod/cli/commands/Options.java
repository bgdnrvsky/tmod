package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;


public class Options {

    @Parameter(names="--repo")
    private String repository = ".tmod";

    @Parameter(names={"-q","--quiet"}, description="Do not print Tmod log messages")
    private boolean quiet = false;

    @Parameter(names={"-h", "--help"}, description="Print help", help=true)
    private boolean help;

    public String getRepository() {
        return repository;
    }

    public boolean isQuiet() {
        return quiet;
    }

    public boolean requestedHelp() {
        return help;
    }
}
