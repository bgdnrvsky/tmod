package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.ModLoader;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import java.io.IOException;
import java.lang.Runtime.Version;
import java.nio.file.Path;
import java.util.Scanner;
import picocli.CommandLine;

@CommandLine.Command(name = "init", description = "Initialize a new empty repo")
public class Init implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @Override
    public void run() {
        Repository repo;
        try (Scanner scanner = new Scanner(System.in)) {
            repo = new Repository(
                promptVersion(scanner).toString(),
                promptLoader(scanner)
            );
        } catch (IllegalArgumentException e) {
            System.err.println(e.getMessage());
            return;
        }

        Path tmodPath = parent.getRepoPath();
        Mapper mapper = new Mapper(tmodPath);

        try {
            mapper.write(repo);
            if (!parent.isQuiet()) {
                System.out.println(
                    "Initialized an empty tmod repository at " + tmodPath
                );
            }
        } catch (IOException e) {
            System.err.println(e.getMessage());
        }
    }

    /**
     * Prompts the user to choose a {@link ModLoader}
     * @return The selected {@link ModLoader}
     */
    private ModLoader promptLoader(Scanner sc) {
        for (int i = 0; i < ModLoader.values().length; ++i) {
            System.out.printf("%d. %s\n", i + 1, ModLoader.values()[i]);
        }

        int id = 0;

        do {
            System.out.print("Choose the mod loader: ");

            if (sc.hasNextInt()) {
                id = sc.nextInt();
            } else {
                sc.nextLine();
            }
        } while (id <= 0 || id > ModLoader.values().length);

        return ModLoader.values()[id - 1];
    }

    /**
     * Prompts the user to choose a game version
     *
     * @return {@link Version} representing the game's version
     * @throws IllegalArgumentException If the given string cannot be interpreted as a valid version
     * @throws NumberFormatException If an element of the version number or the build number cannot be represented as an Integer
     */
    // TODO: Choose among a predefined list of versions ?
    private Version promptVersion(Scanner sc)
        throws IllegalArgumentException, NumberFormatException {
        System.out.print("Choose the game version: ");
        Version versionChoice = Version.parse(sc.nextLine());
        return versionChoice;
    }
}
