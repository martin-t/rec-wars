Rec Wars
========

*Recreational Warfare .rs*

A top-down vehicle shooter based on an old Windows game called RecWar.

TODO logo? use in-game entities to spell out Rec Wars?
TODO screenshot - clicking opens game?
TODO Key Features

Rec Wars aims to have similar gameplay as the original RecWar but not identical since it would be impossible to replicate exactly without decompiling the binary.

Additionally I suspect RecWar was balanced for playing against bots and might result in annoying strats when people start [playing to win](http://www.sirlin.net/articles/playing-to-win). E.g. with enough mines, the cow can be made completely inaccessible, especially to less maneuverable vehicles like the hovercraft. Experience from other poorly designed games also shows large areas will be dominated by instant-hit weapons (here the Railgun) and there might simply be no way to get across the map without getting hit.

TODO badges (flat like https://github.com/dtolnay/enumn ?)
    discord? like https://github.com/not-fl3/macroquad
TODO github description, tags (same as topics?)

TODO goals - web client+server if possible, native client+server
TODO describe impl basics - ecs, net, AI

TODO Installation / Compiling
TODO Contributing

note to self:
    - fix npm
        - https://stackoverflow.com/questions/16151018/npm-throws-error-without-sudo/24404451#24404451
        - prefix path or it uses old debian npm which breaks everything
    - build with `wasm-pack build --target web --dev` or it gives a mime type error

The Original Game
-----------------

Windows .exe only but runs ok-ish wine (sometimes freezes on map load). Includes a map editor.

homepage: http://recreationalwarfare.atspace.com/index_willem.html (the game's download is broken but still hosts extra maps)
unofficial homepage: http://www.recwar.50webs.com/
archive.org download: https://archive.org/details/recwar_201903
archive.org download with extra maps: https://archive.org/details/RecWar

The binaries in both archive.org links are identical to what i got on an old CD so should be safe.

TODO look at:
https://hupage.mypage.cz/menu/domovska-stranka/download/recwar
https://recwar.osoba.cz/rubriky/download

Licence
-------

All code is available under GPL-v3 or newer. TODO include, make sure GH shows it

All assets (maps, textures, sounds, etc.) are taken from the original RecWar by Willem Janssen.
