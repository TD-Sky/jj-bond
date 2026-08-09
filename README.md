<h1 align="center">JJ-Bond</h1>

<p align="center">
  <a href="https://crates.io/crates/jj-bond"><img src="https://img.shields.io/crates/v/jj-bond.svg" alt="crates.io"></a>
  <a href="https://deepwiki.com/TD-Sky/jj-bond"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
</p>



## Features

- [x] View history, show, files, diff
- [x] New, describe, edit, split, split parallelly change
- [x] Abandon, squash, duplicate, rebase changes
- [x] Fetch, push
- [x] View bookmarks
- [x] Create, set, delete, track, untrack bookmark
- [x] View tags
- [x] Create, set, delete tag
- [x] View history of a bookmark
- [x] View history of a tag
- [x] View operations
- [x] Auto update stale workspace
- [ ] Diff of a range of changes
- [ ] Rebase bookmark
- [ ] Search history via [fff](https://github.com/dmtrKovalenko/fff)
- [ ] Mark all the unsynced bookmarks



## Installation

### Cargo

```console
$ cargo install jj-bond
```

### AUR

```console
$ paru -S jj-bond
```

or binary package:

```console
$ paru -S jj-bond-bin
```



## Usage

```console
$ jb
```



## Show

1. Basic usage:

   [![hightlight_basic](https://asciinema.org/a/pNXHAUohBu7iGUDX.svg)](https://asciinema.org/a/pNXHAUohBu7iGUDX)

1. Use external editor:

   [![hightlight_desc](https://asciinema.org/a/93Ez8ThpnRR5H1Jl.svg)](https://asciinema.org/a/93Ez8ThpnRR5H1Jl)

1. Lazy view detail:

   [![hightlight_debounce](https://asciinema.org/a/iuGv3iiV7VzgX8S8.svg)](https://asciinema.org/a/iuGv3iiV7VzgX8S8)

1. Rebase a range of changes:

   [![hightlight_rebase](https://asciinema.org/a/OOtvEk69BbWlOESZ.svg)](https://asciinema.org/a/OOtvEk69BbWlOESZ)

1. Squash a range of changes:

   [![hightlight_squash](https://asciinema.org/a/OgNHwynfMuxGrljP.svg)](https://asciinema.org/a/OgNHwynfMuxGrljP)

1. Add bookmark, track, push:

   [![hightlight_push](https://asciinema.org/a/T0Rydg0pkVpqGOd5.svg)](https://asciinema.org/a/T0Rydg0pkVpqGOd5)

1. View bookmark history:

   [![hightlight_bookmark_history](https://asciinema.org/a/2GYlnAPd15MtaVDP.svg)](https://asciinema.org/a/2GYlnAPd15MtaVDP)
