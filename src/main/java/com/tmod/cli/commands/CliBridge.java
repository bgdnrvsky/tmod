package com.tmod.cli.commands;

import com.tmod.cli.App;
import picocli.CommandLine;

import java.io.PrintWriter;
import java.io.StringWriter;

public class CliBridge {
    private static final CommandLine CMD = new CommandLine(new App());
    private CliBridge() {}


    /** Start tmod in the way, like user wrote all the arguments in the command line
     * and returns all output as a String (STDOUT, STDERR).
     *
     * @param args arguments to pass to tmod
     */
    public static String run (String... args) {
        StringWriter swOut = new StringWriter();
        StringWriter swErr = new StringWriter();

        PrintWriter pwOut = new PrintWriter(swOut);
        PrintWriter pwErr = new PrintWriter(swErr);

        CMD.setOut(pwOut);
        CMD.setErr(pwErr);

        pwOut.flush();
        pwErr.flush();

        return swOut + swErr.toString();
    }


    /** Util for command, result - code of execution
     *
     * @param args
     * @return
     */
    public static int runQuiet(String... args) {
        return CMD.execute(args);
    }

}
