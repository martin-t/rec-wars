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

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

ctx.fillStyle = "green";
ctx.fillRect(50, 100, 50, 60);

const img_base = document.getElementById("img-base");
img_base.addEventListener("load", draw_base);

function draw_base(event) {
    console.log("draw " + event + " " + img_base.complete, " ", img_base.naturalWidth);
    ctx.drawImage(img_base, 100, 100);
}

const img_rhino = document.getElementById("img-rhino");
img_rhino.addEventListener("load", draw_rhino);

function draw_rhino(event) {
    console.log("draw rhino");
    ctx.drawImage(img_rhino, 164, 100);
}

let log_frames = false;

function btn_click() {
    log_frames = !log_frames;
}

const fps_period_ms = 1000;

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
        ctx.clearRect(this.x - 10, this.y - 10, 200, 100);

        // time since page load (rounded to whole ms)
        const ms = performance.now();
        ctx.fillText("time: " + ms.toString(), this.x, this.y);

        // frames in a given period
        this.frame_times.push(ms);
        while (this.frame_times.length > 0 && this.frame_times[0] <= ms - fps_period_ms) {
            this.frame_times.shift();
        }
        ctx.fillText("last " + fps_period_ms + " ms: " + this.frame_times.length + " frames", this.x, this.y + 10);

        // average frame delay over the recorded period
        if (this.frame_times.length > 0) {
            const oldest = this.frame_times[0];
            const ms_per_frame = (ms - oldest) / (this.frame_times.length - 1);
            const fps = (1 / ms_per_frame) * 1000;
            const fps_rounded = Math.round(fps * 10) / 10;
            ctx.fillText("fps: " + fps_rounded, this.x, this.y + 20);
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
        ctx.fillText("prev second: " + this.prev_cnt + " frames", this.x, this.y + 30);

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
            console.log("time: " + ms.toString()
                + " last " + fps_period_ms
                + " ms: " + this.frame_times.length
                + " ms/frame: " + delta_ms
                + " fps: " + delays_mean);
        }
    }
}

const fps_anim_frame = new Fps(20, 20);
const fps_timeout = new Fps(220, 20);
const fps_interval = new Fps(420, 20);

const draw_delay = 1000 / 120;

function draw_frame() {
    window.requestAnimationFrame(draw_frame);

    fps_anim_frame.draw();
}

function draw_frame_timeout() {
    window.setTimeout(draw_frame_timeout, draw_delay);

    fps_timeout.draw();
}

function draw_frame_interval() {
    fps_interval.draw();
}

document.addEventListener('visibilitychange', function () {
    if (document.hidden) {
        console.log("hidden");
    } else {
        console.log("visible");
    }
});

window.requestAnimationFrame(draw_frame);
window.setTimeout(draw_frame_timeout, draw_delay);
window.setInterval(draw_frame_interval, draw_delay);
