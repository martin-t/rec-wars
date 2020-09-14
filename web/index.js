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
    let img_rocket = new Image();
    img_rocket.src = "../assets/weapons/rocket.png";
    let img_gm = new Image();
    img_gm.src = "../assets/weapons/guided_missile.png";
    let img_tank_green = new Image();
    img_tank_green.src = "../assets/tank_green.png";
    let img_tank_red = new Image();
    img_tank_red.src = "../assets/tank_red.png";
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

    document.addEventListener("keydown", event => {
        if (log_time_checkbox.checked) {
            console.log(performance.now(), "down", event.key);
        }

        // TODO IE/edge?
        //  https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key
        //  https://www.w3.org/TR/uievents-key
        // TODO possible to disable shortcuts like CTRL+W? binds without modifiers?
        // Single letters need toLowerCase to be detected when shift is held.
        if (event.key === "ArrowLeft" || event.key.toLowerCase() === "a") {
            input.left = 1;
        } else if (event.key === "ArrowRight" || event.key.toLowerCase() === "d") {
            input.right = 1;
        } else if (event.key === "ArrowUp" || event.key.toLowerCase() === "w") {
            input.up = 1;
        } else if (event.key === "ArrowDown" || event.key.toLowerCase() === "s") {
            input.down = 1;
        } else if (event.key.toLowerCase() === "n") {
            input.turret_left = true;
        } else if (event.key.toLowerCase() === "m") {
            input.turret_right = true;
        } else if (event.key.toLowerCase() === "v" || event.key === ".") {
            input.prev_weapon = true;
        } else if (event.key === "Shift" || event.key.toLowerCase() === "b" || event.key === ",") {
            input.next_weapon = true;
        } else if (event.key === " ") {
            input.fire = true;
        } else if (event.key == "Pause" || event.key.toLowerCase() === "p") {
            paused = !paused;
        }
    });

    document.addEventListener("keyup", event => {
        if (log_time_checkbox.checked) {
            console.log(performance.now(), "up", event.key);
        }

        if (event.key === "ArrowLeft" || event.key.toLowerCase() === "a") {
            input.left = 0;
        } else if (event.key === "ArrowRight" || event.key.toLowerCase() === "d") {
            input.right = 0;
        } else if (event.key === "ArrowUp" || event.key.toLowerCase() === "w") {
            input.up = 0;
        } else if (event.key === "ArrowDown" || event.key.toLowerCase() === "s") {
            input.down = 0;
        } else if (event.key.toLowerCase() === "n") {
            input.turret_left = false;
        } else if (event.key.toLowerCase() === "m") {
            input.turret_right = false;
        } else if (event.key.toLowerCase() === "v" || event.key === ".") {
            input.prev_weapon = false;
        } else if (event.key === "Shift" || event.key.toLowerCase() === "b" || event.key === ",") {
            input.next_weapon = false;
        } else if (event.key === " ") {
            input.fire = false;
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
                return;
            }

            load_map(request.responseText);
        }
        request.send();
    }

    let load_map = (tex_list_text) => {
        let request = new XMLHttpRequest();
        request.open("GET", "../maps/Atrium.map");
        request.onloadend = () => {
            if (request.status !== 200) {
                console.log("Failed to load map: ", request);
                return;
            }

            play(tex_list_text, request.responseText);
        }
        request.send();
    }

    let play = (tex_list_text, map_text) => {
        // Cvars can be changed through the browser's console.
        // For now, they need to live on the JS heap and be passed into each function that needs them.
        // I couldn't find a better way to make them mutable in JS and readable in Rust:
        // - can't return references from Rust into JS
        // - can't have a reference in Game
        // - owned pub cvars in Game need to be copy -> can't be changed from JS (changing will have no effect)
        // - LATER try returning Rc/Arc
        const cvars = new Cvars();
        const game = new Game(cvars, ctx, canvas.width, canvas.height,
            imgs_tiles, imgs_vehicles, imgs_weapon_icons, img_rocket, img_gm, img_tank_green, img_tank_red, img_explosion,
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

                game.update_and_draw(t_scaled, input, cvars);
            } catch (e) {
                console.log("exception - aborting next frame");
                window.cancelAnimationFrame(handle);
                throw e;
            }
        };
        window.requestAnimationFrame(frame);
    }

    // LATER there's gotta be a way to avoid this retarded chain
    load_tex_list();
}

run();
