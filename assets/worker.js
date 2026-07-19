// assets/worker.js

// Dioxus serves the compiled Wasm bindings at the root of the server
import init, { run_stars_worker } from '/yasf.js';

self.onmessage = async (event) => {
    await init();
    const targetStars = event.data;
    const resultString = run_stars_worker(targetStars);
    self.postMessage(resultString);
};