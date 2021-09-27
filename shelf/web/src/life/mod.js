import initialize, { Context, Cell } from "./web.js";


/*const sending = browser.runtime.sendNativeMessage("jago", 'popup');

sending.then(
    function (got) { console.log(got) },
    function (error) { console.error(error) },
);

browser.tabs.query({
    currentWindow: true,
})
    .then(function (tabs) {
        console.log(tabs)
    })
    .catch(function (error) {
        console.error(error)
    });
*/

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    // Render the statistics.
    this.fps.textContent = `
Frames per Second:
         latest = ${Math.round(fps)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }
};

const CELL_SIZE = 5;
const GRID_COLOR = "#FFF";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const root = document.getElementById("universe");
const controlPanel = document.getElementById("controls");

const control = document.createElement('button');

let animationId = null;

const isPaused = () => {
  return animationId === null;
};

const play = () => {
  control.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  control.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

control.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

controlPanel.appendChild(control);

let rate = 9;

const slider = document.createElement('input');
slider.type = 'range';
slider.value = rate;
slider.addEventListener("change", event => {
  rate = event.target.valueAsNumber;
});

controlPanel.appendChild(slider);

function createCanvas(){
    var canvas = document.createElement("canvas");
    canvas.style.position = "absolute";
    canvas.style.left     = "0px";
    canvas.style.top      = "0px";
    canvas.style.zIndex   = 1;
    canvas.width  = window.innerWidth;
    canvas.height = window.innerHeight;
    root.appendChild(canvas);
    return canvas;
}

let canvas = createCanvas();

function sizecanvas(canvas){
    canvas.width  = window.innerWidth;
    canvas.height = window.innerHeight;
    if (animationId) {
      reset()
    }
}

sizecanvas(canvas)

const bounds = root.getBoundingClientRect();

const height = Math.floor(bounds.height / CELL_SIZE);
const width = Math.floor(bounds.width / CELL_SIZE);

let universe = null;

function reset() {
  universe = Context.from_width_height(width, height);
}

canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  if (event.metaKey) {
    universe.add_glider(row, col);
  } else {
    universe.toggle_cell(row, col);
  }

  drawGrid();
  drawCells();
});

root.appendChild(canvas);

const resetRandom = document.createElement("button");

resetRandom.innerText = 'Reset random';

resetRandom.addEventListener("click", event => {
  reset()
});

controlPanel.appendChild(resetRandom);

const resetEmpty = document.createElement("button");

resetEmpty.innerText = 'Reset empty';

resetEmpty.addEventListener("click", event => {
  universe = Universe.new_empty();
});

controlPanel.appendChild(resetEmpty);

const ctx = canvas.getContext('2d');

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
};

const drawCells = (memory) => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

  const bitIsSet = (n, arr) => {
    const byte = Math.floor(n / 8);
    const mask = 1 << (n % 8);
    return (arr[byte] & mask) === mask;
  };

  ctx.beginPath();

  ctx.fillStyle = ALIVE_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (bitIsSet(idx, cells)) {
        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
      }
    }
  }

  ctx.fillStyle = DEAD_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if (!bitIsSet(idx, cells)) {
        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
      }
    }
  }

  ctx.stroke();
};

const renderLoop = (memory) => {
  fps.render();

  for (let i = 0; i < rate; i++) {
    universe.tick();
  }

  drawGrid();
  drawCells(memory);

  animationId = requestAnimationFrame(renderLoop);
};

initialize().then(wasm => {
    universe = Context.from_width_height(width, height);
    play(wasm.memory);
})
    .catch(error => console.error('failed to initialize', error));
