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
import java.util.AbstractMap.SimpleEntry;
import java.util.HashMap;
import java.util.List;
import java.util.Map.Entry;
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
     */
    private void addAllToLocks(
        HashMap<Mod, Entry<File, List<Mod>>> allModsToAddWithInfo,
        Repository repository
    ) {
        for (HashMap.Entry<
            Mod,
            Entry<File, List<Mod>>
        > entry : allModsToAddWithInfo.entrySet()) {
            Mod mod = entry.getKey();
            File file = entry.getValue().getKey();
            List<Mod> dependencies = entry.getValue().getValue();

            DependencyInfo dependencyInfo = new DependencyInfo(
                file.fileDate(),
                clientOnly,
                dependencies
                    .stream()
                    .map(Mod::slug)
                    .collect(Collectors.toList())
            );
            repository.getLocks().put(mod.slug(), dependencyInfo);
        }
    }

    /**
     * Constructs the tree of every other mod that needs to be added
     * if the user wants to add his mod.
     * <p>
     *  HashMap<(the mod to add), Entry<(its file), (its dependencies)>>
     * </p>
     *
     * @throws NoFilesFetchedException couldn't fetch any file for a mod
     * @throws CurseForgeApiGetException error while GETting from CurseForge
     */
    private HashMap<Mod, Entry<File, List<Mod>>> getAllModsToAdd(
        Mod mod,
        Repository repository
    ) throws NoFilesFetchedException, CurseForgeApiGetException {
        Configuration config = repository.getConfig();

        File modFile = TmodClient.newModFileGetter(mod)
            .withGameVersion(config.gameVersion())
            .withModLoader(config.loader())
            .get();

        List<Mod> dependencies = modFile
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

        HashMap<Mod, Entry<File, List<Mod>>> modsToAdd = new HashMap<>();

        modsToAdd.put(mod, new SimpleEntry<>(modFile, dependencies));

        for (Mod dependency : dependencies) {
            modsToAdd.putAll(getAllModsToAdd(dependency, repository));
        }

        return modsToAdd;
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

            HashMap<Mod, Entry<File, List<Mod>>> modsToAddWithInfo =
                getAllModsToAdd(mod, repository);

            repository.getManuallyAdded().add(mod.slug());
            addAllToLocks(modsToAddWithInfo, repository);

            mapper.write(repository);

            System.out.println(mod);
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
