// pkg/rust48.js
var Hp48 = class {
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    Hp48Finalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_hp48_free(ptr, 0);
  }
  /**
   * Get current annunciator state as bitmask.
   * @returns {number}
   */
  annunciator_state() {
    const ret = wasm.hp48_annunciator_state(this.__wbg_ptr);
    return ret >>> 0;
  }
  clear_display_dirty() {
    wasm.hp48_clear_display_dirty(this.__wbg_ptr);
  }
  /**
   * Get pointer to the RGBA display buffer (for use with WASM memory).
   * @returns {number}
   */
  display_buffer_ptr() {
    const ret = wasm.hp48_display_buffer_ptr(this.__wbg_ptr);
    return ret >>> 0;
  }
  /**
   * @returns {number}
   */
  display_height() {
    const ret = wasm.hp48_display_height(this.__wbg_ptr);
    return ret >>> 0;
  }
  /**
   * @returns {number}
   */
  display_width() {
    const ret = wasm.hp48_display_width(this.__wbg_ptr);
    return ret >>> 0;
  }
  /**
   * @returns {boolean}
   */
  is_display_dirty() {
    const ret = wasm.hp48_is_display_dirty(this.__wbg_ptr);
    return ret !== 0;
  }
  /**
   * Create a new emulator instance.
   * `rom` — ROM data (nibble or packed byte format).
   * `ram` — optional RAM data (nibble or packed byte format).
   * `state` — optional saved state (binary format from save_state).
   * @param {Uint8Array} rom
   * @param {Uint8Array | null} [ram]
   * @param {Uint8Array | null} [state]
   */
  constructor(rom, ram, state) {
    const ptr0 = passArray8ToWasm0(rom, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(ram) ? 0 : passArray8ToWasm0(ram, wasm.__wbindgen_export);
    var len1 = WASM_VECTOR_LEN;
    var ptr2 = isLikeNone(state) ? 0 : passArray8ToWasm0(state, wasm.__wbindgen_export);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.hp48_new(ptr0, len0, ptr1, len1, ptr2, len2);
    this.__wbg_ptr = ret >>> 0;
    Hp48Finalization.register(this, this.__wbg_ptr, this);
    return this;
  }
  /**
   * Push a key event into the queue.
   * Bit 31 clear = press, bit 31 set = release.
   * Bits [7:4] = row, bits [3:0] = column.
   * @param {number} code
   */
  push_key_event(code) {
    wasm.hp48_push_key_event(this.__wbg_ptr, code);
  }
  /**
   * Run one frame of emulation.
   * `elapsed_ms` — milliseconds since last frame.
   * `now_secs` — current time in seconds.
   * @param {number} elapsed_ms
   * @param {number} now_secs
   */
  run_frame(elapsed_ms, now_secs) {
    wasm.hp48_run_frame(this.__wbg_ptr, elapsed_ms, now_secs);
  }
  /**
   * Serialize RAM to packed byte format.
   * @returns {Uint8Array}
   */
  save_ram() {
    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      wasm.hp48_save_ram(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      var v1 = getArrayU8FromWasm0(r0, r1).slice();
      wasm.__wbindgen_export2(r0, r1 * 1, 1);
      return v1;
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
  /**
   * Serialize CPU state to binary format (compatible with C version).
   * @returns {Uint8Array}
   */
  save_state() {
    try {
      const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
      wasm.hp48_save_state(retptr, this.__wbg_ptr);
      var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
      var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
      var v1 = getArrayU8FromWasm0(r0, r1).slice();
      wasm.__wbindgen_export2(r0, r1 * 1, 1);
      return v1;
    } finally {
      wasm.__wbindgen_add_to_stack_pointer(16);
    }
  }
  /**
   * Get detected speaker frequency in Hz (0 = no tone).
   * Call every ~20ms from JS.
   * @returns {number}
   */
  speaker_frequency() {
    const ret = wasm.hp48_speaker_frequency(this.__wbg_ptr);
    return ret >>> 0;
  }
  /**
   * Start emulation timers. Call once after construction.
   * `now_secs` — monotonic time in seconds (e.g. performance.now() / 1000).
   * `unix_epoch_secs` — wall-clock seconds since Unix epoch, local time
   *   (e.g. Date.now()/1000 - new Date().getTimezoneOffset()*60).
   * @param {number} now_secs
   * @param {number} unix_epoch_secs
   */
  start(now_secs, unix_epoch_secs) {
    wasm.hp48_start(this.__wbg_ptr, now_secs, unix_epoch_secs);
  }
};
if (Symbol.dispose) Hp48.prototype[Symbol.dispose] = Hp48.prototype.free;
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    }
  };
  return {
    __proto__: null,
    "./rust48_bg.js": import0
  };
}
var Hp48Finalization = typeof FinalizationRegistry === "undefined" ? { register: () => {
}, unregister: () => {
} } : new FinalizationRegistry((ptr) => wasm.__wbg_hp48_free(ptr >>> 0, 1));
function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
var cachedDataViewMemory0 = null;
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === void 0 && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return decodeText(ptr, len);
}
var cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}
function isLikeNone(x) {
  return x === void 0 || x === null;
}
function passArray8ToWasm0(arg, malloc) {
  const ptr = malloc(arg.length * 1, 1) >>> 0;
  getUint8ArrayMemory0().set(arg, ptr / 1);
  WASM_VECTOR_LEN = arg.length;
  return ptr;
}
var cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
var MAX_SAFARI_DECODE_BYTES = 2146435072;
var numBytesDecoded = 0;
function decodeText(ptr, len) {
  numBytesDecoded += len;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    numBytesDecoded = len;
  }
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}
var WASM_VECTOR_LEN = 0;
var wasmModule;
var wasm;
function __wbg_finalize_init(instance, module) {
  wasm = instance.exports;
  wasmModule = module;
  cachedDataViewMemory0 = null;
  cachedUint8ArrayMemory0 = null;
  return wasm;
}
async function __wbg_load(module, imports) {
  if (typeof Response === "function" && module instanceof Response) {
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);
      } catch (e) {
        const validResponse = module.ok && expectedResponseType(module.type);
        if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
        } else {
          throw e;
        }
      }
    }
    const bytes = await module.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);
  } else {
    const instance = await WebAssembly.instantiate(module, imports);
    if (instance instanceof WebAssembly.Instance) {
      return { instance, module };
    } else {
      return instance;
    }
  }
  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true;
    }
    return false;
  }
}
async function __wbg_init(module_or_path) {
  if (wasm !== void 0) return wasm;
  if (module_or_path !== void 0) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead");
    }
  }
  if (module_or_path === void 0) {
    module_or_path = new URL("rust48_bg.wasm", import.meta.url);
  }
  const imports = __wbg_get_imports();
  if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
    module_or_path = fetch(module_or_path);
  }
  const { instance, module } = await __wbg_load(await module_or_path, imports);
  return __wbg_finalize_init(instance, module);
}

