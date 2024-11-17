package com.tmod.cli.commands;

import com.beust.jcommander.Parameter;
import com.beust.jcommander.Parameters;
import com.tmod.core.models.Mod;
import com.tmod.core.net.CurseForgeClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;

import java.io.IOException;
import java.net.URISyntaxException;
import java.util.function.Consumer;

@Parameters(commandNames="add", commandDescription="Add minecraft mod to the repo")
public class CAdd implements ICommand {

    @Parameter(description="<mod id/slug>", required=true)
    private String target;

    @Parameter(names={"-c", "--client-only"}, description="Mark mod as client only, it (and it's dependencies) will be ignored when installing with '--server'")
    private boolean clientOnly = false;

    @Override
    public void onExecute(Options options) {
        try {
            // Repository repo = Mapper.read(options.getRepositoryPath());

            Mod mod = CurseForgeClient.searchModById(target);
            System.out.println(mod);

        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
