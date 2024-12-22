package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.File;
import com.tmod.core.models.Mod;
import com.tmod.core.models.RelationType;
import com.tmod.core.net.CurseForgeApiGetException;
import com.tmod.core.net.NoFilesFetchedException;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import java.util.stream.Collectors;
import picocli.CommandLine;

@CommandLine.Command(
    name = "add",
    description = "Add minecraft mod to the repo"
)
public class Add implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(
        paramLabel = "<mod id/slug>",
        description = "Search using mod id, or mod's 'slug' (slug is not always the same as the mod name)"
    )
    private String target;

    @CommandLine.Option(
        names = { "-c", "--client-only" },
        description = "Mark mod as client only, it (and it's dependencies) will be ignored when installing with '--server'",
        defaultValue = "false"
    )
    private boolean clientOnly = false;

    /**
     * Adds a mod to the locks registry, as well as all of its dependencies
     *
     * @throws NoFilesFetchedException if couldn't find any files for a mod
     * @throws CurseForgeApiGetException if couldn't perform the GET request
     */
    private void recursivelyAddToLocks(Mod mod, Repository repository)
        throws NoFilesFetchedException, CurseForgeApiGetException {
        Configuration config = repository.getConfig();

        File modFile = TmodClient.newModFileGetter(mod)
            .withGameVersion(config.gameVersion())
            .withModLoader(config.loader())
            .get();

        java.util.List<Mod> dependencies = modFile
            .relations()
            .stream()
            .filter(
                relation ->
                    relation.relationType() ==
                        RelationType.RequiredDependency ||
                    relation.relationType() == RelationType.EmbeddedLibrary
            )
            .map(relation -> TmodClient.searchModById(relation.modId()))
            .collect(Collectors.toList());

        DependencyInfo dependencyInfo = new DependencyInfo(
            modFile.fileDate(),
            clientOnly,
            dependencies.stream().map(Mod::slug).collect(Collectors.toList())
        );

        repository.getLocks().put(mod.slug(), dependencyInfo);

        for (Mod dependency : dependencies) {
            recursivelyAddToLocks(dependency, repository);
        }
    }

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();
            Mod mod;

            try {
                mod = TmodClient.searchModById(Integer.parseInt(target));
            } catch (NumberFormatException e) {
                mod = TmodClient.searchModBySlug(target);
            }

            repository.getManuallyAdded().add(mod.slug());
            recursivelyAddToLocks(mod, repository);

            mapper.write(repository);

            System.out.println(mod);
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
