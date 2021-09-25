import wasm_bindgen from './web.js';

function nothing(module) {}

wasm_bindgen("web_bg.wasm")
    .then(nothing)
    .catch(console.error);
