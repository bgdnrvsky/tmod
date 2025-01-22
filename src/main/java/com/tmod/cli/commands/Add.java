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
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.stream.Collectors;
import org.fusesource.jansi.Ansi;
import org.fusesource.jansi.Ansi.Attribute;
import org.fusesource.jansi.AnsiConsole;
import org.fusesource.jansi.AnsiPrintStream;
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

    /**
     * Checks if any of the mods that need to be added to the repository (to locks)
     * is incompatible with any of the mods already present in the repository (locks)
     *
     * @param allModsToAddWithInfo map of all the mods that need to be added
     * @param repository current state of the repository
     * @return `Optional.empty` if no confict, `Optional.of` that has the mod that has to be added but causes a conflict
     * @throws CurseForgeApiGetException error performing GET request to CurseForge
     */
    private Optional<Mod> anyModIsIncompatibleWithRepo(
        HashMap<Mod, Entry<File, List<Mod>>> allModsToAddWithInfo,
        Repository repository
    ) throws CurseForgeApiGetException {
        Configuration config = repository.getConfig();

        // Check if any mod from repo is incompatible with any mod that has to be added
        for (Entry<String, DependencyInfo> entry : repository
            .getLocks()
            .entrySet()) {
            Mod mod = TmodClient.searchModBySlug(entry.getKey());
            File file = TmodClient.newModFileGetter(mod)
                .withModLoader(config.loader())
                .withGameVersion(config.gameVersion())
                .withTimestamp(entry.getValue().timestamp())
                .get();

            List<Mod> incompatibilities = file
                .relations()
                .stream()
                .filter(
                    relation ->
                        relation.relationType() == RelationType.Incompatible
                )
                .map(relation -> TmodClient.searchModById(relation.modId()))
                .collect(Collectors.toList());

            boolean overlaps = !Collections.disjoint(
                incompatibilities,
                allModsToAddWithInfo.keySet()
            );

            if (overlaps) {
                return Optional.of(mod);
            }
        }

        // Check if any mod that needs to be added is incompatible with the repo
        for (HashMap.Entry<
            Mod,
            Entry<File, List<Mod>>
        > entry : allModsToAddWithInfo.entrySet()) {
            Mod mod = entry.getKey();
            File file = entry.getValue().getKey();

            List<String> incompatibilities = file
                .relations()
                .stream()
                .filter(
                    relation ->
                        relation.relationType() == RelationType.Incompatible
                )
                .map(relation -> TmodClient.searchModById(relation.modId()))
                .map(Mod::slug)
                .collect(Collectors.toList());

            // Check if any of incompatible mods are already present in the repo
            boolean overlaps = !Collections.disjoint(
                repository.getLocks().keySet(),
                incompatibilities
            );

            if (overlaps) {
                return Optional.of(mod);
            }
        }

        return Optional.empty();
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

            Optional<Mod> conflict = anyModIsIncompatibleWithRepo(
                modsToAddWithInfo,
                repository
            );

            if (conflict.isPresent()) {
                System.err.println(
                    "The mod " + conflict.get() + " conflicts with other mod"
                );
                return;
            }

            repository.getManuallyAdded().add(mod.slug());
            addAllToLocks(modsToAddWithInfo, repository);

            mapper.write(repository);

            Ansi msg = new Ansi();

            msg
                .fgBlue()
                .a(mod.name())
                .fgDefault()
                .format("(%d) - ", mod.id())
                .a(Attribute.ITALIC)
                .a(mod.summary())
                .a(Attribute.ITALIC_OFF);

            try (AnsiPrintStream stream = AnsiConsole.out()) {
                stream.println(msg);
            }
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
