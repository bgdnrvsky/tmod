package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import picocli.CommandLine;

@CommandLine.Command(name = "list", description = "List the mods in the repo")
public class List implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();

            int i = 1;

            for (String slug : repository.getManuallyAdded()) {
                Mod mod = TmodClient.searchModBySlug(slug);

                System.out.println(
                    String.format(
                        "%d. %s(%d): %s",
                        i,
                        mod.name(),
                        mod.id(),
                        mod.summary()
                    )
                );

                i += 1;
            }
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
