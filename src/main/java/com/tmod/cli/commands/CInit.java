package com.tmod.cli.commands;

import com.beust.jcommander.Parameters;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.models.ModLoader;

import java.io.IOException;
import java.util.Scanner;

@Parameters(commandNames="init", commandDescription="Initialize a new repo")
public class CInit implements ICommand {

    @Override
    public void onExecute(Options options) {
        Repository repo = new Repository(
                promptVersion(),
                promptLoader()
        );

        try {
            Mapper.write(repo, options.getRepositoryPath());
            System.out.println("Initialized empty tmod repository at " + options.getRepositoryPath());
        } catch (IOException e) {
            System.err.println(e.getMessage());
        }
    }

    /**
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
     * TODO: Choose among a predefined list of versions ?
     *
     * @return {@link String} representing the game's version
     */
    private String promptVersion() {
        System.out.print("Choose the game version: ");

        Scanner sc = new Scanner(System.in);

        return sc.nextLine();
    }
}
