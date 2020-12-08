// Load the webassembly and some assets,
// do a couple more things that are easier on the JS side like reading input.
// This could be done in rust but would be more verbose (like
// https://github.com/bzar/wasm-pong-rs/blob/master/src/lib.rs).

// Based on https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html

// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { Input, Game, Cvars } from '../pkg/rec_wars.js';

async function run() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.
    //
    // It may look like this: `await init('./pkg/without_a_bundler_bg.wasm');`,
    // but there is also a handy default inside `init` function, which uses
    // `import.meta` to locate the wasm file relatively to js file.
    //
    // Note that instead of a string you can also pass in any of the
    // following things:
    //
    // * `WebAssembly.Module`
    //
    // * `ArrayBuffer`
    //
    // * `Response`
    //
    // * `Promise` which returns any of the above, e.g. `fetch("./path/to/wasm")`
    //
    // This gives you complete control over how the module is loaded
    // and compiled.
    //
    // Also note that the promise, when resolved, yields the wasm module's
    // exports which is the same as importing the `*_bg` module in other
    // modes
    await init();

    // And afterwards we can use all the functionality defined in wasm.

    let imgs_tiles = [
        "../assets/tiles/g1.bmp",
        "../assets/tiles/g2.bmp",
        "../assets/tiles/g3.bmp",
        "../assets/tiles/g_stripes.bmp",
        "../assets/tiles/bunker1.bmp",
        "../assets/tiles/ice1.bmp",
        "../assets/tiles/ice.bmp",
        "../assets/tiles/ice_side.bmp",
        "../assets/tiles/ice_corner.bmp",
        "../assets/tiles/g_spawn.bmp",
        "../assets/tiles/road.bmp",
        "../assets/tiles/water.bmp",
        "../assets/tiles/snow.bmp",
        "../assets/tiles/snow2.bmp",
        "../assets/tiles/bunker2.bmp",
        "../assets/tiles/base.bmp",
        "../assets/tiles/water_side.bmp",
        "../assets/tiles/water_corner.bmp",
        "../assets/tiles/desert.bmp",
        "../assets/tiles/d_rock.bmp",
        "../assets/tiles/g2d.bmp",
        "../assets/tiles/water_middle.bmp",
    ].map((path) => {
        let img = new Image();
        img.src = path;
        return img;
    });

    let imgs_vehicles = [
        "../assets/vehicles/tank_chassis_flames.png",
        "../assets/vehicles/tank_turret_flames.png",
        "../assets/vehicles/hovercraft_chassis_flames.png",
        "../assets/vehicles/hovercraft_turret_flames.png",
        "../assets/vehicles/hummer_chassis_flames.png",
        "../assets/vehicles/hummer_turret_flames.png",
    ].map((path) => {
        let img = new Image();
        img.src = path;
        return img;
    });

    let imgs_wrecks = [
        "../assets/wrecks/tank.png",
        "../assets/wrecks/hovercraft.png",
        "../assets/wrecks/hummer.png",
    ].map((path) => {
        let img = new Image();
        img.src = path;
        return img;
    });

    let imgs_weapon_icons = [
        "../assets/weapon_icons/mg.png",
        "../assets/weapon_icons/rail.png",
        "../assets/weapon_icons/cb.png",
        "../assets/weapon_icons/rockets.png",
        "../assets/weapon_icons/hm.png",
        "../assets/weapon_icons/gm.png",
        "../assets/weapon_icons/bfg.png",
    ].map((path) => {
        let img = new Image();
        img.src = path;
        return img;
    });

    let img_explosion = new Image();
    img_explosion.src = "../assets/explosion.png";
    let img_explosion_cyan = new Image();
    img_explosion_cyan.src = "../assets/explosion_cyan.png";
    let img_rocket = new Image();
    img_rocket.src = "../assets/weapons/rocket.png";
    let img_hm = new Image();
    img_hm.src = "../assets/weapons/homing_missile.png";
    let img_gm = new Image();
    img_gm.src = "../assets/weapons/guided_missile.png";
    // https://stackoverflow.com/questions/46399223/async-await-in-image-loading
    // https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/createImageBitmap
    // or better yet, figure out how webpack works

    const canvas = document.getElementById("canvas");
    // It appears disabling alpha just means that the canvas itself won't be transparent to show other elements behind it.
    // Transparency can still be used within the canvas e.g. when drawing overlapping images.
    const ctx = canvas.getContext("2d", { alpha: false });

    const log_time_checkbox = document.getElementById("log-time-checkbox");
    log_time_checkbox.addEventListener("change", event => {
        // Unfocus so it doesn't react to keyboard after being clicked.
        log_time_checkbox.blur();
    });

    const input = new Input();
    let paused = false;

    // This can't be part of Cvars - there is no way to expose rust's String to JS as a struct field.
    // It's a good idea to avoid Ctrl and Alt:
    // - it's not possible to disable some browser shortcuts like Ctrl+W
    // - Alt shows/hides the menu bar on linux
    // TODO IE/edge?
    //  https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key
    //  https://www.w3.org/TR/uievents-key
    const binds = {
        left: ["arrowleft", "a"],
        right: ["arrowright", "d"],
        up: ["arrowup", "w"],
        down: ["arrowdown", "s"],
        turret_left: ["q", "n"],
        turret_right: ["e", "m"],
        prev_weapon: ["v", "."],
        next_weapon: ["shift", "b", ","],
        fire: [" "],
        mine: ["j", "x"],
        self_destruct: ["l"],
        horn: ["h"],
        chat: [],
        pause: ["pause", "p"],
    };

    document.addEventListener("keydown", event => {
        if (log_time_checkbox.checked) {
            console.log(performance.now(), "down", event.key);
        }

        // Single letters need toLowerCase to be detected when shift is held.
        for (const action in binds) {
            if (binds[action].includes(event.key.toLowerCase())) {
                input[action] = true;
            }
        }

        if (binds.pause.includes(event.key.toLowerCase())) {
            paused = !paused;
        }
    });

    document.addEventListener("keyup", event => {
        if (log_time_checkbox.checked) {
            console.log(performance.now(), "up", event.key);
        }

        for (const action in binds) {
            if (binds[action].includes(event.key.toLowerCase())) {
                input[action] = false;
            }
        }
    });

    // LATER maybe prevent accidental close when using Ctrl (use separate event handlers for detecting the key press/release)
    // https://stackoverflow.com/questions/10311341/confirmation-before-closing-of-tab-browser
    // Note: won't work if closing immediately after opening - need to at least click on the page.
    /*window.addEventListener("beforeunload", event => {
        event.preventDefault();
        event.returnValue = "Sure?";
        return "Sure?";
    });*/

    const speed_slider = document.getElementById("speed-slider");
    const speed_value = document.getElementById("speed-value");
    speed_slider.addEventListener("input", () => {
        // This fires every time the selected value changes while dragging.
        cvars.d_speed = speed_slider.value;
        speed_value.innerHTML = speed_slider.value;
    });
    speed_slider.addEventListener("change", () => {
        // This fires once dragging is complete.
        // Unfocus so that arrows don't move the slider when using them to play.
        // Needs to be the `change` event, not `input`.
        speed_slider.blur();
    });

    // listen to all events
    // Object.keys(window).forEach(key => {
    //     if (/^on/.test(key)) {
    //         window.addEventListener(key.slice(2), event => {
    //             console.log(event);
    //         });
    //     }
    // });

    let load_tex_list = () => {
        let request = new XMLHttpRequest();
        request.open("GET", "../assets/texture_list.txt");
        request.onloadend = () => {
            if (request.status !== 200) {
                console.log("Failed to load texture_list: ", request);
                alert("Failed to load texture_list");
            } else {
                load_map(request.responseText);
            }
        }
        request.send();
    }

    let load_map = (tex_list_text) => {
        // This is a subset of maps that are not blatantly broken with the current bots.
        let maps = [
            //"Arena",
            //"A simple plan (2)",
            "Atrium",
            "Bunkers (2)",
            "Castle Islands (2)",
            "Castle Islands (4)",
            //"Corners (4)",
            "Delta",
            "Desert Eagle",
            //"Joust (2)",
            //"Large front (2)",
            //"Oases (4)",
            "Park",
            "Roads",
            "Snow",
            "Spots (8)",
            //"Vast Arena",

            //"extra/6 terrains (2)",
            //"extra/A Cow Too Far",
            //"extra/All Water",
            //"extra/Battlegrounds (2)",
            //"extra/Crossing", // No spawns
            "extra/Damned Rockets (2)", // Asymmetric CTF, left half like Castly Islands (2)
            //"extra/doom",
            //"extra/elements",
            //"extra/Exile (4)", // Tiny, many spawns
            //"extra/football",
            "extra/Ice ring",
            //"extra/ice skating ring (2)",
            "extra/IceWorld",
            "extra/I see you (2)", // Like Large Front (2) but without any cover
            //"extra/Knifflig (2)",
            //"extra/Large",
            //"extra/Neutral",
            "extra/Nile",
            //"extra/OK Corral (2)",
            //"extra/Peninsulae (3)",
            //"extra/River Crossings",
            //"extra/Road To Hell (2)", // Only 4 spawns in a tiny area
            //"extra/THE Crossing",
            //"extra/Thomap1 (4)",
            //"extra/Town on Fire",
            "extra/twisted (2)",
            //"extra/winterhardcore",
            "extra/Yellow and Green",

            "extra2/Mini Islands (4)",
            //"extra2/Symmetric",
            //"extra2/Training room",
            //"extra2/Winter (4)",
            //"extra2/World War (2)",
        ];
        let random_index = Math.floor(Math.random() * maps.length);
        let random_map = maps[random_index];

        let params = new URLSearchParams(document.location.search);
        let map = params.get("map") ?? random_map;
        if (!map.endsWith(".map")) {
            map += ".map";
        }
        let map_path = `../maps/${map}`;

        let request = new XMLHttpRequest();
        request.open("GET", map_path);
        request.onloadend = () => {
            if (request.status !== 200) {
                console.log("Failed to load map: ", request);
                alert(`Failed to load map '${map_path}'`);
            } else {
                play(tex_list_text, request.responseText);
            }
        }
        request.send();
    }

    let play = (tex_list_text, map_text) => {
        // Ping master server
        let request = new XMLHttpRequest();
        request.open("GET", "https://rec-wars-master.herokuapp.com/");
        request.onloadend = () => {
            if (request.status !== 200) {
                console.log("Failed to ping master server");
            } else {
                console.log(request.responseText);
            }
        }
        request.send();

        let params = new URLSearchParams(document.location.search);

        // Cvars can be changed through the browser's console.
        // For now, they need to live on the JS heap and be passed into each function that needs them.
        // I couldn't find a better way to make them mutable in JS and readable in Rust:
        // - can't return references from Rust into JS
        // - can't have a reference in Game
        // - owned pub cvars in Game need to be copy -> can't be changed from JS (changing will have no effect)
        // - LATER try returning Rc/Arc
        let cvars;
        let balance = params.get("balance");
        if (balance === null || balance.toLowerCase() === "recwars") {
            cvars = Cvars.new_rec_wars();
        } else if (balance.toLowerCase() === "recwar") {
            cvars = Cvars.new_rec_war();
        } else {
            const msg = `Unknown balance '${balance}', falling back to default RecWars`;
            console.log(msg);
            alert(msg);
            cvars = Cvars.new_rec_wars();
        }

        // Allow changing cvars using URL params
        params.forEach(function (value, key) {
            if (key !== "map" && key !== "balance") {
                console.log(`URL param: cvars.${key} = ${value}`);

                // The param is a string, convert it to the type of the cvar.
                // Technically, this is not necessary for float cvars, they get converted implicitly.
                // For non-float cvars, however, the generated bindings in rec_wars.js explicitly check the type
                // so they need to be converted here.
                if (typeof cvars[key] === "boolean" && value === "false") {
                    // JS is retarded and Boolean("false") is true
                    // https://www.ecma-international.org/ecma-262/#sec-toboolean
                    cvars[key] = false;
                } else {
                    cvars[key] = cvars[key].constructor(value);
                }
            }
        });

        const game = new Game(cvars, ctx, canvas.width, canvas.height,
            imgs_tiles, imgs_vehicles, imgs_wrecks, imgs_weapon_icons,
            img_rocket, img_hm, img_gm, img_explosion, img_explosion_cyan,
            tex_list_text, map_text);

        // Make some things available on window for easier debugging.
        window.cvars = cvars;
        window.game = game;
        window.min_frame_delay = 0;

        let last_frame_t_real = 0;
        let last_frame_t_scaled = 0;

        const frame = (t) => {
            if (log_time_checkbox.checked) {
                console.log(performance.now(), "frame", t);
            }

            // Seconds just make more sense, plus I keep assuming they're seconds and causing bugs.
            const t_real = t / 1000.0;

            // Apparently it's best practice to call requestAnimationFrame at the start of the frame.
            // However if something throws an exception, it'll likely happen every frame and
            // spam the console, making firefox painfully slow. In that case, cancel the next frame.
            // Note that rust panics seem to throw exceptions but ALSO abort the program after the catch runs.
            const handle = window.requestAnimationFrame(frame);

            try {
                // Hack to test lower FPS - skip some frames as if they never happened.
                // TODO remove when physics/gamelogic have a separate framerate
                if (t_real - last_frame_t_real < window.min_frame_delay) {
                    return;
                }
                const diff_real = t_real - last_frame_t_real;
                last_frame_t_real = t_real;

                if (paused) {
                    return;
                }

                const diff_scaled = diff_real * cvars.d_speed;
                const t_scaled = last_frame_t_scaled + diff_scaled;
                last_frame_t_scaled = t_scaled;

                // In case it was updated using console.
                // Does not trigger the `input` / `change` events.
                // Check the label instead of the actual value to avoid floating point comparisons
                // and the related issues with rounding.
                if (speed_value.innerHTML !== cvars.d_speed.toString()) {
                    speed_slider.value = cvars.d_speed;
                    speed_value.innerHTML = cvars.d_speed;
                }

                game.update_and_render(t_scaled, input, cvars);
            } catch (e) {
                console.log("exception - aborting next frame");
                window.cancelAnimationFrame(handle);
                throw e;
            }
        };
        window.requestAnimationFrame(frame);
    }

    // TODO load assets at the same time - in parallel or packed into an archive
    load_tex_list();
}

run();
