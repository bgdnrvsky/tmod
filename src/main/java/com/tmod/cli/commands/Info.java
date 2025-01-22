package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.models.Mod;
import com.tmod.core.net.TmodClient;
import org.fusesource.jansi.Ansi;
import org.fusesource.jansi.Ansi.Attribute;
import org.fusesource.jansi.AnsiConsole;
import org.fusesource.jansi.AnsiPrintStream;
import picocli.CommandLine;

@CommandLine.Command(
    name = "info",
    description = "Search a remote mod and print its info"
)
public class Info implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    @CommandLine.Parameters(
        paramLabel = "<mod id/slug>",
        description = "Search using mod id, or mod's 'slug' (slug is not always the same as the mod name)"
    )
    private String target;

    @Override
    public void run() {
        try {
            Mod mod;

            try {
                mod = TmodClient.searchModById(Integer.parseInt(target));
            } catch (NumberFormatException e) {
                mod = TmodClient.searchModBySlug(target);
            }

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
