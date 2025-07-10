/**
 * @author: Era
 */

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
    public static String run(String... args) {
        StringWriter swOut = new StringWriter();
        StringWriter swErr = new StringWriter();

        // New execution
        CommandLine cmd = new CommandLine(new App());

        cmd.setOut(new PrintWriter(swOut));
        cmd.setErr(new PrintWriter(swErr));

        cmd.execute(args);

        return swOut.toString() + swErr.toString();
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
