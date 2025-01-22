package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import org.fusesource.jansi.Ansi;
import org.fusesource.jansi.AnsiConsole;
import org.fusesource.jansi.AnsiPrintStream;
import picocli.CommandLine;
import picocli.CommandLine.Option;

@CommandLine.Command(name = "list", description = "List the mods in the repo")
public class List implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @Option(
        names = { "-n", "--names" },
        description = "Use mod display name",
        defaultValue = "false"
    )
    private boolean useOnlineNames = false;

    @Override
    public void run() {
        try {
            Mapper mapper = new Mapper(parent.getRepoPath());
            Repository repository = mapper.read();

            int i = 1;

            for (String slug : repository.getManuallyAdded()) {
                String modName;

                if (this.useOnlineNames) {
                    Mod mod = TmodClient.searchModBySlug(slug);
                    modName = mod.name();
                } else {
                    modName = slug;
                }

                Ansi msg = new Ansi();

                msg.format("%d. ", i).fgBlue().a(modName).fgDefault();

                try (AnsiPrintStream stream = AnsiConsole.out()) {
                    stream.println(msg);
                }

                i += 1;
            }
        } catch (Exception e) {
            System.err.println(e.getMessage());
        }
    }
}
