<div align="center">
    <h1>RecWars</h1>
    <i>Recreational Warfare .rs</i>
    <br />
    A multiplayer top-down tank shooter - Rust/WASM port of an old Windows game called RecWar.
</div>
<br />

<!-- Note to my future OCD: The ideal image width for github is 838 pixels -->
[![Gameplay](media/screenshot.jpg)](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web)

_**[Play Online](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web)**_

RecWars is a free and open source clone of [RecWar](#the-original-game) - you control a vehicle and fight other vehicles in a variety of game modes using an arsenal of several distinct weapons. You can play against bots, in splitscreen and over the network.

RecWars aims to have gameplay similar but not identical to RecWar - it would be impossible to replicate exactly without decompiling the binary (which doesn't even contain debug symbols), though if a fan of the original finds this project, I am not gonna stop them from trying.

Additionally I suspect RecWar was balanced for playing against bots and might result in annoying strats when people start [playing to win](http://www.sirlin.net/articles/playing-to-win). E.g. with enough mines, the cow can be made completely inaccessible, especially to less maneuverable vehicles like the hovercraft. Experience from poorly designed games also shows large areas will be dominated by instant-hit weapons (in RecWar the railgun) and there might simply be no way to get across the map without getting hit. I might make balance changes based on how the online gameplay evolves.

Currently this is very much a work-in-progress: only some weapons work, the driving physics don't feel right, there are no collisions between vehicles, ..., you can't respawn, etc.

The ultimate goal is to create a multiplayer game playable in the browser and on Linux, Windows and macOS. This might be tricky since WASM in the browser doesn't allow UDP. I have some ideas how to solve that.

TODOs
- badges (flat like https://github.com/dtolnay/enumn ?)
    - discord? like https://github.com/not-fl3/macroquad

(Planned) Features
------------------

- [ ] Bots
- [ ] Multiplayer
    - [ ] Splitscreen
    - [ ] Network
    - [ ] Combination of both (plus bots)
- [x] [Browser client](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web)
- [ ] Native client
- [ ] Game modes
    - [ ] Free For All
    - [ ] Team War
    - [ ] Capture The Cow

Dependencies
------------

- [wasm-pack](https://github.com/rustwasm/wasm-pack) - use the [installer](https://rustwasm.github.io/wasm-pack/installer/)

Compiling
---------

- build with `wasm-pack build --target web --dev`
    - you can replace `--dev` with `--profiling` or `--release` if perf is an issue (see [Cargo.toml](Cargo.toml) for more info)
- host with `python3 -m http.server` (or any other web server, simply opening `index.html` will *not* work though)
- open http://localhost:8000/web/

Contributing
------------

If you notice a bug or have a suggestion, please [open an Issue](https://github.com/martin-t/rec-wars/issues/new).

If you'd like to improve RecWars, feel free to make a [Pull Request](https://github.com/martin-t/rec-wars/pulls). I want to make RecWars highly configurable with many different gamemodes and balance settings votable by players and anybody will be able to host their own server (if technically possible even from the browser). If you have a gameplay idea and don't suffer from the NIH syndrome, I'd be very happy to help you test it in RecWars.

Most of the code is commented to be understandable to anyone with a vague idea of how a game works. If it's not clear why a particular piece of code exists or needs to be written the way it is, I consider that a bug which should be fixed by either rewriting the code more clearly or adding comments explaining it.

RecWars is written in Rust with a small bit of JS glue. It does *not* depend on NPM. Currently, it draws directly to an HTML5 canvas using the 2D API which turns out to be too slow to redraw the entire screen at 60Hz. I am still deciding between macroquad, luminance and wgpu-rs.

Most game state is in the legion ECS, however it's cumbersome to use and WASM doesn't get any benefits from parallelism. It might have been a better idea to use a generational arena or similar allocator - the only reason I am using ECS is so I can have references between entities and for this I am paying by having all entities dynamicly typed which leads to bugs. It's a Rust tradition to start writing a game and end up writing a game engine or ECS so I am considering creating an ECS crate that would satisfy my standards of clean API and static typing.

The Original Game
-----------------

RecWar by Willem Janssen:
- homepage: http://recreationalwarfare.atspace.com/index_willem.html (the game's download is broken but still hosts extra maps)
- unofficial homepage: http://www.recwar.50webs.com/
- archive.org download: https://archive.org/details/recwar_201903
- archive.org download with extra maps: https://archive.org/details/RecWar

The original RecWar only contains a Windows .exe but runs ok-ish wine (sometimes freezes on map load). It includes a map editor. The binaries in both archive.org links are identical to what I got on an old CD so should be safe.

Known ways RecWars differs from RecWar:
- Speeds, accelerations, turning, inertia of vehicles and weapons - I will make best effort here but it won't be exact
- Push force of mines and railguns
- Tank in RecWar turned around turret swivel point, not center of chassis
- Weapons
    - Damage - Cluster bomb and BFG beam are hard to measure exactly
    - Spreads - Cluster bombs and MG are hard to measure exactly
    - Railgun - Will be a very fast projectile because hitscan weapons ruin large maps
- Self destruct damage and range - it appears to be the only explosion in RecWar with damage decreasing by distance and it's really hard to measure exactly.

TODO look at:
https://hupage.mypage.cz/menu/domovska-stranka/download/recwar
https://recwar.osoba.cz/rubriky/download

License
-------

<!-- When updating this, also update LICENSE and Cargo.toml -->
All code is available under [AGPL-v3](agpl-3.0.txt) or newer.

All assets (maps, textures, sounds, etc.) are taken from the original RecWar by Willem Janssen which is freely available online.
