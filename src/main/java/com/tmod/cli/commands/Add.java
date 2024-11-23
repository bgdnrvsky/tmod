package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import picocli.CommandLine;

@CommandLine.Command(
        name = "add",
        description = "Add minecraft mod to the repo"
)
public class Add implements Runnable {
    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(
            paramLabel="<mod id/slug>",
            description = "Search using mod id, or mod's 'slug' (slug is not always the same as the mod name)"
    )
    private String target;

    @CommandLine.Option(
        names = {"-c", "--client-only"},
        description="Mark mod as client only, it (and it's dependencies) will be ignored when installing with '--server'",
        defaultValue = "false"
    )
    private boolean clientOnly = false;

    @Override
    public void run() {
        try {
            // Repository repo = Mapper.read(options.getRepositoryPath());
            Mod mod = TmodClient.searchModById(Integer.parseInt(target));

            System.out.println(mod);
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