// web/hp48_rust.ts
var DISPLAY_WIDTH = 131;
var DISPLAY_HEIGHT = 64;
var DISPLAY_BYTES = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;
var AUTO_SAVE_INTERVAL_MS = 3e4;
var DB_NAME = "hp48_rust";
var DB_STORE = "files";
function openDB() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(DB_STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}
async function dbGet(key) {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(DB_STORE, "readonly");
    const req = tx.objectStore(DB_STORE).get(key);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}
async function dbPut(key, value) {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(DB_STORE, "readwrite");
    tx.objectStore(DB_STORE).put(value, key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}
var BUTTON_KEYCODES = [
  // Row 0: menu keys A–F
  20,
  132,
  131,
  130,
  129,
  128,
  // Row 1: MTH PRG CST VAR UP NXT
  36,
  116,
  115,
  114,
  113,
  112,
  // Row 2: ' STO EVAL LEFT DOWN RIGHT
  4,
  100,
  99,
  98,
  97,
  96,
  // Row 3: SIN COS TAN SQRT POWER INV
  52,
  84,
  83,
  82,
  81,
  80,
  // Row 4: ENTER NEG EEX DEL BS
  68,
  67,
  66,
  65,
  64,
  // Row 5: ALPHA 7 8 9 DIV
  53,
  51,
  50,
  49,
  48,
  // Row 6: SHL 4 5 6 MUL
  37,
  35,
  34,
  33,
  32,
  // Row 7: SHR 1 2 3 MINUS
  21,
  19,
  18,
  17,
  16,
  // Row 8: ON 0 . SPC PLUS
  32768,
  3,
  2,
  1,
  0
];
function buttonToKeyEvent(btnId, press) {
  const keycode = BUTTON_KEYCODES[btnId];
  return press ? keycode : (keycode | 2147483648) >>> 0;
}
var KEY_MAP = {
  "0": 45,
  "1": 40,
  "2": 41,
  "3": 42,
  "4": 35,
  "5": 36,
  "6": 37,
  "7": 30,
  "8": 31,
  "9": 32,
  "Enter": 24,
  "Backspace": 28,
  "Delete": 27,
  ".": 46,
  "+": 48,
  "-": 43,
  "*": 38,
  "/": 33,
  " ": 47,
  "Escape": 44,
  "ArrowUp": 10,
  "ArrowDown": 16,
  "ArrowLeft": 15,
  "ArrowRight": 17,
  "'": 12
};
var ALPHA_MAP = {
  a: 0,
  b: 1,
  c: 2,
  d: 3,
  e: 4,
  f: 5,
  g: 6,
  h: 7,
  i: 8,
  j: 9,
  k: 10,
  l: 11,
  m: 12,
  n: 13,
  o: 14,
  p: 15,
  q: 16,
  r: 17,
  s: 18,
  t: 19,
  u: 20,
  v: 21,
  w: 22,
  x: 23,
  y: 25,
  z: 26
};
var ALPHA_BTN = 29;
var SHL_BTN = 34;
var SHR_BTN = 39;
var SHIFT_MAP = {
  "(": [SHL_BTN, 33],
  ")": [SHL_BTN, 33],
  "[": [SHL_BTN, 38],
  "]": [SHL_BTN, 38],
  "{": [SHL_BTN, 48],
  "}": [SHL_BTN, 48],
  "<": [SHL_BTN, 43],
  ">": [SHL_BTN, 43],
  "#": [SHR_BTN, 33],
  "_": [SHR_BTN, 38],
  '"': [SHR_BTN, 43],
  ":": [SHR_BTN, 48],
  ",": [SHL_BTN, 46]
};
var pressedKeys = /* @__PURE__ */ new Set();
var hp48;
var wasmMemory;
var ANN_WIDTH = 15;
var ANN_HEIGHT = 12;
var ANN_CANVAS_W = 262;
var ANN_CANVAS_H = 12;
var PIX_ON = [16, 32, 16, 255];
var PIX_OFF = [188, 196, 165, 255];
var ANN_DEFS = [
  { bit: 129, x: 16, bits: [
    // left shift
    254,
    63,
    255,
    127,
    159,
    127,
    207,
    127,
    231,
    127,
    3,
    120,
    3,
    112,
    231,
    115,
    207,
    115,
    159,
    115,
    255,
    115,
    254,
    51
  ] },
  { bit: 130, x: 61, bits: [
    // right shift
    254,
    63,
    255,
    127,
    255,
    124,
    255,
    121,
    255,
    115,
    15,
    96,
    7,
    96,
    231,
    115,
    231,
    121,
    231,
    124,
    231,
    127,
    230,
    63
  ] },
  { bit: 132, x: 106, bits: [
    // alpha
    224,
    3,
    24,
    68,
    12,
    76,
    6,
    44,
    7,
    44,
    7,
    28,
    7,
    12,
    7,
    12,
    7,
    14,
    14,
    77,
    248,
    56,
    0,
    0
  ] },
  { bit: 136, x: 151, bits: [
    // battery
    4,
    16,
    2,
    32,
    18,
    36,
    9,
    72,
    201,
    73,
    201,
    73,
    201,
    73,
    9,
    72,
    18,
    36,
    2,
    32,
    4,
    16,
    0,
    0
  ] },
  { bit: 144, x: 196, bits: [
    // busy
    252,
    31,
    8,
    8,
    8,
    8,
    240,
    7,
    224,
    3,
    192,
    1,
    64,
    1,
    32,
    2,
    16,
    4,
    200,
    9,
    232,
    11,
    252,
    31
  ] },
  { bit: 160, x: 241, bits: [
    // IO
    12,
    0,
    30,
    0,
    51,
    12,
    97,
    24,
    204,
    48,
    254,
    127,
    254,
    127,
    204,
    48,
    97,
    24,
    51,
    12,
    30,
    0,
    12,
    0
  ] }
];
var annBitmaps = ANN_DEFS.map((def) => {
  const rows = [];
  for (let y = 0; y < ANN_HEIGHT; y++) {
    const row = [];
    const b0 = def.bits[y * 2];
    const b1 = def.bits[y * 2 + 1];
    const rowBits = b0 | b1 << 8;
    for (let x = 0; x < ANN_WIDTH; x++) {
      row.push((rowBits >> x & 1) !== 0);
    }
    rows.push(row);
  }
  return rows;
});
function startDisplayLoop() {
  const canvas = document.getElementById("lcd");
  if (!canvas) {
    console.error("hp48: #lcd canvas not found");
    return;
  }
  canvas.width = DISPLAY_WIDTH;
  canvas.height = DISPLAY_HEIGHT;
  const ctx = canvas.getContext("2d");
  const imageData = ctx.createImageData(DISPLAY_WIDTH, DISPLAY_HEIGHT);
  const annCanvas = document.getElementById("annunciators");
  let annCtx = null;
  let annImageData = null;
  if (annCanvas) {
    annCanvas.width = ANN_CANVAS_W;
    annCanvas.height = ANN_CANVAS_H;
    annCtx = annCanvas.getContext("2d");
    annImageData = annCtx.createImageData(ANN_CANVAS_W, ANN_CANVAS_H);
    for (let i = 0; i < ANN_CANVAS_W * ANN_CANVAS_H; i++) {
      annImageData.data.set(PIX_OFF, i * 4);
    }
    annCtx.putImageData(annImageData, 0, 0);
  }
  let lastAnnunc = -1;
  function frame() {
    if (hp48.is_display_dirty()) {
      const ptr = hp48.display_buffer_ptr();
      const src = new Uint8Array(wasmMemory.buffer, ptr, DISPLAY_BYTES);
      imageData.data.set(src);
      hp48.clear_display_dirty();
      ctx.putImageData(imageData, 0, 0);
    }
    const annunc = hp48.annunciator_state();
    if (annunc !== lastAnnunc && annCtx && annImageData) {
      lastAnnunc = annunc;
      const d = annImageData.data;
      for (let i = 0; i < ANN_CANVAS_W * ANN_CANVAS_H; i++) {
        d[i * 4] = PIX_OFF[0];
        d[i * 4 + 1] = PIX_OFF[1];
        d[i * 4 + 2] = PIX_OFF[2];
        d[i * 4 + 3] = PIX_OFF[3];
      }
      for (let a = 0; a < ANN_DEFS.length; a++) {
        if ((annunc & ANN_DEFS[a].bit) !== ANN_DEFS[a].bit) continue;
        const bx = ANN_DEFS[a].x;
        const bmp = annBitmaps[a];
        for (let y = 0; y < ANN_HEIGHT; y++) {
          for (let x = 0; x < ANN_WIDTH; x++) {
            if (bmp[y][x]) {
              const idx = (y * ANN_CANVAS_W + bx + x) * 4;
              d[idx] = PIX_ON[0];
              d[idx + 1] = PIX_ON[1];
              d[idx + 2] = PIX_ON[2];
              d[idx + 3] = PIX_ON[3];
            }
          }
        }
      }
      annCtx.putImageData(annImageData, 0, 0);
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
}
function setupButtonInput() {
  const buttons = document.querySelectorAll("[data-btn]");
  buttons.forEach((el) => {
    const btnId = parseInt(el.dataset.btn, 10);
    if (isNaN(btnId)) return;
    function press() {
      el.classList.add("pressed");
      hp48.push_key_event(buttonToKeyEvent(btnId, true));
    }
    function release() {
      el.classList.remove("pressed");
      hp48.push_key_event(buttonToKeyEvent(btnId, false));
    }
    el.addEventListener("mousedown", (e) => {
      e.preventDefault();
      press();
    });
    el.addEventListener("mouseup", () => {
      release();
    });
    el.addEventListener("mouseleave", () => {
      if (el.classList.contains("pressed")) release();
    });
    el.addEventListener("touchstart", (e) => {
      e.preventDefault();
      press();
    }, { passive: false });
    el.addEventListener("touchend", (e) => {
      e.preventDefault();
      release();
    }, { passive: false });
    el.addEventListener("touchcancel", (e) => {
      e.preventDefault();
      release();
    }, { passive: false });
  });
  document.getElementById("buttons")?.addEventListener("contextmenu", (e) => e.preventDefault());
}
function pushKeySequence(events, delay = 20) {
  events.forEach((code, i) => {
    if (i === 0) {
      hp48.push_key_event(code);
    } else {
      setTimeout(() => hp48.push_key_event(code), i * delay);
    }
  });
}
function setupKeyboardInput() {
  document.addEventListener("keydown", (e) => {
    const btnId = KEY_MAP[e.key];
    if (btnId !== void 0) {
      if (pressedKeys.has(e.key)) return;
      pressedKeys.add(e.key);
      e.preventDefault();
      hp48.push_key_event(buttonToKeyEvent(btnId, true));
      return;
    }
    const shiftCombo = SHIFT_MAP[e.key];
    if (shiftCombo !== void 0) {
      if (e.repeat) return;
      e.preventDefault();
      const [shiftBtn, targetBtn] = shiftCombo;
      pushKeySequence([
        buttonToKeyEvent(shiftBtn, true),
        buttonToKeyEvent(targetBtn, true),
        buttonToKeyEvent(targetBtn, false),
        buttonToKeyEvent(shiftBtn, false)
      ]);
      return;
    }
    const alphaBtn = ALPHA_MAP[e.key.toLowerCase()];
    if (alphaBtn !== void 0) {
      if (e.repeat || e.ctrlKey || e.metaKey || e.altKey) return;
      e.preventDefault();
      pushKeySequence([
        buttonToKeyEvent(ALPHA_BTN, true),
        buttonToKeyEvent(alphaBtn, true),
        buttonToKeyEvent(alphaBtn, false),
        buttonToKeyEvent(ALPHA_BTN, false)
      ]);
      return;
    }
  });
  document.addEventListener("keyup", (e) => {
    const btnId = KEY_MAP[e.key];
    if (btnId === void 0) return;
    pressedKeys.delete(e.key);
    e.preventDefault();
    hp48.push_key_event(buttonToKeyEvent(btnId, false));
  });
}
function showCalculator() {
  const loading = document.getElementById("loading");
  if (loading) loading.style.display = "none";
}
var SVG_NS = "http://www.w3.org/2000/svg";
var FONT_STACK = "'Helvetica Neue',Arial,Helvetica,sans-serif";
var LS_COLOR = "#8B6BA0";
var RS_COLOR = "#5AABB8";
var BODY_GRAD = ["#141c2c", "#2c3448"];
var B6 = { x: 22, y: 33, w: 96, h: 69, rx: 20 };
var B5 = { x: 18, y: 33, w: 126, h: 69, rx: 16 };
var BN = { x: 22, y: 33, w: 96, h: 69, rx: 20 };
var BE = { x: 22, y: 33, w: 226, h: 69, rx: 20 };
var BUTTONS = [
  // Row 0: Menu keys (btn 0–5) with alpha labels A–F
  { face: "", menu: true, alpha: "A" },
  { face: "", menu: true, alpha: "B" },
  { face: "", menu: true, alpha: "C" },
  { face: "", menu: true, alpha: "D" },
  { face: "", menu: true, alpha: "E" },
  { face: "", menu: true, alpha: "F" },
  // Row 1 (btn 6–11)
  { face: "MTH", ls: "RAD", rs: "POLAR", alpha: "G" },
  { face: "PRG", rs: "CHARS", alpha: "H" },
  { face: "CST", rs: "MODES", alpha: "I" },
  { face: "VAR", rs: "MEMORY", alpha: "J" },
  { face: "\u25B2", rs: "STACK", alpha: "K" },
  { face: "NXT", ls: "PREV", rs: "MENU", alpha: "L" },
  // Row 2 (btn 12–17)
  { face: "'", ls: "UP", rs: "HOME", alpha: "M" },
  { face: "STO", ls: "DEF", rs: "RCL", alpha: "N" },
  { face: "EVAL", ls: "\u2192NUM", rs: "UNDO", alpha: "O" },
  { face: "\u25C0", ls: "PICTURE", alpha: "P" },
  { face: "\u25BC", ls: "VIEW", alpha: "Q" },
  { face: "\u25B6", ls: "SWAP", alpha: "R" },
  // Row 3 (btn 18–23)
  { face: "SIN", ls: "ASIN", rs: "\u2202", alpha: "S" },
  { face: "COS", ls: "ACOS", rs: "\u222B", alpha: "T" },
  { face: "TAN", ls: "ATAN", rs: "\u03A3", alpha: "U" },
  { face: "\u221Ax", ls: "x\xB2", rs: "\u02E3\u221Ay", alpha: "V" },
  { face: "y\u02E3", ls: "10\u02E3", rs: "LOG", alpha: "W" },
  { face: "1/x", ls: "e\u02E3", rs: "LN", alpha: "X" },
  // Row 4 (btn 24–28)
  { face: "ENTER", ls: "EQUATION", rs: "MATRIX", wide: true },
  { face: "+/\u2212", ls: "EDIT", rs: "CMD", alpha: "Y" },
  { face: "EEX", ls: "PURG", rs: "ARG", alpha: "Z" },
  { face: "DEL", ls: "CLEAR" },
  { face: "\u2190", ls: "DROP" },
  // Row 5 (btn 29–33)
  { face: "\u03B1", ls: "USER", rs: "ENTRY", narrow: true },
  { face: "7", rs: "SOLVE" },
  { face: "8", rs: "PLOT" },
  { face: "9", rs: "SYMBOLIC" },
  { face: "\xF7", ls: "( )", rs: "#" },
  // Row 6 (btn 34–38)
  { face: "", arrowDir: "left", narrow: true },
  { face: "4", rs: "TIME" },
  { face: "5", rs: "STAT" },
  { face: "6", rs: "UNITS" },
  { face: "\xD7", ls: "[ ]", rs: "_" },
  // Row 7 (btn 39–43)
  { face: "", arrowDir: "right", narrow: true },
  { face: "1", rs: "I/O" },
  { face: "2", rs: "LIBRARY" },
  { face: "3", rs: "EQ LIB" },
  { face: "\u2212", ls: "\xAB \xBB", rs: "\u201C \u201D" },
  // Row 8 (btn 44–48)
  { face: "ON", ls: "CONT", rs: "OFF", subtitle: "CANCEL", bodyColor: ["#282e3c", "#3c4252"], narrow: true },
  { face: "0", ls: "=", rs: "\u2192" },
  { face: ".", ls: ",", rs: "\u2190" },
  { face: "SPC", ls: "\u03C0", rs: "\u2220" },
  { face: "+", ls: "{ }", rs: "::" }
];
var ROW_SIZES = [6, 6, 6, 6, 5, 5, 5, 5, 5];
function svgEl(tag, attrs) {
  const el = document.createElementNS(SVG_NS, tag);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
  return el;
}
function svgText(x, y, content, fill, anchor, weight, size) {
  const t = svgEl("text", {
    x: String(x),
    y: String(y),
    fill,
    "text-anchor": anchor,
    "font-family": FONT_STACK,
    "font-size": String(size),
    "font-weight": weight
  });
  t.textContent = content;
  return t;
}
var SUPER_MAP = {
  "\xB2": "2",
  "\u02E3": "x"
};
var MATH_RE = /([\u00B2\u02E3\u221Axye])/;
var ITALIC_VARS = /* @__PURE__ */ new Set(["x", "y", "e"]);
function appendStyledText(parent, text, fontSize, fill, dx) {
  const parts = text.split(MATH_RE);
  let first = true;
  let afterRadical = false;
  for (const part of parts) {
    if (!part) continue;
    const span = document.createElementNS(SVG_NS, "tspan");
    span.setAttribute("fill", fill);
    if (first && dx) {
      span.setAttribute("dx", dx);
      first = false;
    }
    if (SUPER_MAP[part]) {
      const ch = SUPER_MAP[part];
      span.setAttribute("font-size", String(fontSize - 2));
      span.setAttribute("baseline-shift", "30%");
      if (ITALIC_VARS.has(ch)) span.setAttribute("font-style", "italic");
      span.textContent = ch;
      afterRadical = false;
    } else if (part === "\u221A") {
      span.textContent = "\u221A";
      afterRadical = true;
    } else if (ITALIC_VARS.has(part)) {
      span.setAttribute("font-style", "italic");
      if (afterRadical) span.setAttribute("text-decoration", "overline");
      span.textContent = part;
      afterRadical = false;
    } else {
      span.textContent = part;
      afterRadical = false;
    }
    first = false;
    parent.appendChild(span);
  }
}
function generateButtons() {
  const container = document.getElementById("buttons");
  if (!container) return;
  const rows = container.querySelectorAll(".btn-row");
  let idx = 0;
  for (let r = 0; r < rows.length; r++) {
    for (let c = 0; c < ROW_SIZES[r]; c++, idx++) {
      rows[r].appendChild(buildButton(BUTTONS[idx], idx, r));
    }
  }
}
var BG_BUTTONS = /* @__PURE__ */ new Set([30, 31, 32, 35, 36, 37, 39, 40, 41, 42]);
function buildButton(btn, idx, row) {
  const isMenu = !!btn.menu;
  const isWide = !!btn.wide;
  const is6 = !isMenu && !isWide && row <= 4;
  const [vw, vh] = isMenu ? [156, 66] : isWide ? [260, 108] : is6 ? [130, 108] : [156, 108];
  const svg = svgEl("svg", {
    viewBox: `0 0 ${vw} ${vh}`,
    class: `btn${isWide ? " btn-wide" : ""}`,
    "data-btn": String(idx)
  });
  if (BG_BUTTONS.has(idx)) {
    svg.appendChild(svgEl("rect", {
      x: "2",
      y: "2",
      width: String(vw - 4),
      height: String(vh - 4),
      rx: "3",
      fill: "#505868"
    }));
  }
  if (isMenu) {
    buildMenuButton(svg, btn);
  } else {
    buildStdButton(svg, btn, idx, vw, row);
  }
  return svg;
}
function buildMenuButton(svg, btn) {
  svg.appendChild(svgEl("rect", {
    x: "26",
    y: "6",
    width: "104",
    height: "54",
    rx: "10",
    fill: "#203040",
    stroke: "#0a0e14",
    "stroke-width": "1"
  }));
  svg.appendChild(svgEl("rect", {
    x: "38",
    y: "12",
    width: "80",
    height: "30",
    rx: "6",
    fill: "#E8ECE4"
  }));
  if (btn.alpha) {
    const al = svgText(130, 54, btn.alpha, "#9a9a9a", "start", "bold", 22);
    al.setAttribute("font-stretch", "condensed");
    svg.appendChild(al);
  }
}
function buildStdButton(svg, btn, idx, vw, row) {
  const bd = btn.wide ? BE : btn.narrow ? BN : row <= 4 ? B6 : B5;
  const { x: bx, y: by, w: bw, h: bh, rx } = bd;
  const isShift = !!btn.arrowDir;
  const [gt, gb] = isShift ? BODY_GRAD : btn.bodyColor || BODY_GRAD;
  const gid = `bg${idx}`;
  const defs = svgEl("defs", {});
  const grad = svgEl("linearGradient", {
    id: gid,
    x1: "0",
    y1: "0",
    x2: "0",
    y2: "1"
  });
  grad.appendChild(svgEl("stop", { offset: "0%", "stop-color": gt }));
  grad.appendChild(svgEl("stop", { offset: "100%", "stop-color": gb }));
  defs.appendChild(grad);
  svg.appendChild(defs);
  addShiftLabels(svg, btn, vw / 2, by);
  svg.appendChild(svgEl("rect", {
    x: String(bx + 2),
    y: String(by + 3),
    width: String(bw),
    height: String(bh),
    rx: String(rx),
    fill: "rgba(0,0,0,0.5)",
    filter: "url(#bshadow)"
  }));
  if (!svg.querySelector("#bshadow")) {
    const sf = svgEl("filter", { id: "bshadow", x: "-15%", y: "-15%", width: "140%", height: "140%" });
    sf.appendChild(svgEl("feGaussianBlur", { stdDeviation: "2.5" }));
    defs.appendChild(sf);
    const hf = svgEl("filter", { id: "bhlite", x: "-10%", y: "-10%", width: "130%", height: "130%" });
    hf.appendChild(svgEl("feGaussianBlur", { stdDeviation: "1.2" }));
    defs.appendChild(hf);
  }
  svg.appendChild(svgEl("rect", {
    x: String(bx),
    y: String(by),
    width: String(bw),
    height: String(bh),
    rx: String(rx),
    fill: `url(#${gid})`,
    stroke: "#0c1018",
    "stroke-width": "1.5"
  }));
  if (isShift) {
    const inset = 7;
    const shiftCol = btn.arrowDir === "left" ? LS_COLOR : RS_COLOR;
    svg.appendChild(svgEl("rect", {
      x: String(bx + inset),
      y: String(by + inset),
      width: String(bw - 2 * inset),
      height: String(bh - 2 * inset),
      rx: String(Math.max(rx - inset, 6)),
      fill: shiftCol
    }));
  }
  svg.appendChild(svgEl("rect", {
    x: String(bx + 3),
    y: String(by + 1),
    width: String(bw - 6),
    height: String(bh / 2),
    rx: String(rx - 2),
    fill: "rgba(255,255,255,0.09)",
    filter: "url(#bhlite)"
  }));
  svg.appendChild(svgEl("line", {
    x1: String(bx + rx),
    y1: String(by + bh - 1.5),
    x2: String(bx + bw - rx),
    y2: String(by + bh - 1.5),
    stroke: "rgba(0,0,0,0.3)",
    "stroke-width": "2.5",
    "stroke-linecap": "round",
    filter: "url(#bhlite)"
  }));
  svg.appendChild(svgEl("line", {
    x1: String(bx + rx * 0.7),
    y1: String(by + 2.5),
    x2: String(bx + bw - rx * 0.7),
    y2: String(by + 2.5),
    stroke: "rgba(255,255,255,0.18)",
    "stroke-width": "1.5",
    "stroke-linecap": "round",
    filter: "url(#bhlite)"
  }));
  const cx = bx + bw / 2;
  const cy = by + bh / 2;
  if (btn.arrowDir) {
    const agid = `ag${idx}`;
    const agrad = svgEl("linearGradient", {
      id: agid,
      x1: "0",
      y1: String(by),
      x2: "0",
      y2: String(by + bh),
      gradientUnits: "userSpaceOnUse"
    });
    agrad.appendChild(svgEl("stop", { offset: "0%", "stop-color": BODY_GRAD[0] }));
    agrad.appendChild(svgEl("stop", { offset: "100%", "stop-color": BODY_GRAD[1] }));
    defs.appendChild(agrad);
    drawShiftArrow(svg, btn.arrowDir, bx, by, bw, bh, `url(#${agid})`);
  } else if (btn.face) {
    const fs = faceFontSize(btn.face, bw, !!btn.wide);
    if (btn.face === "\u2190") {
      const x1 = bx + bw * 0.25, x2 = bx + bw * 0.5, x3 = bx + bw * 0.75;
      const y1 = by + bh * 0.25, y2 = by + bh * 0.375, y3 = by + bh * 0.5;
      const y4 = by + bh * 0.625, y5 = by + bh * 0.75;
      svg.appendChild(svgEl("path", {
        d: `M ${x1} ${y3} L ${x2} ${y1} V ${y2} H ${x3} V ${y4} H ${x2} V ${y5} Z`,
        fill: "#fff"
      }));
    } else if (btn.face.includes("\u221A")) {
      const radicand = btn.face.split("\u221A")[1] || "";
      drawRadical(svg, cx, cy, fs, radicand);
    } else if (MATH_RE.test(btn.face)) {
      const ft = svgEl("text", {
        x: String(cx),
        y: String(cy),
        "text-anchor": "middle",
        "font-family": FONT_STACK,
        "font-size": String(fs),
        "font-weight": "bold",
        "dominant-baseline": "central"
      });
      appendStyledText(ft, btn.face, fs, "#fff");
      svg.appendChild(ft);
    } else {
      const wt = OPERATORS.has(btn.face) ? "normal" : "bold";
      const ft = svgText(cx, cy, btn.face, "#fff", "middle", wt, fs);
      ft.setAttribute("dominant-baseline", "central");
      svg.appendChild(ft);
    }
    if (btn.subtitle) {
      svg.appendChild(svgText(cx, by + bh + 16, btn.subtitle, "#888", "middle", "bold", 18));
    }
  }
  if (btn.alpha) {
    const al = svgText(bx + bw + 2, by + bh + 4, btn.alpha, "#9a9a9a", "start", "bold", 22);
    al.setAttribute("font-stretch", "condensed");
    svg.appendChild(al);
  }
}
var OPERATORS = /* @__PURE__ */ new Set(["+", "\u2212", "-", "\xD7", "\xF7"]);
function faceFontSize(face, bodyW, wide) {
  const len = face.length;
  const big = bodyW >= 120;
  if (len === 1 && OPERATORS.has(face)) return big ? 72 : 65;
  if (len === 1) return big ? 46 : 42;
  if (len === 2) return big ? 40 : 36;
  if (len === 3) return big ? 38 : 34;
  if (len <= 5) return wide ? 34 : big ? 30 : 26;
  return wide ? 28 : big ? 24 : 20;
}
function addShiftLabels(svg, btn, cx, bodyY) {
  if (!btn.ls && !btn.rs) return;
  const labelY = bodyY - 6;
  const text = svgEl("text", {
    x: String(cx),
    y: String(labelY),
    "text-anchor": "middle",
    "font-family": FONT_STACK,
    "font-size": "26",
    "font-weight": "bold",
    "font-stretch": "condensed"
  });
  if (btn.ls && btn.rs) {
    appendStyledText(text, btn.ls, 26, LS_COLOR);
    appendStyledText(text, btn.rs, 26, RS_COLOR, "12");
  } else if (btn.ls) {
    appendStyledText(text, btn.ls, 26, LS_COLOR);
  } else {
    appendStyledText(text, btn.rs, 26, RS_COLOR);
  }
  svg.appendChild(text);
}
function drawRadical(svg, cx, cy, fontSize, radicand) {
  const h = fontSize * 0.9;
  const radW = fontSize * 0.5;
  const barW = fontSize * 0.55;
  const totalW = radW + barW;
  const startX = cx - totalW / 2;
  const topY = cy - h * 0.45;
  const botY = cy + h * 0.15;
  const midY = cy + h * 0.05;
  svg.appendChild(svgEl("path", {
    d: `M ${startX} ${midY} L ${startX + radW * 0.3} ${botY} L ${startX + radW} ${topY} H ${startX + totalW}`,
    fill: "none",
    stroke: "#fff",
    "stroke-width": "2.5",
    "stroke-linecap": "round",
    "stroke-linejoin": "round"
  }));
  const t = svgEl("text", {
    x: String(startX + radW + barW / 2),
    y: String(cy),
    "text-anchor": "middle",
    "dominant-baseline": "central",
    "font-family": FONT_STACK,
    "font-size": String(fontSize),
    "font-weight": "bold",
    fill: "#fff"
  });
  if (ITALIC_VARS.has(radicand)) t.setAttribute("font-style", "italic");
  t.textContent = radicand;
  svg.appendChild(t);
}
function drawShiftArrow(svg, dir, bx, by, bw, bh, strokeRef) {
  const cy = by + bh / 2;
  const bot = by + bh - 5;
  const r = 3;
  const tail = dir === "left" ? bx + bw * 0.75 : bx + bw * 0.25;
  const tip = dir === "left" ? bx + bw * 0.25 : bx + bw * 0.75;
  const attrs = {
    fill: "none",
    stroke: strokeRef,
    "stroke-width": "10",
    "stroke-linecap": "round",
    "stroke-linejoin": "round"
  };
  if (dir === "left") {
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tail} ${bot} V ${cy + r} Q ${tail} ${cy},${tail - r} ${cy} H ${tip}`
    }));
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tip + 16} ${cy - 15} L ${tip} ${cy} L ${tip + 16} ${cy + 15}`
    }));
  } else {
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tail} ${bot} V ${cy + r} Q ${tail} ${cy},${tail + r} ${cy} H ${tip}`
    }));
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tip - 16} ${cy - 15} L ${tip} ${cy} L ${tip - 16} ${cy + 15}`
    }));
  }
}
var AUDIO_POLL_MS = 20;
var audioCtx = null;
var oscillator = null;
var gainNode = null;
var playing = false;
function initAudioOnGesture() {
  if (audioCtx) return;
  try {
    const AudioCtor = window.AudioContext ?? window.webkitAudioContext;
    if (!AudioCtor) {
      console.warn("hp48: no AudioContext support");
      return;
    }
    audioCtx = new AudioCtor();
    void audioCtx.resume();
    gainNode = audioCtx.createGain();
    gainNode.gain.value = 0;
    gainNode.connect(audioCtx.destination);
    oscillator = audioCtx.createOscillator();
    oscillator.type = "square";
    oscillator.frequency.value = 440;
    oscillator.connect(gainNode);
    oscillator.start();
    window.setInterval(pollSpeaker, AUDIO_POLL_MS);
  } catch (e) {
    console.warn("hp48: audio init failed", e);
  }
}
function pollSpeaker() {
  const freq = hp48.speaker_frequency();
  if (freq > 0) {
    oscillator.frequency.value = freq;
    if (!playing) {
      gainNode.gain.value = 0.15;
      playing = true;
    }
  } else if (playing) {
    gainNode.gain.value = 0;
    playing = false;
  }
}
function setupAudio() {
  const handler = () => {
    initAudioOnGesture();
    document.removeEventListener("mousedown", handler);
    document.removeEventListener("touchstart", handler);
    document.removeEventListener("keydown", handler);
  };
  document.addEventListener("mousedown", handler);
  document.addEventListener("touchstart", handler);
  document.addEventListener("keydown", handler);
}
async function saveToIDB() {
  try {
    const state = hp48.save_state();
    const ram = hp48.save_ram();
    await dbPut("state", state);
    await dbPut("ram", ram);
    console.log(`[hp48] saved to IDB: state=${state.byteLength}B, ram=${ram.byteLength}B`);
  } catch (e) {
    console.warn("[hp48] save failed", e);
  }
}
function startAutoSave() {
  setInterval(saveToIDB, AUTO_SAVE_INTERVAL_MS);
  window.addEventListener("beforeunload", () => {
    void saveToIDB();
  });
}
function startEmulationLoop() {
  let lastTime = performance.now();
  function frame(now) {
    const elapsed = now - lastTime;
    lastTime = now;
    hp48.run_frame(elapsed, now / 1e3);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
}
async function fetchAsset(name) {
  const resp = await fetch(`./assets/${name}`);
  if (!resp.ok) throw new Error(`Failed to fetch assets/${name}: ${resp.status}`);
  return new Uint8Array(await resp.arrayBuffer());
}
async function main() {
  const wasm2 = await __wbg_init();
  wasmMemory = wasm2.memory;
  const rom = await fetchAsset("rom");
  let ram = await dbGet("ram") ?? null;
  if (ram) {
    console.log(`[hp48] loaded RAM from IDB: ${ram.byteLength} bytes`);
  } else {
    try {
      ram = await fetchAsset("ram");
      console.log(`[hp48] loaded RAM from assets: ${ram.byteLength} bytes`);
    } catch {
      console.log("[hp48] no RAM found, starting fresh");
    }
  }
  let state = await dbGet("state") ?? null;
  if (state) {
    console.log(`[hp48] loaded state from IDB: ${state.byteLength} bytes`);
  } else {
    try {
      state = await fetchAsset("hp48");
      console.log(`[hp48] loaded state from assets: ${state.byteLength} bytes`);
    } catch {
      console.log("[hp48] no state found, starting fresh");
    }
  }
  hp48 = new Hp48(rom, ram, state);
  const localEpochSecs = Date.now() / 1e3 - (/* @__PURE__ */ new Date()).getTimezoneOffset() * 60;
  hp48.start(performance.now() / 1e3, localEpochSecs);
  generateButtons();
  startDisplayLoop();
  setupButtonInput();
  setupKeyboardInput();
  setupAudio();
  showCalculator();
  startEmulationLoop();
  startAutoSave();
  console.log("HP-48 Rust WASM emulator initialized");
}
main().catch((e) => console.error("hp48_rust: init failed", e));
