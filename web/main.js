"use strict";

//var importObject = { imports: { alert: s => alert(s) } };
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
img_explosion.src = "../assets/Explosion.bmp"

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

let log_frames = false;

function btn_fps_log_click() {
    log_frames = !log_frames;
}

const fps_period_ms = 500;

class Fps {
    constructor(x, y) {
        this.x = x;
        this.y = y;

        this.frame_times = [];

        this.cur_cnt = 0;
        this.prev_cnt = 0;
        this.prev_second = 0;

        this.prev_ms = 0;

        this.frame_delays = [];
    }

    /** Several different methods of counting FPS. */
    draw() {
        ctx.fillStyle = "red";

        // time since page load (rounded to whole ms)
        const ms = performance.now();
        ctx.fillText("time: " + ms, this.x, this.y);

        // frames in a given period
        this.frame_times.push(ms);
        while (this.frame_times.length > 0 && this.frame_times[0] <= ms - fps_period_ms) {
            this.frame_times.shift();
        }
        ctx.fillText("last " + fps_period_ms + " ms: " + this.frame_times.length, this.x, this.y + 10);

        // average frame delay over the recorded period
        let fps_exact_rounded = 0;
        if (this.frame_times.length > 0) {
            const oldest = this.frame_times[0];
            const ms_per_frame = (ms - oldest) / (this.frame_times.length - 1);
            const fps = (1 / ms_per_frame) * 1000;
            fps_exact_rounded = Math.round(fps * 10) / 10;
            ctx.fillText("fps: " + fps_exact_rounded, this.x, this.y + 20);
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
        ctx.fillText("prev second: " + this.prev_cnt, this.x, this.y + 30);

        // delay between frames (rounded)
        const delta_ms = ms - this.prev_ms;
        ctx.fillText("ms/frame: " + delta_ms, this.x, this.y + 40);
        this.prev_ms = ms;

        // fps using average of the (rounded) delays - similar to what sandspiel does
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
        ctx.fillText("fps: " + fps_rounded, this.x, this.y + 50);

        if (log_frames) {
            console.log("time: " + ms
                + "\t last " + fps_period_ms + " ms: " + this.frame_times.length
                + "\t fps: " + fps_exact_rounded
                + "\t prev second: " + this.prev_cnt
                + "\t ms/frame: " + delta_ms
                + "\t fps: " + fps_rounded);
        }
    }
}

const fps_anim_frame = new Fps(20, 20);

const explosions = []

function draw_frame() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let x = 0; x < canvas.width; x += 64) {
        for (let y = 0; y < canvas.height; y += 64) {
            ctx.drawImage(img_base, x, y);
        }
    }

    for (let i = 0; i < 2; i++) {
        let x = Math.random() * canvas.width;
        let y = Math.random() * canvas.height;
        explosions.push({ x: x, y: y, frame: 0 });
        while (explosions.length > 0 && explosions[0].frame >= 13) {
            explosions.shift();
        }
    }

    explosions.forEach(explosion => {
        const offset = explosion.frame * 100;
        ctx.drawImage(
            img_explosion,
            offset, 0, 100, 100,
            explosion.x, explosion.y, 100, 100);
        explosion.frame++;
    });

    fps_anim_frame.draw();

    // TODO at the end for testing so any error stops the animation
    window.requestAnimationFrame(draw_frame);
}
window.requestAnimationFrame(draw_frame);

/*const fps_timeout = new Fps(220, 20);
const fps_interval = new Fps(420, 20);

const draw_delay = 1000 / 120;

function draw_frame_timeout() {
    window.setTimeout(draw_frame_timeout, draw_delay);

    fps_timeout.draw();
}

function draw_frame_interval() {
    fps_interval.draw();
}

window.setTimeout(draw_frame_timeout, draw_delay);
window.setInterval(draw_frame_interval, draw_delay);*/

/*document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
        console.log("hidden");
    } else {
        console.log("visible");
    }
});*/
