package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import picocli.CommandLine;

import com.tmod.core.models.ModLoader;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Scanner;

@CommandLine.Command(
        name = "init",
        description = "Initialize a new empty repo"
)
public class Init implements Runnable {
    @CommandLine.ParentCommand
    private App parent;

    @Override
    public void run() {
        Repository repo = new Repository(
                promptVersion(),
                promptLoader()
        );

        try {
            Path tmodPath = Path.of(parent.getRepoPath());
            Mapper.write(repo, tmodPath);
            if (!parent.isQuiet()) {
                System.out.println("Initialized an empty tmod repository at " + tmodPath);
            }
        } catch (IOException e) {
            System.err.println(e.getMessage());
        }
    }

    /**
     * Prompts the user to choose a {@link ModLoader}
     * @return The selected {@link ModLoader}
     */
    private ModLoader promptLoader() {
        for (int i = 0; i < ModLoader.values().length; ++i) {
            System.out.printf("%d. %s\n", i + 1, ModLoader.values()[i]);
        }

        Scanner sc = new Scanner(System.in);

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
     * @return {@link String} representing the game's version
     */
    // TODO: Choose among a predefined list of versions ?
    private String promptVersion() {
        System.out.print("Choose the game version: ");

        Scanner sc = new Scanner(System.in);

        return sc.nextLine();
    }
}
