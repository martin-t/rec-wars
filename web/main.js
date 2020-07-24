"use strict";

var importObject = { imports: { imported_func: arg => console.log(arg) } };

WebAssembly.instantiateStreaming(fetch('simple.wasm'), importObject)
    .then(results => {
        results.instance.exports.exported_func();
    });

WebAssembly.instantiateStreaming(fetch('handwritten.wasm'), importObject)
    .then(results => {
        console.log(results.instance.exports.exp_add(5, 6));
    });

let img_base = new Image();
img_base.src = "../assets/tiles/Base.bmp";

let img_g3 = new Image();
img_g3.src = "../assets/tiles/g3.bmp";

let img_explosion = new Image();
img_explosion.src = "../assets/Explosion.png"

const FPS_PERIOD_MS = 500;
let fps_log = false;

const ANIM_FRAME = 0;
const SET_TIMEOUT = 1;
const SET_INTERVAL = 2;
let anim_method = ANIM_FRAME;
let anim_handle = null;

let align_to_pixels = true;

const draw_delay = 1000 / 60;

//const canvas = document.getElementById("canvas");
//const ctx = canvas.getContext("2d", { alpha: false });

//anim_handle = window.requestAnimationFrame(draw_frame_animation);

class Fps {
    constructor(x, y) {
        this.frame_times = [];

        this.cur_cnt = 0;
        this.prev_cnt = 0;
        this.prev_second = 0;

        this.prev_ms = 0;

        this.frame_delays = [];
    }

    /** Several different methods of counting FPS. */
    draw(x, y, ms) {
        // frames in a given period
        this.frame_times.push(ms);
        while (this.frame_times.length > 0 && this.frame_times[0] <= ms - FPS_PERIOD_MS) {
            this.frame_times.shift();
        }
        ctx.fillText("last " + FPS_PERIOD_MS + " ms: " + this.frame_times.length, x, y);

        // average frame delay over the recorded period
        let fps_exact_rounded = 0;
        if (this.frame_times.length > 0) {
            const oldest = this.frame_times[0];
            const ms_per_frame = (ms - oldest) / (this.frame_times.length - 1);
            const fps = (1 / ms_per_frame) * 1000;
            fps_exact_rounded = Math.round(fps * 10) / 10;
            ctx.fillText("fps: " + fps_exact_rounded, x, y + 10);
        }

        // frames in the previous whole second
        const second = Math.round(ms / 1000);
        if (this.prev_second === second) {
            this.cur_cnt++;
        } else {
            this.prev_second = second;
            this.prev_cnt = this.cur_cnt;
            this.cur_cnt = 1;
        }
        ctx.fillText("prev second: " + this.prev_cnt, x, y + 20);

        // delay between frames
        const delta_ms = ms - this.prev_ms;
        const delta_ms_rounded = Math.round(delta_ms * 10) / 10;
        ctx.fillText("ms/frame: " + delta_ms_rounded, x, y + 30);
        this.prev_ms = ms;

        // fps using average of the delays - similar to what sandspiel does
        this.frame_delays.push(delta_ms);
        if (this.frame_delays.length > 30) {
            this.frame_delays.shift();
        }
        let delays_sum = 0;
        this.frame_delays.forEach(delay_ms => {
            delays_sum += delay_ms;
        });
        const delays_mean = delays_sum / this.frame_delays.length;
        const fps = (1 / delays_mean) * 1000;
        const fps_rounded = Math.round(fps * 10) / 10;
        ctx.fillText("fps: " + fps_rounded, x, y + 40);

        if (fps_log) {
            console.log("time: " + ms
                + "\t last " + FPS_PERIOD_MS + " ms: " + this.frame_times.length
                + "\t fps: " + fps_exact_rounded
                + "\t prev second: " + this.prev_cnt
                + "\t ms/frame: " + delta_ms_rounded
                + "\t fps: " + fps_rounded);
        }
    }
}

const fps_anim_frame = new Fps(20, 30);

// It looks like the original animation is made for 30 fps.
// When stepping through frames of a recording, some images take 3 frames,
// might be a bug in mplayer though.
const explosions_fast = [];
const explosions_slow = [];

let offset_x = 0;
let offset_y = 0;
let speed_x = 0;
let speed_y = 0;

