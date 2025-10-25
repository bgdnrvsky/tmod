package com.tmod.cli.commands;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
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

    @CommandLine.Option(
        names = { "-w", "--web" },
        description = "Print link to CurseForge page",
        defaultValue = "false"
    )
    private boolean showLinkToWeb = false;

    @CommandLine.Option(
        names = { "--json" },
        description = "Print full info about the mod in JSON format",
        defaultValue = "false"
    )
    private boolean json = false;

    @CommandLine.Parameters(
        paramLabel = "<mod id/slug>",
        description = "Search using mod id, or mod's 'slug' (slug is not always the same as the mod name)"
    )
    private String target;

    @Override
    public void run() {
        Mod mod;

        try {
            mod = TmodClient.searchModById(Integer.parseInt(target));
        } catch (NumberFormatException e) {
            mod = TmodClient.searchModBySlug(target);
        }

        Ansi msg = new Ansi();

        msg.fgBlue().a(mod.name()).fgDefault().format("(id: %d)", mod.id());

        if (showLinkToWeb) {
            msg.format("[web: %s]", mod.links().websiteUrl());
        }

        msg
            .a(" - ")
            .a(Attribute.ITALIC)
            .a(mod.summary())
            .a(Attribute.ITALIC_OFF);

        if (json) {
            ObjectMapper mapper = new ObjectMapper();
            mapper.configure(SerializationFeature.INDENT_OUTPUT, true);

            try {
                String jsonString = mapper.writeValueAsString(mod);
                msg.a(jsonString);
            } catch (JsonProcessingException e) {
                msg.fgRed().a("ERROR").fgDefault().a(": ").a(e.getMessage());
            }
        }

        try (AnsiPrintStream stream = AnsiConsole.out()) {
            stream.println(msg);
        }
    }
}
