CLI_DIR = ./src/main/java/com/tmod/cli
CLI_APP = $(CLI_DIR)/App.java
CLI_COMMANDS = $(CLI_DIR)/commands/*.java

ifeq ($(OS), Windows_NT)
	MVN_CMD = mvnw.cmd
else
	MVN_CMD = ./mvnw
endif

JAR_TARGET = ./target/tmod-1.0-SNAPSHOT-shaded.jar

tmod: $(JAR_TARGET)
	touch tmod
	echo "#!/bin/sh" >> tmod
	echo 'exec java -jar $$0 "$$@"' >> tmod
	cat $(JAR_TARGET) >> tmod
	chmod +x tmod

$(JAR_TARGET): $(CLI_APP) $(CLI_COMMANDS)
	$(MVN_CMD) -q -Dmaven.test.skip=true package
