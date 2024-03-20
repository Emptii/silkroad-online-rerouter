# Silkroad Online Rerouter
This is a small tool that patches Silkroad Online's `Media.pk2` file to connect to another host than the original.
More specifically, it replaces the `DIVISIONINFO.TXT` within the Media.pk2 with one containing `127.0.0.1`.

## Take care
Make sure to back up your game files before using this tool. I don't know if this works for every version. This tool will create a backup file before patching called `Media.pk2.bak`, so if your game doesnt launch after patching you might want to use your old file instead.

## How to use it
### Ubuntu
```bash
silkroad-online-rerouter reroute --key <blowfish_key> --game_directory <game_directory>
```

## Why
I wanted to try out [Kumpelblase2](https://github.com/kumpelblase2)'s [silkroad online backend emulator](https://github.com/kumpelblase2/skrillax).
To do that I had to edit the `DIVISIONINFO.TXT`, which Silkroad uses to know which host it connects to. This file is stored within the `Media.pk2`.

Silkroad online uses the [pk2 file format](https://en.wikipedia.org/wiki/PK2_(file_extension)), so I was not able to just edit files as I wanted to. Thankfully, there is a [rust package](https://crates.io/crates/pk2) developed by [Lukas Wirth](https://crates.io/users/Veykril) that enables reading and writing of pk2 files. I also took heavy inspiration from his pk2_mate tool.

## How it works
It first extracts the `Media.pk2` file. Then it replaces the `DIVISIONINFO.TXT` file with one pointing to `127.0.0.1`. Finally it packs everything back into a `pk2` file and overwrites the original `Media.pk2` file with the new one. 
