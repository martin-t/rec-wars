<div align="center">
    <h1>RecWars</h1>
    <i>Recreational Warfare .rs</i>
    <br />
    A multiplayer top-down tank shooter - Rust/WASM port of an old Windows game called RecWar.
</div>
<br />

[![CI](https://github.com/martin-t/rec-wars/workflows/CI/badge.svg)](https://github.com/martin-t/rec-wars/actions)
[![Discord](https://img.shields.io/discord/770013530593689620?label=discord)](https://discord.gg/9BQVVgV)
[![Total lines](https://tokei.rs/b1/github/martin-t/rec-wars)](https://github.com/martin-t/rec-wars)
[![Lines of comments](https://tokei.rs/b1/github/martin-t/rec-wars?category=comments)](https://github.com/martin-t/rec-wars#architecture-overview)

<!-- Note to my future OCD: The ideal image width for github is 838 pixels -->
[![Gameplay](media/screenshot.jpg)](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web)

_**[Play in the Browser](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web) | [Join Discord](https://discord.gg/9BQVVgV)**_

RecWars is a free and open source clone of [RecWar](#the-original-game) - you control a vehicle and fight other vehicles in a variety of game modes using an arsenal of several distinct weapons. You can play against bots, in splitscreen and over the network.

RecWars aims to have gameplay similar but not identical to RecWar. I suspect RecWar was balanced for playing against bots and might result in annoying strats being the most effective when people start [playing to win](http://www.sirlin.net/articles/playing-to-win). However, almost everything in RecWars is [configurable](#cvars) and you can switch to the original RecWar balance to [compare](#recwars-vs-recwar-differences).

**Currently this is very much a work-in-progress**: only some weapons work, the driving physics don't feel right, there are no collisions between vehicles, ...

The ultimate goal is to create a moddable multiplayer game playable in the browser and natively on Linux, Windows and macOS. This might be tricky since WASM in the browser doesn't allow UDP. I have some ideas how to solve that.

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
- [x] [Highly configurable](#cvars)

Cvars
-----

Cvars are *console variables* - configuration settings which control everything in the game like physics, weapon behavior, AI, HUD layout, etc.

There are two ways to change them:
- Edit the `cvars` object using the browser console - e.g. `cvars.g_armor = 100`.
- Set them using URL parameters - e.g. [https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?g_armor=100](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?g_armor=100)

The entire list of cvars is in [src/cvars.rs](src/cvars.rs).

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

You can always find me on the [RecWars Discord server](https://discord.gg/9BQVVgV) if you have any questions or suggestions.

[Issues](https://github.com/martin-t/rec-wars/issues/new) and [Pull Requests](https://github.com/martin-t/rec-wars/pulls) are welcome. While I am not *actively* looking for contributors, I am open to anyone wanting to help improve RecWars.

I want to make RecWars highly configurable with many different gamemodes and balance settings votable by players and anybody will be able to host their own server (if technically possible even from the browser). If you have a gameplay idea and don't suffer from the NIH syndrome, I'd be very happy to help you test it in RecWars.

**Optionally** enable extra checks before every commit with `git config core.hooksPath git-hooks`.

### Architecture Overview

Most of the code is commented to be understandable to anyone with a vague idea of how a game works. If it's not clear why a particular piece of code exists or why it needs to be written the way it is, I consider that a bug which should be fixed by either rewriting the code more clearly or adding comments explaining it.

RecWars is written in Rust with a small bit of JS glue. It does *not* depend on NPM. Currently, it draws directly to an HTML5 canvas using the 2D API which turns out to be too slow to redraw the entire screen at 60Hz. I am still deciding between [macroquad](https://github.com/not-fl3/macroquad), [luminance](https://github.com/phaazon/luminance-rs) and [wgpu-rs](https://github.com/gfx-rs/wgpu-rs).

Currently, most game state is managed by generational arenas from the [thunderdome](https://github.com/LPGhatguy/thunderdome) crate to make the code type-safe and readable. Previously, RecWars used the [legion](https://github.com/amethyst/legion) ECS. However it was cumbersome to use and WASM didn't get any benefits from parallelism. The only reason I was using ECS was so I could have references between entities and for this I was paying by having all entities dynamicly typed which lead to bugs. It's a Rust tradition to start writing a game and end up writing a game engine or ECS so I am considering creating an ECS crate that would satisfy my standards of clean API and static typing. For now arenas seem to be close enough.

The Original Game
-----------------

RecWar by Willem Janssen:
- homepage: http://recreationalwarfare.atspace.com/index_willem.html (the game's download is broken but still hosts extra maps)
- unofficial homepage: http://www.recwar.50webs.com/
- archive.org download: https://archive.org/details/recwar_201903
- archive.org download with extra maps: https://archive.org/details/RecWar

The original RecWar only contains a Windows .exe but runs ok-ish wine (sometimes freezes on map load). It includes a map editor. The binaries in both archive.org links are identical to what I got on an old CD so should be safe.

### RecWars vs RecWar differences

RecWar would probably be impossible to replicate exactly without decompiling the binary (which doesn't even contain debug symbols), though if a fan of the original finds this project, I am not gonna stop them from trying.

Additionally, when playing against people instead of bots, I suspect RecWar's original balance would lead to annoying and boring strats like making the cow inaccessible with mines or just simple camping. For example, experience from poorly designed games shows large areas will be dominated by instant-hit weapons (in RecWar the railgun) and there might simply be no way to get across the map alive. Therefore I made the railgun a very fast projectile in RecWars. I might make more balance changes based on how the online gameplay evolves.

For those reasons, RecWars will have a slightly different balance than RecWar. I will try to keep them as similar as possible but some things like physics will never be exact and I will make changes where I see fit to make the gameplay more interesting.

The two balance presets are available here:
- https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?balance=recwars (default)
- https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?balance=recwar

Intentional differences (can be toggled by switching the balance):
- Railgun - RecWar railgun hits instantly, RecWars uses a very fast projectile because hitscan weapons ruin large maps

Other unintentional differences - I will make best effort here but some things won't be exact:
- Speeds, accelerations, turning, inertia of vehicles and weapons
- Push force of mines and railguns
- Tank in RecWar turned around turret swivel point, not center of chassis - this is for simplicity for now
- Weapons
    - Damage - Cluster bomb and BFG beam are hard to measure exactly
    - Spreads - Cluster bombs and MG are hard to measure exactly
- Self destruct damage and range - it appears to be the only explosion in RecWar with damage decreasing by distance and it's really hard to measure exactly.

Maps
----

- `maps/` - Maps from the original RecWar
- `maps/extra/` - Extra maps from the official homepage
- `maps/extra2/` - Extra maps from archive.org

Currently the map is picked randomly by default, however, you can select one manually by using the `map` URL parameter, for example [https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?map=Castle Islands (4)](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web/?map=Castle%20Islands%20(4)).

Lessons Learned
---------------

Read this to learn from other people's mistakes and save yourself some time.

- It's possible and advisable to use WASM without NPM. The official [Rust+WASM book](https://rustwasm.github.io/docs/book/) heavily pushes people towards NPM and the whole thing feels like "just download this big template, don't try to understand it and only touch the parts we tell you to". Honestly how do you even statically host the thing on GH pages without `npm run`?. If you're not planning to use other NPM packages, all you need is a few lines of JS glue to run your WASM. Use the [Without a Bundler](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html) example as your "template" and host it with `python3 -m http.server`. You'll understand exactly what is going on and you'll avoid the whole JS ecosystem.
- The canvas 2D API is too slow for a game which needs to redraw the entire screen each frame, especially in firefox.
- ECS is overhyped. It will make all your game entities dynamicly typed but with much more boilerplate than a dynlang and will predictably lead to bugs. If you don't need to add/remove components at runtime, the only reason you're using it is probably so you can have references between entities - just use generational arenas. Appeal to authority: [rg3d](https://rg3d.rs/) is written by an experienced game engine dev and avoids ECS for much the same reason.
- Generationl arenas will lead to slightly more borrowchk errors. ECS either avoided them implicitly by borrowing only parts of the gamestate (runtime borrowchecking) or resolved them explicitly by postponing mutation (e.g. legion's [`CommandBuffer`](https://docs.rs/legion/*/legion/systems/struct.CommandBuffer.html)). With arenas, you have to deal with them explicitly more often:
    - You can't add to / remove from an arena while iterating through it
        - Use specialized methods like `retain` if available
        - Avoid borrowing the whole arena in a loop - instead, collect handles into a vector, iterate through that vector and borrow only for parts of the loop body
        - Collect entities to add / handles to remove into a vector, add / remove after the loop is done
    - Subfunctions need mutable game state while you're iterating through one of the arenas
        - Be more granular - don't pass them the entire game state, just the arenas it needs
        - Again, collect handles, iterate through them, reborrow each iteration, end borrow before calling subfunctions

License
-------

<!-- When updating this, also update LICENSE and Cargo.toml -->
All code is available under [AGPL-v3](agpl-3.0.txt) or newer.

All assets (maps, textures, sounds, etc.) are taken from the original RecWar by Willem Janssen which is freely available online.
