<div align="center">
    <h1>RecWars</h1>
    <i>Recreational Warfare .rs</i>
    <br />
    A top-down vehicle shooter - Rust/WASM port of an old Windows game called RecWar.
</div>
<br />

_**[Play Online](https://martin-t.gitlab.io/gitlab-pages/rec-wars/web)**_

TODOs
- badges (flat like https://github.com/dtolnay/enumn ?)
    - discord? like https://github.com/not-fl3/macroquad
   
- logo? use in-game entities to spell out Rec Wars?
- screenshot - clicking opens game?
- Key Features

- goals - web client+server if possible, native client+server
- describe impl basics - ecs, net, AI

- Rec Wars vs RecWars

note to self:
- fix npm
    - https://stackoverflow.com/questions/16151018/npm-throws-error-without-sudo/24404451#24404451
    - prefix path or it uses old debian npm which breaks everything

Compiling
---------

- build with `wasm-pack build --target web --dev`
    - you can replace `--dev` with `--profiling` or `--release` if perf is an issue
- host with `python3 -m http.server` (or any other web server, simply opening `index.html` will *not* work though)
- open http://localhost:8000/web/

Contributing
------------

If you notice a bug or have a suggestion, don't hesitate to [open an Issue](https://github.com/martin-t/rec-wars/issues/new).

If you'd like to improve RecWars, feel free to make a [Pull Request](https://github.com/martin-t/rec-wars/pulls), I'll do my best to review it soon. I want to make RecWars highly configurable with many different gamemodes and balance settings votable by players. If you have a gameplay idea and don't suffer from the NIH syndrome, I'd be very happy to help you test it in RecWars.

The Original Game
-----------------

- homepage: http://recreationalwarfare.atspace.com/index_willem.html (the game's download is broken but still hosts extra maps)
- unofficial homepage: http://www.recwar.50webs.com/
- archive.org download: https://archive.org/details/recwar_201903
- archive.org download with extra maps: https://archive.org/details/RecWar

The original RecWar only contains a Windows .exe but runs ok-ish wine (sometimes freezes on map load). It includes a map editor. The binaries in both archive.org links are identical to what I got on an old CD so should be safe.

TODO look at:
https://hupage.mypage.cz/menu/domovska-stranka/download/recwar
https://recwar.osoba.cz/rubriky/download

Rec Wars aims to have similar gameplay as the original RecWar but not identical since it would be impossible to replicate exactly without decompiling the binary (which doesn't even contain debug symbols as far as I can tell).

Additionally I suspect RecWar was balanced for playing against bots and might result in annoying strats when people start [playing to win](http://www.sirlin.net/articles/playing-to-win). E.g. with enough mines, the cow can be made completely inaccessible, especially to less maneuverable vehicles like the hovercraft. Experience from other poorly designed games also shows large areas will be dominated by instant-hit weapons (here the Railgun) and there might simply be no way to get across the map without getting hit.

Licence
-------

<!-- When updating this, also update LICENSE -->

All code is available under [GPL-v3](GPL-3.0.txt) or newer.

All assets (maps, textures, sounds, etc.) are taken from the original RecWar by Willem Janssen which is freely available online.
