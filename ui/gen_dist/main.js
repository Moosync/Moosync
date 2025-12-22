import * as tauriApi from '@tauri-apps/api';

import init from '../leptos/moosync_ui_bindgen/moosync_ui_bindgen.js';

window.__TAURI__ = tauriApi;

async function run() {
    await init();
}

run();