function draw_frame(t_frame) {
    const t_start = performance.now();

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const draw_offset_x = align_to_pixels ? Math.round(offset_x) : offset_x;
    const draw_offset_y = align_to_pixels ? Math.round(offset_y) : offset_y;
    for (let x = draw_offset_x - 64; x < canvas.width; x += 64) {
        for (let y = draw_offset_y - 64; y < canvas.height; y += 64) {
            ctx.drawImage(img_base, x, y);
        }
    }
    offset_x = (offset_x + 64 + speed_x) % 64;
    offset_y = (offset_y + 64 + speed_y) % 64;

    for (let i = 0; i < 2; i++) {
        let x = canvas.width / 2 + Math.random() * (canvas.width / 2);
        let y = Math.random() * (canvas.height / 2);
        explosions_fast.push({ x: x, y: y, frame: 0 });
        while (explosions_fast.length > 0 && explosions_fast[0].frame >= 13) {
            explosions_fast.shift();
        }
    }
    for (let i = 0; i < 1; i++) {
        let x = canvas.width / 2 + Math.random() * (canvas.width / 2);
        let y = canvas.height / 2 + Math.random() * (canvas.height / 2);
        explosions_slow.push({ x: x, y: y, frame: 0 });
        while (explosions_slow.length > 0 && explosions_slow[0].frame >= 26) {
            explosions_slow.shift();
        }
    }

    explosions_fast.forEach(explosion => {
        const offset = explosion.frame * 100;
        ctx.drawImage(
            img_explosion,
            offset, 0, 100, 100,
            explosion.x, explosion.y, 100, 100);
        explosion.frame++;
    });
    explosions_slow.forEach(explosion => {
        const frame = Math.round(explosion.frame / 2);
        const offset = frame * 100;
        ctx.drawImage(
            img_explosion,
            offset, 0, 100, 100,
            explosion.x, explosion.y, 100, 100);
        explosion.frame++;
    });

    const t_end = performance.now();
    const diff = t_end - t_start;

    ctx.fillStyle = "red";
    ctx.fillText("t_frame: " + t_frame, 20, 20);
    ctx.fillText("t_start: " + t_start, 20, 30);
    ctx.fillText("t_end: " + t_end, 20, 40);
    ctx.fillText("diff: " + diff, 20, 50);
    fps_anim_frame.draw(20, 60, t_frame);
}

function draw_frame_animation(t) {
    // t has 2 decimals
    // performance.now() is rounded to whole ms
    anim_handle = window.requestAnimationFrame(draw_frame_animation);
    draw_frame(t);
}

function draw_frame_timeout() {
    anim_handle = window.setTimeout(draw_frame_timeout, draw_delay);
    draw_frame(performance.now());
}

function draw_frame_interval() {
    draw_frame(performance.now());
}

function btn_fps_log_click() {
    let btn = document.getElementById("btn_fps_log");
    if (fps_log) {
        fps_log = false;
        btn.innerHTML = "FPS log: off";
    } else {
        fps_log = true;
        btn.innerHTML = "FPS log: on";
    }
}

function btn_anim_method_click() {
    if (anim_handle === null) return;

    let btn = document.getElementById("btn_anim_method");
    if (anim_method === ANIM_FRAME) {
        anim_method = SET_TIMEOUT;
        window.cancelAnimationFrame(anim_handle);
        anim_handle = window.setTimeout(draw_frame_timeout, draw_delay);
        btn.innerHTML = "Anim method: setTimeout";
    } else if (anim_method === SET_TIMEOUT) {
        anim_method = SET_INTERVAL;
        window.clearInterval(anim_handle)
        anim_handle = window.setInterval(draw_frame_interval, draw_delay);
        btn.innerHTML = "Anim method: setInterval";
    } else {
        anim_method = ANIM_FRAME;
        window.clearTimeout(anim_handle);
        anim_handle = window.requestAnimationFrame(draw_frame_animation);
        btn.innerHTML = "Anim method: requestAnimationFrame";
    }
}

function btn_align_click() {
    let btn = document.getElementById("btn_align");
    if (align_to_pixels) {
        align_to_pixels = false;
        btn.innerHTML = "Align movement to whole pixels: off";
    } else {
        align_to_pixels = true;
        btn.innerHTML = "Align movement to whole pixels: on";
    }
}

document.addEventListener('keydown', event => {
    //console.log(event.timeStamp, event.key);
    if (event.key === "ArrowLeft") {
        speed_x += 0.1;
    } else if (event.key === "ArrowRight") {
        speed_x -= 0.1;
    } else if (event.key === "ArrowUp") {
        speed_y += 0.1;
    } else if (event.key === "ArrowDown") {
        speed_y -= 0.1;
    }
});

/*document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
        console.log("hidden");
    } else {
        console.log("visible");
    }
});*/
