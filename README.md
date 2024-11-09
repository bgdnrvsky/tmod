# Tmod

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![build.yml](https://github.com/bgdnrvsky/tmod/actions/workflows/rust.yml/badge.svg)](https://github.com/bgdnrvsky/tmod/actions/workflows/rust.yml)
![](https://tokei.rs/b1/github/bgdnrvsky/tmod)

Tmod is a tool that allows you to quickly create and manage your Minecraft modpack.

Tmod keeps information about dependencies and incompatibilities so you can add the single mod and all the necessary libraries for it will be already there !

# Usage example

## Initializing
Let's start by `cd`ing into the folder where you wish to install mods.

```sh
$ cd ~/.minecraft/my-pack/ # or any folder you wish
```

Now you have to init `tmod`!

```sh
$ tmod init
```

## Choose the mod loader
You will be prompted to choose the mod loader

```
Choose the mod loader:
  Forge
  Fabric
  Quilt
  NeoForge
```

Choose the loader you wish for your modpack using arrow keys, and confirm your choice with <kbd>Enter</kbd>. Let's say that I have chosen `Forge`

## Enter the version
You will be asked about the version of Minecraft that you are going to be using with your modpack.

`Game version:`

Now, enter the version. Let's say that I have entered the version `1.20.1`

## Adding mods

The time has finally come to add your favourite mods!

### Remote mods

Tmod fetches almost all the data from [CurseForge](https://www.curseforge.com/), a mod that you add from it, I call a _remote_ mod.

You have two possible ways to add a remote mod in Tmod: by its slug and its id.

#### Using slug

Slug is the name used by CurseForge to identify the mod (textual id). Note, that the name of the mod might not be its slug!

Slug is always positioned at the end of the CurseForge mod link, so the pattern is following _https://www.curseforge.com/minecraft/mc-mods/slug_.

For example, the slug of [Just Enough Items (JEI)](https://www.curseforge.com/minecraft/mc-mods/jei) is _jei_.

#### Using the id

Id is another way that CurseForge uses to identify the mod. You can find it on the CurseForge mod page, _Project ID: XXX_.

For example, the id of [Just Enough Items (JEI)](https://www.curseforge.com/minecraft/mc-mods/jei) is _238222_.

***

Let's start by adding some of the basic, and most popular mods: [Just Enough Items (JEI)](https://www.curseforge.com/minecraft/mc-mods/jei) and [JourneyMap](https://www.curseforge.com/minecraft/mc-mods/journeymap).

```sh
$ tmod add slug jei
$ tmod add slug journeymap
```

If everything went well, you should see some searching info and short information about each of the mods you have added.

Let's now see the representation of our pool!

```sh
$ tmod tree

Tmod
├─ journeymap
└─ jei
```

***

Let's add more mods !

```sh
$ tmod add slug waystones
$ tmod add slug ram-compat
```

Now, let's check our tree again.

```
Tmod
├─ waystones
│  └─ balm
├─ jei
└─ ram-compat
   ├─ curios
   ├─ alexs-mobs
   └─ octo-lib
      └─ architectury-api
```

Well, well... As we can see, Tmod is aware about dependencies.

## Installing mods
Let's finaly install the mods we have added.

```sh
$ tmod install
```

This command will download all the mods and put them into the `mods/` directory. Use `-o` flag to overwrite this behaviour.

```sh
$ tree mods/

mods/
├── alexsmobs-1.22.9.jar
├── architectury-9.2.14-forge.jar
├── balm-forge-1.20.1-7.3.9-all.jar
├── curios-forge-5.11.0+1.20.1.jar
├── jei-1.20.1-forge-15.20.0.105.jar
├── journeymap-1.20.1-5.10.3-forge.jar
├── OctoLib-FORGE-0.4.2+1.20.1.jar
├── ramcompat-1.20.1-0.1.4.jar
└── waystones-forge-1.20-14.1.6.jar

1 directory, 9 files
```

All the mods and their dependencies were installed !
