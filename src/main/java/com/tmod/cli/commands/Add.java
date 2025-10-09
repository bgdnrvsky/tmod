package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.File;
import com.tmod.core.models.Mod;
import com.tmod.core.models.RelationType;
import com.tmod.core.net.CurseForgeApiGetException;
import com.tmod.core.net.CurseForgeModSearchException;
import com.tmod.core.net.NoFilesFetchedException;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.Configuration;
import com.tmod.core.repo.models.DependencyInfo;
import java.io.IOException;
import java.util.AbstractMap.SimpleEntry;
import java.util.Collection;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
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
    aliases = { "a" },
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
     * Constructs the tree of every other mod that needs to be added
     * if the user wants to add his mod.
     * <p>
     *  Map<(the mod to add), Entry<(its file), (its dependencies)>>
     * </p>
     *
     * @throws NoFilesFetchedException couldn't fetch any file for a mod
     * @throws CurseForgeApiGetException error while GETting from CurseForge
     */
    private Map<Mod, Entry<File, List<Mod>>> getAllModsToAdd(
        final Mod mod,
        final Repository repository
    ) throws NoFilesFetchedException, CurseForgeApiGetException {
        final Configuration config = repository.getConfig();

        final File modFile = TmodClient.newModFileGetter(mod)
            .withGameVersion(config.gameVersion())
            .withModLoader(config.loader())
            .get();

        final List<Mod> dependencies = modFile
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

        final Map<Mod, Entry<File, List<Mod>>> modsToAdd = new HashMap<>();

        modsToAdd.put(mod, new SimpleEntry<>(modFile, dependencies));

        for (final Mod dependency : dependencies) {
            modsToAdd.putAll(getAllModsToAdd(dependency, repository));
        }

        return modsToAdd;
    }

    /**
     * Checks if a mod file is incompatible with any mods from the provided collection.
     * <p>
     * This method extracts all the incompatible relations from the mod file and checks
     * if any of those incompatible mods are present in the provided collection.
     *
     * @param modFile the mod file to check incompatibilities for
     * @param modsToCheckAgainst collection of mod slugs to check against
     * @return true if the mod is incompatible with any of the mods in the collection, false otherwise
     */
    private boolean modIsIncompatibleWithAnyFrom(
        final File modFile,
        final Collection<String> modsToCheckAgainst
    ) {
        final List<String> incompatibilities = modFile
            .relations()
            .stream()
            .filter(
                relation -> relation.relationType() == RelationType.Incompatible
            )
            .map(relation -> TmodClient.searchModById(relation.modId()))
            .map(Mod::slug)
            .collect(Collectors.toList());

        return !Collections.disjoint(modsToCheckAgainst, incompatibilities);
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
        final Map<Mod, Entry<File, List<Mod>>> allModsToAddWithInfo,
        final Repository repository
    ) throws CurseForgeApiGetException {
        final Configuration config = repository.getConfig();

        // Check if any mod from repo is incompatible with any mod that has to be added
        for (var entry : repository.getLocks().entrySet()) {
            final Mod mod = TmodClient.searchModBySlug(entry.getKey());
            final File file = TmodClient.newModFileGetter(mod)
                .withModLoader(config.loader())
                .withGameVersion(config.gameVersion())
                .withTimestamp(entry.getValue().timestamp())
                .get();

            if (
                modIsIncompatibleWithAnyFrom(
                    file,
                    allModsToAddWithInfo
                        .keySet()
                        .stream()
                        .map(Mod::slug)
                        .collect(Collectors.toList())
                )
            ) {
                return Optional.of(mod);
            }
        }

        // Check if any mod that needs to be added is incompatible with the repo
        for (var entry : allModsToAddWithInfo.entrySet()) {
            final Mod mod = entry.getKey();
            final File file = entry.getValue().getKey();

            if (
                modIsIncompatibleWithAnyFrom(
                    file,
                    repository.getLocks().keySet()
                )
            ) {
                return Optional.of(mod);
            }
        }

        return Optional.empty();
    }

    @Override
    public void run() {
        final Mapper mapper = new Mapper(parent.getRepoPath());
        Repository repository;

        try {
            repository = mapper.read();
        } catch (final IOException e) {
            System.err.println(e.getMessage());
            return;
        }

        Mod mod;

        try {
            try {
                mod = TmodClient.searchModById(Integer.parseInt(target));
            } catch (final NumberFormatException e) {
                mod = TmodClient.searchModBySlug(target);
            }
        } catch (final CurseForgeModSearchException e) {
            System.err.println(e.getMessage());
            return;
        }

        Map<Mod, Entry<File, List<Mod>>> modsToAddWithInfo;
        try {
            modsToAddWithInfo = getAllModsToAdd(mod, repository);
        } catch (final CurseForgeApiGetException e) {
            System.err.println(e.getMessage());
            return;
        }

        Optional<Mod> conflict;
        try {
            conflict = anyModIsIncompatibleWithRepo(
                modsToAddWithInfo,
                repository
            );
        } catch (final CurseForgeApiGetException e) {
            System.err.println(e.getMessage());
            return;
        }

        if (conflict.isPresent()) {
            System.err.println(
                "The mod " + conflict.get() + " conflicts with other mod"
            );
            return;
        }

        repository.getManuallyAdded().add(mod.slug());
        for (final var entry : modsToAddWithInfo.entrySet()) {
            final Mod modToAdd = entry.getKey();
            final File modToAddFile = entry.getValue().getKey();
            final List<Mod> modToAddDependencies = entry.getValue().getValue();

            final DependencyInfo dependencyInfo = new DependencyInfo(
                modToAddFile.fileDate(),
                clientOnly,
                modToAddDependencies
                    .stream()
                    .map(Mod::slug)
                    .collect(Collectors.toList())
            );
            repository.getLocks().put(modToAdd.slug(), dependencyInfo);
        }

        try {
            mapper.write(repository);
        } catch (final IOException e) {
            System.err.println(e.getMessage());
        }

        final Ansi msg = new Ansi();

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
    }
}
