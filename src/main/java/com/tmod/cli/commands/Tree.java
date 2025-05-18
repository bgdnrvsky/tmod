package com.tmod.cli.commands;

import com.tmod.cli.App;
import com.tmod.core.repo.Mapper;
import com.tmod.core.repo.Repository;
import com.tmod.core.repo.models.DependencyInfo;
import hu.webarticum.treeprinter.SimpleTreeNode;
import hu.webarticum.treeprinter.printer.listing.ListingTreePrinter;
import java.io.IOException;
import java.util.Map;
import picocli.CommandLine;

@CommandLine.Command(
    name = "tree",
    description = "Print the tree of added mods and dependencies"
)
public class Tree implements Runnable {

    @CommandLine.ParentCommand
    private App parent;

    private SimpleTreeNode generateNode(
        String slug,
        Map<String, DependencyInfo> locks
    ) {
        SimpleTreeNode node = new SimpleTreeNode(slug);

        for (String dependency : locks.get(slug).dependencies()) {
            node.addChild(generateNode(dependency, locks));
        }

        return node;
    }

    @Override
    public void run() {
        Mapper mapper = new Mapper(parent.getRepoPath());
        Repository repository;

        try {
            repository = mapper.read();
        } catch (IOException e) {
            System.err.println(e.getMessage());
            return;
        }

        SimpleTreeNode root = new SimpleTreeNode("tmod");

        for (String slug : repository.getManuallyAdded()) {
            root.addChild(generateNode(slug, repository.getLocks()));
        }

        new ListingTreePrinter().print(root);
    }
}
