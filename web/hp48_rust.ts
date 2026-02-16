// hp48_rust.ts — TypeScript bridge for the Rust/wasm-bindgen WASM path.
// Independent from hp48.ts (Emscripten/C path). Same SVG buttons, keyboard,
// audio — different emulator interface.

import init, { Hp48 } from "../pkg/rust48.js";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DISPLAY_WIDTH = 131;
const DISPLAY_HEIGHT = 64;
const DISPLAY_BYTES = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;
const AUTO_SAVE_INTERVAL_MS = 30_000;
const DB_NAME = "hp48_rust";
const DB_STORE = "files";

// ---------------------------------------------------------------------------
// IndexedDB persistence
// ---------------------------------------------------------------------------

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(DB_STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function dbGet(key: string): Promise<Uint8Array | undefined> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(DB_STORE, "readonly");
    const req = tx.objectStore(DB_STORE).get(key);
    req.onsuccess = () => resolve(req.result as Uint8Array | undefined);
    req.onerror = () => reject(req.error);
  });
}

async function dbPut(key: string, value: Uint8Array): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(DB_STORE, "readwrite");
    tx.objectStore(DB_STORE).put(value, key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

// ---------------------------------------------------------------------------
// Button ID → Saturn keycode mapping (from x48_web.c buttons[].code)
// ---------------------------------------------------------------------------

const BUTTON_KEYCODES: number[] = [
  // Row 0: menu keys A–F
  0x14, 0x84, 0x83, 0x82, 0x81, 0x80,
  // Row 1: MTH PRG CST VAR UP NXT
  0x24, 0x74, 0x73, 0x72, 0x71, 0x70,
  // Row 2: ' STO EVAL LEFT DOWN RIGHT
  0x04, 0x64, 0x63, 0x62, 0x61, 0x60,
  // Row 3: SIN COS TAN SQRT POWER INV
  0x34, 0x54, 0x53, 0x52, 0x51, 0x50,
  // Row 4: ENTER NEG EEX DEL BS
  0x44, 0x43, 0x42, 0x41, 0x40,
  // Row 5: ALPHA 7 8 9 DIV
  0x35, 0x33, 0x32, 0x31, 0x30,
  // Row 6: SHL 4 5 6 MUL
  0x25, 0x23, 0x22, 0x21, 0x20,
  // Row 7: SHR 1 2 3 MINUS
  0x15, 0x13, 0x12, 0x11, 0x10,
  // Row 8: ON 0 . SPC PLUS
  0x8000, 0x03, 0x02, 0x01, 0x00,
];

/** Convert button ID (0–48) + press/release to Saturn key event code. */
function buttonToKeyEvent(btnId: number, press: boolean): number {
  const keycode = BUTTON_KEYCODES[btnId];
  return press ? keycode : (keycode | 0x80000000) >>> 0;
}

// ---------------------------------------------------------------------------
// Keyboard mapping: KeyboardEvent.key -> button ID (0-48)
// ---------------------------------------------------------------------------

const KEY_MAP: Record<string, number> = {
  "0": 45, "1": 40, "2": 41, "3": 42, "4": 35,
  "5": 36, "6": 37, "7": 30, "8": 31, "9": 32,
  "Enter": 24, "Backspace": 28, "Delete": 27,
  ".": 46, "+": 48, "-": 43, "*": 38, "/": 33,
  " ": 47, "Escape": 44,
  "ArrowUp": 10, "ArrowDown": 16, "ArrowLeft": 15, "ArrowRight": 17,
  "'": 12,
};

const ALPHA_MAP: Record<string, number> = {
  a: 0,  b: 1,  c: 2,  d: 3,  e: 4,  f: 5,
  g: 6,  h: 7,  i: 8,  j: 9,  k: 10, l: 11,
  m: 12, n: 13, o: 14, p: 15, q: 16, r: 17,
  s: 18, t: 19, u: 20, v: 21, w: 22, x: 23,
  y: 25, z: 26,
};

const ALPHA_BTN = 29;
const SHL_BTN = 34;
const SHR_BTN = 39;

const SHIFT_MAP: Record<string, [number, number]> = {
  "(": [SHL_BTN, 33], ")": [SHL_BTN, 33],
  "[": [SHL_BTN, 38], "]": [SHL_BTN, 38],
  "{": [SHL_BTN, 48], "}": [SHL_BTN, 48],
  "<": [SHL_BTN, 43], ">": [SHL_BTN, 43],
  "#": [SHR_BTN, 33], "_": [SHR_BTN, 38],
  "\"": [SHR_BTN, 43], ":": [SHR_BTN, 48],
  ",": [SHL_BTN, 46],
};

const pressedKeys = new Set<string>();

// ---------------------------------------------------------------------------
// Globals
// ---------------------------------------------------------------------------

let hp48: Hp48;
let wasmMemory: WebAssembly.Memory;

// ---------------------------------------------------------------------------
// Display rendering
// ---------------------------------------------------------------------------

// -----------------------------------------------------------------------
// Annunciator XBM bitmaps — from annunc.h (15×12, LSB-first, 2 bytes/row)
// -----------------------------------------------------------------------

const ANN_WIDTH = 15;
const ANN_HEIGHT = 12;
const ANN_CANVAS_W = 262; // 2× LCD resolution for crisp annunciator icons
const ANN_CANVAS_H = 12;

// Pixel colors matching the LCD
const PIX_ON  = [0x10, 0x20, 0x10, 0xFF];
const PIX_OFF = [0xBC, 0xC4, 0xA5, 0xFF];

const ANN_DEFS: { bit: number; x: number; bits: number[] }[] = [
  { bit: 0x81, x: 16, bits: [  // left shift
    0xfe,0x3f, 0xff,0x7f, 0x9f,0x7f, 0xcf,0x7f, 0xe7,0x7f, 0x03,0x78,
    0x03,0x70, 0xe7,0x73, 0xcf,0x73, 0x9f,0x73, 0xff,0x73, 0xfe,0x33] },
  { bit: 0x82, x: 61, bits: [  // right shift
    0xfe,0x3f, 0xff,0x7f, 0xff,0x7c, 0xff,0x79, 0xff,0x73, 0x0f,0x60,
    0x07,0x60, 0xe7,0x73, 0xe7,0x79, 0xe7,0x7c, 0xe7,0x7f, 0xe6,0x3f] },
  { bit: 0x84, x: 106, bits: [ // alpha
    0xe0,0x03, 0x18,0x44, 0x0c,0x4c, 0x06,0x2c, 0x07,0x2c, 0x07,0x1c,
    0x07,0x0c, 0x07,0x0c, 0x07,0x0e, 0x0e,0x4d, 0xf8,0x38, 0x00,0x00] },
  { bit: 0x88, x: 151, bits: [ // battery
    0x04,0x10, 0x02,0x20, 0x12,0x24, 0x09,0x48, 0xc9,0x49, 0xc9,0x49,
    0xc9,0x49, 0x09,0x48, 0x12,0x24, 0x02,0x20, 0x04,0x10, 0x00,0x00] },
  { bit: 0x90, x: 196, bits: [ // busy
    0xfc,0x1f, 0x08,0x08, 0x08,0x08, 0xf0,0x07, 0xe0,0x03, 0xc0,0x01,
    0x40,0x01, 0x20,0x02, 0x10,0x04, 0xc8,0x09, 0xe8,0x0b, 0xfc,0x1f] },
  { bit: 0xa0, x: 241, bits: [ // IO
    0x0c,0x00, 0x1e,0x00, 0x33,0x0c, 0x61,0x18, 0xcc,0x30, 0xfe,0x7f,
    0xfe,0x7f, 0xcc,0x30, 0x61,0x18, 0x33,0x0c, 0x1e,0x00, 0x0c,0x00] },
];

// Pre-decode XBM bitmaps into boolean pixel arrays
const annBitmaps: boolean[][][] = ANN_DEFS.map(def => {
  const rows: boolean[][] = [];
  for (let y = 0; y < ANN_HEIGHT; y++) {
    const row: boolean[] = [];
    const b0 = def.bits[y * 2];
    const b1 = def.bits[y * 2 + 1];
    const rowBits = b0 | (b1 << 8);
    for (let x = 0; x < ANN_WIDTH; x++) {
      row.push(((rowBits >> x) & 1) !== 0);
    }
    rows.push(row);
  }
  return rows;
});

function startDisplayLoop(): void {
  const canvas = document.getElementById("lcd") as HTMLCanvasElement | null;
  if (!canvas) { console.error("hp48: #lcd canvas not found"); return; }

  canvas.width = DISPLAY_WIDTH;
  canvas.height = DISPLAY_HEIGHT;

  const ctx = canvas.getContext("2d")!;
  const imageData = ctx.createImageData(DISPLAY_WIDTH, DISPLAY_HEIGHT);

  // Annunciator canvas — 2× LCD resolution for crisp icons
  const annCanvas = document.getElementById("annunciators") as HTMLCanvasElement | null;
  let annCtx: CanvasRenderingContext2D | null = null;
  let annImageData: ImageData | null = null;
  if (annCanvas) {
    annCanvas.width = ANN_CANVAS_W;
    annCanvas.height = ANN_CANVAS_H;
    // Force CSS sizing — WebView2 ignores stylesheet width:100% on canvas
    annCanvas.style.width = "100%";
    annCanvas.style.height = "auto";
    annCtx = annCanvas.getContext("2d")!;
    annImageData = annCtx.createImageData(ANN_CANVAS_W, ANN_CANVAS_H);
    // Fill with LCD off color
    for (let i = 0; i < ANN_CANVAS_W * ANN_CANVAS_H; i++) {
      annImageData.data.set(PIX_OFF, i * 4);
    }
    annCtx.putImageData(annImageData, 0, 0);
  }
  let lastAnnunc = -1;

  function frame(): void {
    if (hp48.is_display_dirty()) {
      const ptr = hp48.display_buffer_ptr();
      const src = new Uint8Array(wasmMemory.buffer, ptr, DISPLAY_BYTES);
      imageData.data.set(src);
      hp48.clear_display_dirty();
      ctx.putImageData(imageData, 0, 0);
    }

    // Update annunciator bitmaps
    const annunc = hp48.annunciator_state();
    if (annunc !== lastAnnunc && annCtx && annImageData) {
      lastAnnunc = annunc;
      const d = annImageData.data;
      // Clear to off color
      for (let i = 0; i < ANN_CANVAS_W * ANN_CANVAS_H; i++) {
        d[i * 4]     = PIX_OFF[0];
        d[i * 4 + 1] = PIX_OFF[1];
        d[i * 4 + 2] = PIX_OFF[2];
        d[i * 4 + 3] = PIX_OFF[3];
      }
      // Draw active annunciator bitmaps
      for (let a = 0; a < ANN_DEFS.length; a++) {
        if ((annunc & ANN_DEFS[a].bit) !== ANN_DEFS[a].bit) continue;
        const bx = ANN_DEFS[a].x;
        const bmp = annBitmaps[a];
        for (let y = 0; y < ANN_HEIGHT; y++) {
          for (let x = 0; x < ANN_WIDTH; x++) {
            if (bmp[y][x]) {
              const idx = ((y) * ANN_CANVAS_W + bx + x) * 4;
              d[idx]     = PIX_ON[0];
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

// ---------------------------------------------------------------------------
// Button (pointer) input
// ---------------------------------------------------------------------------

function setupButtonInput(): void {
  const buttons = document.querySelectorAll<HTMLElement>("[data-btn]");

  buttons.forEach((el) => {
    const btnId = parseInt(el.dataset.btn!, 10);
    if (isNaN(btnId)) return;

    function press(): void {
      el.classList.add("pressed");
      hp48.push_key_event(buttonToKeyEvent(btnId, true));
    }
    function release(): void {
      el.classList.remove("pressed");
      hp48.push_key_event(buttonToKeyEvent(btnId, false));
    }

    el.addEventListener("mousedown", (e) => { e.preventDefault(); press(); });
    el.addEventListener("mouseup", () => { release(); });
    el.addEventListener("mouseleave", () => {
      if (el.classList.contains("pressed")) release();
    });

    el.addEventListener("touchstart", (e) => { e.preventDefault(); press(); }, { passive: false });
    el.addEventListener("touchend", (e) => { e.preventDefault(); release(); }, { passive: false });
    el.addEventListener("touchcancel", (e) => { e.preventDefault(); release(); }, { passive: false });
  });

  document.getElementById("buttons")
    ?.addEventListener("contextmenu", (e) => e.preventDefault());
}

// ---------------------------------------------------------------------------
// Keyboard input
// ---------------------------------------------------------------------------

function pushKeySequence(events: number[], delay = 20): void {
  events.forEach((code, i) => {
    if (i === 0) {
      hp48.push_key_event(code);
    } else {
      setTimeout(() => hp48.push_key_event(code), i * delay);
    }
  });
}

function setupKeyboardInput(): void {
  document.addEventListener("keydown", (e) => {
    const btnId = KEY_MAP[e.key];
    if (btnId !== undefined) {
      if (pressedKeys.has(e.key)) return;
      pressedKeys.add(e.key);
      e.preventDefault();
      hp48.push_key_event(buttonToKeyEvent(btnId, true));
      return;
    }

    const shiftCombo = SHIFT_MAP[e.key];
    if (shiftCombo !== undefined) {
      if (e.repeat) return;
      e.preventDefault();
      const [shiftBtn, targetBtn] = shiftCombo;
      pushKeySequence([
        buttonToKeyEvent(shiftBtn, true),
        buttonToKeyEvent(targetBtn, true),
        buttonToKeyEvent(targetBtn, false),
        buttonToKeyEvent(shiftBtn, false),
      ]);
      return;
    }

    const alphaBtn = ALPHA_MAP[e.key.toLowerCase()];
    if (alphaBtn !== undefined) {
      if (e.repeat || e.ctrlKey || e.metaKey || e.altKey) return;
      e.preventDefault();
      pushKeySequence([
        buttonToKeyEvent(ALPHA_BTN, true),
        buttonToKeyEvent(alphaBtn, true),
        buttonToKeyEvent(alphaBtn, false),
        buttonToKeyEvent(ALPHA_BTN, false),
      ]);
      return;
    }
  });

  document.addEventListener("keyup", (e) => {
    const btnId = KEY_MAP[e.key];
    if (btnId === undefined) return;
    pressedKeys.delete(e.key);
    e.preventDefault();
    hp48.push_key_event(buttonToKeyEvent(btnId, false));
  });
}

// ---------------------------------------------------------------------------
// Loading state
// ---------------------------------------------------------------------------

function showCalculator(): void {
  const loading = document.getElementById("loading");
  if (loading) loading.style.display = "none";
}

// ---------------------------------------------------------------------------
// SVG Button Generation
// (Identical to hp48.ts — pure DOM code, no emulator dependency)
// ---------------------------------------------------------------------------

interface ButtonDef {
  face: string;
  ls?: string;
  rs?: string;
  alpha?: string;
  bodyColor?: [string, string];
  wide?: boolean;
  narrow?: boolean;
  menu?: boolean;
  subtitle?: string;
  arrowDir?: "left" | "right";
}

const SVG_NS = "http://www.w3.org/2000/svg";
const FONT_STACK = "'Helvetica Neue',Arial,Helvetica,sans-serif";
const LS_COLOR = "#8B6BA0";
const RS_COLOR = "#5AABB8";

const BODY_GRAD: [string, string] = ["#141c2c", "#2c3448"];

const B6 = { x: 22, y: 33, w: 96,  h: 69, rx: 20 };
const B5 = { x: 18, y: 33, w: 126, h: 69, rx: 16 };
const BN = { x: 22, y: 33, w: 96,  h: 69, rx: 20 };
const BE = { x: 22, y: 33, w: 226, h: 69, rx: 20 };

const BUTTONS: ButtonDef[] = [
  // Row 0: Menu keys (btn 0–5) with alpha labels A–F
  { face: "", menu: true, alpha: "A" }, { face: "", menu: true, alpha: "B" },
  { face: "", menu: true, alpha: "C" }, { face: "", menu: true, alpha: "D" },
  { face: "", menu: true, alpha: "E" }, { face: "", menu: true, alpha: "F" },
  // Row 1 (btn 6–11)
  { face: "MTH",  ls: "RAD",  rs: "POLAR",  alpha: "G" },
  { face: "PRG",               rs: "CHARS",  alpha: "H" },
  { face: "CST",               rs: "MODES",  alpha: "I" },
  { face: "VAR",               rs: "MEMORY", alpha: "J" },
  { face: "\u25B2",            rs: "STACK",  alpha: "K" },
  { face: "NXT",  ls: "PREV", rs: "MENU",   alpha: "L" },
  // Row 2 (btn 12–17)
  { face: "'",     ls: "UP",       rs: "HOME",    alpha: "M" },
  { face: "STO",   ls: "DEF",      rs: "RCL",     alpha: "N" },
  { face: "EVAL",  ls: "\u2192NUM", rs: "UNDO",   alpha: "O" },
  { face: "\u25C0", ls: "PICTURE",                  alpha: "P" },
  { face: "\u25BC", ls: "VIEW",                     alpha: "Q" },
  { face: "\u25B6", ls: "SWAP",                     alpha: "R" },
  // Row 3 (btn 18–23)
  { face: "SIN",  ls: "ASIN", rs: "\u2202",    alpha: "S" },
  { face: "COS",  ls: "ACOS", rs: "\u222B",    alpha: "T" },
  { face: "TAN",  ls: "ATAN", rs: "\u03A3",    alpha: "U" },
  { face: "\u221Ax",  ls: "x\u00B2",  rs: "\u02E3\u221Ay", alpha: "V" },
  { face: "y\u02E3",  ls: "10\u02E3", rs: "LOG",  alpha: "W" },
  { face: "1/x",  ls: "e\u02E3",     rs: "LN",   alpha: "X" },
  // Row 4 (btn 24–28)
  { face: "ENTER", ls: "EQUATION", rs: "MATRIX", wide: true },
  { face: "+/\u2212", ls: "EDIT",  rs: "CMD",  alpha: "Y" },
  { face: "EEX",   ls: "PURG",     rs: "ARG",  alpha: "Z" },
  { face: "DEL",   ls: "CLEAR" },
  { face: "\u2190", ls: "DROP" },
  // Row 5 (btn 29–33)
  { face: "\u03B1", ls: "USER", rs: "ENTRY", narrow: true },
  { face: "7",                  rs: "SOLVE" },
  { face: "8",                  rs: "PLOT" },
  { face: "9",                  rs: "SYMBOLIC" },
  { face: "\u00F7", ls: "( )", rs: "#" },
  // Row 6 (btn 34–38)
  { face: "", arrowDir: "left", narrow: true },
  { face: "4",                  rs: "TIME" },
  { face: "5",                  rs: "STAT" },
  { face: "6",                  rs: "UNITS" },
  { face: "\u00D7", ls: "[ ]", rs: "_" },
  // Row 7 (btn 39–43)
  { face: "", arrowDir: "right", narrow: true },
  { face: "1",                   rs: "I/O" },
  { face: "2",                   rs: "LIBRARY" },
  { face: "3",                   rs: "EQ LIB" },
  { face: "\u2212", ls: "\u00AB \u00BB", rs: '\u201C \u201D' },
  // Row 8 (btn 44–48)
  { face: "ON", ls: "CONT", rs: "OFF", subtitle: "CANCEL", bodyColor: ["#282e3c", "#3c4252"], narrow: true },
  { face: "0",  ls: "=",    rs: "\u2192" },
  { face: ".",  ls: ",",    rs: "\u2190" },
  { face: "SPC", ls: "\u03C0", rs: "\u2220" },
  { face: "+",  ls: "{ }",  rs: "::" },
];

const ROW_SIZES = [6, 6, 6, 6, 5, 5, 5, 5, 5];

function svgEl(tag: string, attrs: Record<string, string>): SVGElement {
  const el = document.createElementNS(SVG_NS, tag);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
  return el;
}

function svgText(
  x: number, y: number, content: string,
  fill: string, anchor: string, weight: string, size: number,
): SVGTextElement {
  const t = svgEl("text", {
    x: String(x), y: String(y), fill, "text-anchor": anchor,
    "font-family": FONT_STACK, "font-size": String(size), "font-weight": weight,
  }) as SVGTextElement;
  t.textContent = content;
  return t;
}

const SUPER_MAP: Record<string, string> = {
  "\u00B2": "2",
  "\u02E3": "x",
};
const MATH_RE = /([\u00B2\u02E3\u221Axye])/;
const ITALIC_VARS = new Set(["x", "y", "e"]);

function appendStyledText(
  parent: SVGElement, text: string, fontSize: number, fill: string, dx?: string,
): void {
  const parts = text.split(MATH_RE);
  let first = true;
  let afterRadical = false;
  for (const part of parts) {
    if (!part) continue;
    const span = document.createElementNS(SVG_NS, "tspan");
    span.setAttribute("fill", fill);
    if (first && dx) { span.setAttribute("dx", dx); first = false; }
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

function generateButtons(): void {
  const container = document.getElementById("buttons");
  if (!container) return;
  const rows = container.querySelectorAll<HTMLElement>(".btn-row");
  let idx = 0;
  for (let r = 0; r < rows.length; r++) {
    for (let c = 0; c < ROW_SIZES[r]; c++, idx++) {
      rows[r].appendChild(buildButton(BUTTONS[idx], idx, r));
    }
  }
}

const BG_BUTTONS = new Set([30,31,32, 35,36,37, 39,40,41,42]);

function buildButton(btn: ButtonDef, idx: number, row: number): SVGSVGElement {
  const isMenu = !!btn.menu;
  const isWide = !!btn.wide;
  const is6 = !isMenu && !isWide && row <= 4;
  const [vw, vh] = isMenu ? [156, 66] : isWide ? [260, 108] : is6 ? [130, 108] : [156, 108];

  const svg = svgEl("svg", {
    viewBox: `0 0 ${vw} ${vh}`,
    class: `btn${isWide ? " btn-wide" : ""}`,
    "data-btn": String(idx),
  }) as SVGSVGElement;

  if (BG_BUTTONS.has(idx)) {
    svg.appendChild(svgEl("rect", {
      x: "2", y: "2", width: String(vw - 4), height: String(vh - 4),
      rx: "3", fill: "#505868",
    }));
  }

  if (isMenu) {
    buildMenuButton(svg, btn);
  } else {
    buildStdButton(svg, btn, idx, vw, row);
  }
  return svg;
}

function buildMenuButton(svg: SVGSVGElement, btn: ButtonDef): void {
  svg.appendChild(svgEl("rect", {
    x: "26", y: "6", width: "104", height: "54",
    rx: "10", fill: "#203040", stroke: "#0a0e14", "stroke-width": "1",
  }));
  svg.appendChild(svgEl("rect", {
    x: "38", y: "12", width: "80", height: "30",
    rx: "6", fill: "#E8ECE4",
  }));
  if (btn.alpha) {
    const al = svgText(130, 54, btn.alpha, "#9a9a9a", "start", "bold", 22);
    al.setAttribute("font-stretch", "condensed");
    svg.appendChild(al);
  }
}

function buildStdButton(
  svg: SVGSVGElement, btn: ButtonDef, idx: number, vw: number, row: number,
): void {
  const bd = btn.wide ? BE : btn.narrow ? BN : row <= 4 ? B6 : B5;
  const { x: bx, y: by, w: bw, h: bh, rx } = bd;
  const isShift = !!btn.arrowDir;
  const [gt, gb] = isShift ? BODY_GRAD : (btn.bodyColor || BODY_GRAD);
  const gid = `bg${idx}`;

  const defs = svgEl("defs", {});
  const grad = svgEl("linearGradient", {
    id: gid, x1: "0", y1: "0", x2: "0", y2: "1",
  });
  grad.appendChild(svgEl("stop", { offset: "0%", "stop-color": gt }));
  grad.appendChild(svgEl("stop", { offset: "100%", "stop-color": gb }));
  defs.appendChild(grad);
  svg.appendChild(defs);

  addShiftLabels(svg, btn, vw / 2, by);

  svg.appendChild(svgEl("rect", {
    x: String(bx + 2), y: String(by + 3), width: String(bw), height: String(bh),
    rx: String(rx), fill: "rgba(0,0,0,0.5)",
    filter: "url(#bshadow)",
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
    x: String(bx), y: String(by), width: String(bw), height: String(bh),
    rx: String(rx), fill: `url(#${gid})`, stroke: "#0c1018", "stroke-width": "1.5",
  }));

  if (isShift) {
    const inset = 7;
    const shiftCol = btn.arrowDir === "left" ? LS_COLOR : RS_COLOR;
    svg.appendChild(svgEl("rect", {
      x: String(bx + inset), y: String(by + inset),
      width: String(bw - 2 * inset), height: String(bh - 2 * inset),
      rx: String(Math.max(rx - inset, 6)), fill: shiftCol,
    }));
  }

  svg.appendChild(svgEl("rect", {
    x: String(bx + 3), y: String(by + 1), width: String(bw - 6), height: String(bh / 2),
    rx: String(rx - 2), fill: "rgba(255,255,255,0.09)",
    filter: "url(#bhlite)",
  }));

  svg.appendChild(svgEl("line", {
    x1: String(bx + rx), y1: String(by + bh - 1.5),
    x2: String(bx + bw - rx), y2: String(by + bh - 1.5),
    stroke: "rgba(0,0,0,0.3)", "stroke-width": "2.5", "stroke-linecap": "round",
    filter: "url(#bhlite)",
  }));

  svg.appendChild(svgEl("line", {
    x1: String(bx + rx * 0.7), y1: String(by + 2.5),
    x2: String(bx + bw - rx * 0.7), y2: String(by + 2.5),
    stroke: "rgba(255,255,255,0.18)", "stroke-width": "1.5", "stroke-linecap": "round",
    filter: "url(#bhlite)",
  }));

  const cx = bx + bw / 2;
  const cy = by + bh / 2;

  if (btn.arrowDir) {
    const agid = `ag${idx}`;
    const agrad = svgEl("linearGradient", {
      id: agid, x1: "0", y1: String(by), x2: "0", y2: String(by + bh),
      gradientUnits: "userSpaceOnUse",
    });
    agrad.appendChild(svgEl("stop", { offset: "0%", "stop-color": BODY_GRAD[0] }));
    agrad.appendChild(svgEl("stop", { offset: "100%", "stop-color": BODY_GRAD[1] }));
    defs.appendChild(agrad);
    drawShiftArrow(svg, btn.arrowDir, bx, by, bw, bh, `url(#${agid})`);
  } else if (btn.face) {
    const fs = faceFontSize(btn.face, bw, !!btn.wide);
    if (btn.face === "\u2190") {
      const x1 = bx + bw * 0.25, x2 = bx + bw * 0.50, x3 = bx + bw * 0.75;
      const y1 = by + bh * 0.25, y2 = by + bh * 0.375, y3 = by + bh * 0.5;
      const y4 = by + bh * 0.625, y5 = by + bh * 0.75;
      svg.appendChild(svgEl("path", {
        d: `M ${x1} ${y3} L ${x2} ${y1} V ${y2} H ${x3} V ${y4} H ${x2} V ${y5} Z`,
        fill: "#fff",
      }));
    } else if (btn.face.includes("\u221A")) {
      const radicand = btn.face.split("\u221A")[1] || "";
      drawRadical(svg, cx, cy, fs, radicand);
    } else if (MATH_RE.test(btn.face)) {
      const ft = svgEl("text", {
        x: String(cx), y: String(cy), "text-anchor": "middle",
        "font-family": FONT_STACK, "font-size": String(fs), "font-weight": "bold",
        "dominant-baseline": "central",
      }) as SVGTextElement;
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

const OPERATORS = new Set(["+", "\u2212", "-", "\u00D7", "\u00F7"]);

function faceFontSize(face: string, bodyW: number, wide: boolean): number {
  const len = face.length;
  const big = bodyW >= 120;
  if (len === 1 && OPERATORS.has(face)) return big ? 72 : 65;
  if (len === 1) return big ? 46 : 42;
  if (len === 2) return big ? 40 : 36;
  if (len === 3) return big ? 38 : 34;
  if (len <= 5) return wide ? 34 : big ? 30 : 26;
  return wide ? 28 : big ? 24 : 20;
}

function addShiftLabels(
  svg: SVGSVGElement, btn: ButtonDef, cx: number, bodyY: number,
): void {
  if (!btn.ls && !btn.rs) return;
  const labelY = bodyY - 6;
  const text = svgEl("text", {
    x: String(cx), y: String(labelY),
    "text-anchor": "middle",
    "font-family": FONT_STACK, "font-size": "26", "font-weight": "bold",
    "font-stretch": "condensed",
  }) as SVGTextElement;

  if (btn.ls && btn.rs) {
    appendStyledText(text, btn.ls, 26, LS_COLOR);
    appendStyledText(text, btn.rs, 26, RS_COLOR, "12");
  } else if (btn.ls) {
    appendStyledText(text, btn.ls, 26, LS_COLOR);
  } else {
    appendStyledText(text, btn.rs!, 26, RS_COLOR);
  }
  svg.appendChild(text);
}

function drawRadical(
  svg: SVGSVGElement, cx: number, cy: number, fontSize: number, radicand: string,
): void {
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
    fill: "none", stroke: "#fff", "stroke-width": "2.5",
    "stroke-linecap": "round", "stroke-linejoin": "round",
  }));

  const t = svgEl("text", {
    x: String(startX + radW + barW / 2), y: String(cy),
    "text-anchor": "middle", "dominant-baseline": "central",
    "font-family": FONT_STACK, "font-size": String(fontSize), "font-weight": "bold",
    fill: "#fff",
  }) as SVGTextElement;
  if (ITALIC_VARS.has(radicand)) t.setAttribute("font-style", "italic");
  t.textContent = radicand;
  svg.appendChild(t);
}

function drawShiftArrow(
  svg: SVGSVGElement, dir: "left" | "right",
  bx: number, by: number, bw: number, bh: number,
  strokeRef: string,
): void {
  const cy = by + bh / 2;
  const bot = by + bh - 5;
  const r = 3;
  const tail = dir === "left" ? bx + bw * 0.75 : bx + bw * 0.25;
  const tip  = dir === "left" ? bx + bw * 0.25 : bx + bw * 0.75;
  const attrs = {
    fill: "none", stroke: strokeRef, "stroke-width": "10",
    "stroke-linecap": "round", "stroke-linejoin": "round",
  };

  if (dir === "left") {
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tail} ${bot} V ${cy + r} Q ${tail} ${cy},${tail - r} ${cy} H ${tip}`,
    }));
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tip + 16} ${cy - 15} L ${tip} ${cy} L ${tip + 16} ${cy + 15}`,
    }));
  } else {
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tail} ${bot} V ${cy + r} Q ${tail} ${cy},${tail + r} ${cy} H ${tip}`,
    }));
    svg.appendChild(svgEl("path", {
      ...attrs,
      d: `M ${tip - 16} ${cy - 15} L ${tip} ${cy} L ${tip - 16} ${cy + 15}`,
    }));
  }
}

// ---------------------------------------------------------------------------
// Audio
// ---------------------------------------------------------------------------

const AUDIO_POLL_MS = 20;

let audioCtx: AudioContext | null = null;
let oscillator: OscillatorNode | null = null;
let gainNode: GainNode | null = null;
let playing = false;

function initAudioOnGesture(): void {
  if (audioCtx) return;
  try {
    const AudioCtor = window.AudioContext
      ?? (window as unknown as Record<string, unknown>).webkitAudioContext as typeof AudioContext | undefined;
    if (!AudioCtor) { console.warn("hp48: no AudioContext support"); return; }

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

function pollSpeaker(): void {
  const freq = hp48.speaker_frequency();
  if (freq > 0) {
    oscillator!.frequency.value = freq;
    if (!playing) {
      gainNode!.gain.value = 0.15;
      playing = true;
    }
  } else if (playing) {
    gainNode!.gain.value = 0;
    playing = false;
  }
}

function setupAudio(): void {
  const handler = (): void => {
    initAudioOnGesture();
    document.removeEventListener("mousedown", handler);
    document.removeEventListener("touchstart", handler);
    document.removeEventListener("keydown", handler);
  };
  document.addEventListener("mousedown", handler);
  document.addEventListener("touchstart", handler);
  document.addEventListener("keydown", handler);
}

// ---------------------------------------------------------------------------
// Auto-save (IndexedDB)
// ---------------------------------------------------------------------------

async function saveToIDB(): Promise<void> {
  try {
    const state = hp48.save_state();
    const ram = hp48.save_ram();
    await dbPut("state", state);
    await dbPut("ram", ram);
    console.log(`[hp48] saved to IDB: state=${state.byteLength}B, ram=${ram.byteLength}B`);
  } catch (e) { console.warn("[hp48] save failed", e); }
}

function startAutoSave(): void {
  setInterval(saveToIDB, AUTO_SAVE_INTERVAL_MS);
  // Save on page close — multiple events for Tauri/WebView2 reliability
  window.addEventListener("beforeunload", () => { void saveToIDB(); });
  window.addEventListener("pagehide", () => { void saveToIDB(); });
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") void saveToIDB();
  });
  // Save once shortly after boot so closing within 30s doesn't lose state
  setTimeout(saveToIDB, 5_000);
}

// ---------------------------------------------------------------------------
// Emulation loop
// ---------------------------------------------------------------------------

function startEmulationLoop(): void {
  let lastTime = performance.now();

  function frame(now: number): void {
    const elapsed = now - lastTime;
    lastTime = now;
    hp48.run_frame(elapsed, now / 1000.0);
    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

// ---------------------------------------------------------------------------
// Asset fetching helper
// ---------------------------------------------------------------------------

async function fetchAsset(name: string): Promise<Uint8Array> {
  const resp = await fetch(`./assets/${name}`);
  if (!resp.ok) throw new Error(`Failed to fetch assets/${name}: ${resp.status}`);
  return new Uint8Array(await resp.arrayBuffer());
}

// ---------------------------------------------------------------------------
// Bootstrap
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const wasm = await init();
  wasmMemory = wasm.memory;

  // Load ROM from assets/ (same files the C/Emscripten path uses)
  const rom = await fetchAsset("rom");

  // RAM and state: try IndexedDB first (saved from previous Rust session),
  // fall back to bundled assets/
  let ram = await dbGet("ram") ?? null;
  if (ram) {
    console.log(`[hp48] loaded RAM from IDB: ${ram.byteLength} bytes`);
  } else {
    try { ram = await fetchAsset("ram"); console.log(`[hp48] loaded RAM from assets: ${ram.byteLength} bytes`); } catch { console.log("[hp48] no RAM found, starting fresh"); }
  }
  let state = await dbGet("state") ?? null;
  if (state) {
    console.log(`[hp48] loaded state from IDB: ${state.byteLength} bytes`);
  } else {
    try { state = await fetchAsset("hp48"); console.log(`[hp48] loaded state from assets: ${state.byteLength} bytes`); } catch { console.log("[hp48] no state found, starting fresh"); }
  }

  hp48 = new Hp48(rom, ram, state);
  // C set_accesstime() uses local time (gettimeofday - timezone offset).
  // Date.now() is UTC ms; subtract timezone offset to get local epoch seconds.
  const localEpochSecs = Date.now() / 1000 - new Date().getTimezoneOffset() * 60;
  hp48.start(performance.now() / 1000.0, localEpochSecs);

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
