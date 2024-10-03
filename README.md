# Tmod

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)
[![build.yml](https://github.com/bgdnrvsky/tmod/actions/workflows/rust.yml/badge.svg)](https://github.com/bgdnrvsky/tmod/actions/workflows/rust.yml)
![](https://tokei.rs/b1/github/bgdnrvsky/tmod)

Tmod is a tool that allows you to quickly create and manage your Minecraft modpack.

Add a remote mod to `.tmod/remotes.json` using its slug, or add already existing JAR files to `.tmod/locals/` directory.

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

`TODO: Write more`
