package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import java.util.Set;
import picocli.CommandLine;

@CommandLine.Command(
    name = "remove",
    description = "Remove one or multiple mods from the repo"
)
public class Remove implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(arity = "1..*", paramLabel = "slugs")
    private Set<String> removalTargetMods;

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();

            for (String slug : removalTargetMods) {
                boolean removedFromManuallyAdded = repository
                    .getManuallyAdded()
                    .remove(slug);
                boolean removedFromLocks =
                    repository.getLocks().remove(slug) != null;
                boolean removedFromRepo =
                    removedFromManuallyAdded || removedFromLocks;

                if (!removedFromRepo) {
                    System.out.println(
                        String.format(
                            "The mod '%s' wasn't present in the repo",
                            slug
                        )
                    );
                }
            }

            mapper.write(repository);
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
