// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { World } from '../pkg/rec_wars.js';

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

    let img_base = new Image();
    img_base.src = "../assets/tiles/Base.bmp";
    let img_explosion = new Image();
    img_explosion.src = "../assets/Explosion.png"
    // https://stackoverflow.com/questions/46399223/async-await-in-image-loading
    // https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/createImageBitmap

    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext("2d", { alpha: false });

    let left = 0, right = 0, up = 0, down = 0;

    document.addEventListener('keydown', event => {
        if (event.key === "ArrowLeft") {
            left = 1;
        } else if (event.key === "ArrowRight") {
            right = 1;
        } else if (event.key === "ArrowUp") {
            up = 1;
        } else if (event.key === "ArrowDown") {
            down = 1;
        }
    });

    document.addEventListener('keyup', event => {
        if (event.key === "ArrowLeft") {
            left = 0;
        } else if (event.key === "ArrowRight") {
            right = 0;
        } else if (event.key === "ArrowUp") {
            up = 0;
        } else if (event.key === "ArrowDown") {
            down = 0;
        }
    });

    const world = new World(ctx, canvas.width, canvas.height);

    const frame = (t) => {
        world.input(left, right, up, down);
        world.update(t);
        world.draw(img_base, img_explosion);
        window.requestAnimationFrame(frame);
    };
    window.requestAnimationFrame(frame);
}

run();
