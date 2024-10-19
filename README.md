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

Choose the loader you wish for your modpack using arrow keys, and confirm your choice with `Enter`. Let's say that I have chosen `Forge`

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

### Local mods

You can also add the mod if you have its jar!

```sh
$ tmod add jar /path/to/the/mod.jar
```

***

Let's start by adding some of the basic, and most popular mods: [Just Enough Items (JEI)](https://www.curseforge.com/minecraft/mc-mods/jei) and [JourneyMap](https://www.curseforge.com/minecraft/mc-mods/journeymap).

```sh
$ tmod add slug jei journeymap
```

If everything went well, you should see some searching info and short information about each of the mods you have added.

Let's now see the representation of our pool!

```sh
$ tmod tree
```
```
Tmod
├─ Remotes
│  ├─ jei
│  └─ journeymap
└─ Locals
```

#### Tree representation

The most basic tree, is the tree of the empty pool.

```
Tmod
├─ Remotes
└─ Locals
```

The tree has two parts: remote and local, each contains the mods you have added, as well as their dependencies.

***

Let's add more mods !

```sh
$ tmod add slug waystones ram-compat
```

Now, let's check our tree again.

```
Tmod
├─ Remotes
│  ├─ waystones
│  │  └─ balm
│  ├─ ram-compat
│  │  ├─ curios
│  │  ├─ alexs-mobs
│  │  └─ octo-lib
│  │     └─ architectury-api
│  ├─ journeymap
│  └─ jei
└─ Locals
```

Well, well... As we can see, Tmod is aware about dependencies.
