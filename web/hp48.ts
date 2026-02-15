// hp48.ts — TypeScript bridge between the Emscripten WASM module and the browser.
// Compiled to hp48.js via esbuild. Must load BEFORE hp48_emu.js.

export {};

// ---------------------------------------------------------------------------
// Emscripten Module type declarations
// ---------------------------------------------------------------------------

interface HP48Module {
  HEAPU8: Uint8Array;
  onRuntimeInitialized: (() => void) | null;

  _push_key_event(code: number): void;
  _get_display_buffer(): number;
  _get_display_width(): number;
  _get_display_height(): number;
  _is_display_dirty(): number;
  _clear_display_dirty(): void;
  _get_annunciator_state(): number;
  _get_speaker_frequency(): number;
  _web_save_state(): void;
}

declare global {
  var Module: HP48Module;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DISPLAY_WIDTH = 262;
const DISPLAY_HEIGHT = 142;
const DISPLAY_BYTES = DISPLAY_WIDTH * DISPLAY_HEIGHT * 4;
const AUTO_SAVE_INTERVAL_MS = 30_000;

// ---------------------------------------------------------------------------
// Keyboard mapping: KeyboardEvent.key -> button ID (0-48)
// ---------------------------------------------------------------------------

// Direct key → button ID (for keys that map 1:1 to HP-48 buttons)
const KEY_MAP: Record<string, number> = {
  "0": 45, "1": 40, "2": 41, "3": 42, "4": 35,
  "5": 36, "6": 37, "7": 30, "8": 31, "9": 32,
  "Enter": 24, "Backspace": 28, "Delete": 27,
  ".": 46, "+": 48, "-": 43, "*": 38, "/": 33,
  " ": 47, "Escape": 44,
  "ArrowUp": 10, "ArrowDown": 16, "ArrowLeft": 15, "ArrowRight": 17,
  "'": 12,
};

// Alpha character → button ID (letters typed via ALPHA + button)
const ALPHA_MAP: Record<string, number> = {
  a: 0,  b: 1,  c: 2,  d: 3,  e: 4,  f: 5,
  g: 6,  h: 7,  i: 8,  j: 9,  k: 10, l: 11,
  m: 12, n: 13, o: 14, p: 15, q: 16, r: 17,
  s: 18, t: 19, u: 20, v: 21, w: 22, x: 23,
  y: 25, z: 26,
};

const ALPHA_BTN = 29;
const SHL_BTN = 34;  // left shift
const SHR_BTN = 39;  // right shift

// Shift + key combos: character → [shift button, target button]
// HP-48 operator keys have brackets/delimiters on their shift positions:
//   LS+÷ = ()   RS+÷ = #
//   LS+× = []   RS+× = _
//   LS+- = «»   RS+- = ""
//   LS++ = {}   RS++ = ::
const SHIFT_MAP: Record<string, [number, number]> = {
  "(": [SHL_BTN, 33],  // LS + ÷ → ()
  ")": [SHL_BTN, 33],  // same combo (enters pair)
  "[": [SHL_BTN, 38],  // LS + × → []
  "]": [SHL_BTN, 38],
  "{": [SHL_BTN, 48],  // LS + + → {}
  "}": [SHL_BTN, 48],
  "<": [SHL_BTN, 43],  // LS + - → «»
  ">": [SHL_BTN, 43],
  "#": [SHR_BTN, 33],  // RS + ÷ → #
  "_": [SHR_BTN, 38],  // RS + × → _
  "\"": [SHR_BTN, 43], // RS + - → ""
  ":": [SHR_BTN, 48],  // RS + + → ::
  ",": [SHL_BTN, 46],  // LS + . → ,
};

const pressedKeys = new Set<string>();

// ---------------------------------------------------------------------------
// Display rendering
// ---------------------------------------------------------------------------

function startDisplayLoop(): void {
  const canvas = document.getElementById("lcd") as HTMLCanvasElement | null;
  if (!canvas) { console.error("hp48: #lcd canvas not found"); return; }

  canvas.width = DISPLAY_WIDTH;
  canvas.height = DISPLAY_HEIGHT;

  const ctx = canvas.getContext("2d")!;
  if (!ctx) { console.error("hp48: no 2D context"); return; }

  const imageData = ctx.createImageData(DISPLAY_WIDTH, DISPLAY_HEIGHT);

  function frame(): void {
    if (Module._is_display_dirty()) {
      const ptr = Module._get_display_buffer();
      const src = Module.HEAPU8.subarray(ptr, ptr + DISPLAY_BYTES);
      imageData.data.set(src);
      Module._clear_display_dirty();
      ctx.putImageData(imageData, 0, 0);
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

    const pressCode = btnId + 1;         // 1-49 = press button 0-48
    const releaseCode = btnId + 101;     // 101-149 = release button 0-48

    function press(): void {
      el.classList.add("pressed");
      Module._push_key_event(pressCode);
    }
    function release(): void {
      el.classList.remove("pressed");
      Module._push_key_event(releaseCode);
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

/** Queue key events with delays so the emulator has time to scan the keyboard
 *  matrix between each event (needed for shift/alpha combos). */
function pushKeySequence(events: number[], delay = 20): void {
  events.forEach((code, i) => {
    if (i === 0) {
      Module._push_key_event(code);
    } else {
      setTimeout(() => Module._push_key_event(code), i * delay);
    }
  });
}

function setupKeyboardInput(): void {
  document.addEventListener("keydown", (e) => {
    // Direct button mapping (numbers, operators, arrows, etc.)
    const btnId = KEY_MAP[e.key];
    if (btnId !== undefined) {
      if (pressedKeys.has(e.key)) return;
      pressedKeys.add(e.key);
      e.preventDefault();
      Module._push_key_event(btnId + 1);
      return;
    }

    // Shift + key combos: brackets, delimiters, special chars
    const shiftCombo = SHIFT_MAP[e.key];
    if (shiftCombo !== undefined) {
      if (e.repeat) return;
      e.preventDefault();
      const [shiftBtn, targetBtn] = shiftCombo;
      pushKeySequence([
        shiftBtn + 1,       // shift press
        targetBtn + 1,      // key press (shift held)
        targetBtn + 101,    // key release
        shiftBtn + 101,     // shift release
      ]);
      return;
    }

    // Alpha characters: simulate ALPHA press/release then letter press/release
    const alphaBtn = ALPHA_MAP[e.key.toLowerCase()];
    if (alphaBtn !== undefined) {
      if (e.repeat || e.ctrlKey || e.metaKey || e.altKey) return;
      e.preventDefault();
      pushKeySequence([
        ALPHA_BTN + 1,      // ALPHA press
        alphaBtn + 1,       // letter press (ALPHA still held)
        alphaBtn + 101,     // letter release
        ALPHA_BTN + 101,    // ALPHA release
      ]);
      return;
    }
  });

  document.addEventListener("keyup", (e) => {
    const btnId = KEY_MAP[e.key];
    if (btnId === undefined) return;
    pressedKeys.delete(e.key);
    e.preventDefault();
    Module._push_key_event(btnId + 101);
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

// Body gradient — darker than calculator body (#303848)
const BODY_GRAD: [string, string] = ["#141c2c", "#2c3448"];

// Body dimensions — B6/B5 tuned so right column edges align across row types
const B6 = { x: 22, y: 33, w: 96,  h: 69, rx: 20 };   // 6-per-row (rows 1–4, viewBox 130×108)
const B5 = { x: 18, y: 33, w: 126, h: 69, rx: 16 };   // 5-per-row (rows 5–8, viewBox 156×108)
const BN = { x: 22, y: 33, w: 96,  h: 69, rx: 20 };   // narrow special keys (α, LS, RS, ON) — left-aligned with B6
const BE = { x: 22, y: 33, w: 226, h: 69, rx: 20 };   // ENTER spans 2 B6 columns (260×108)

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
  { face: "\u25C0",                 rs: "PICTURE", alpha: "P" },
  { face: "\u25BC",                 rs: "VIEW",    alpha: "Q" },
  { face: "\u25B6",                 rs: "SWAP",    alpha: "R" },
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

// Map Unicode superscript chars to their regular equivalents
const SUPER_MAP: Record<string, string> = {
  "\u00B2": "2",   // ² → 2
  "\u02E3": "x",   // ˣ → x
};
// Match superscripts, lowercase math variables, and radical sign
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

// Buttons with light gray cell background: RS key + number keys 1–9
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

  // Light gray cell background (dark body lines show through the margins)
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
  // Dark button body (frame + base) — from PNG: x=29,y=8,w≈104,h≈54
  svg.appendChild(svgEl("rect", {
    x: "26", y: "6", width: "104", height: "54",
    rx: "10", fill: "#203040", stroke: "#0a0e14", "stroke-width": "1",
  }));
  // White face area (inset) — from PNG: inner white region
  svg.appendChild(svgEl("rect", {
    x: "38", y: "12", width: "80", height: "30",
    rx: "6", fill: "#E8ECE4",
  }));
  // Alpha label (A–F) to the right of face
  if (btn.alpha) {
    const al = svgText(130, 54, btn.alpha, "#9a9a9a", "start", "bold", 22);
    al.setAttribute("font-stretch", "condensed");
    svg.appendChild(al
    );
  }
}

function buildStdButton(
  svg: SVGSVGElement, btn: ButtonDef, idx: number, vw: number, row: number,
): void {
  // Select body dimensions based on row type
  const bd = btn.wide ? BE : btn.narrow ? BN : row <= 4 ? B6 : B5;
  const { x: bx, y: by, w: bw, h: bh, rx } = bd;
  const isShift = !!btn.arrowDir;
  // Shift keys always use dark body gradient; bodyColor is for the inset fill
  const [gt, gb] = isShift ? BODY_GRAD : (btn.bodyColor || BODY_GRAD);
  const gid = `bg${idx}`;

  // Gradient definition
  const defs = svgEl("defs", {});
  const grad = svgEl("linearGradient", {
    id: gid, x1: "0", y1: "0", x2: "0", y2: "1",
  });
  grad.appendChild(svgEl("stop", { offset: "0%", "stop-color": gt }));
  grad.appendChild(svgEl("stop", { offset: "100%", "stop-color": gb }));
  defs.appendChild(grad);
  svg.appendChild(defs);

  // Shift labels — centered above button body
  addShiftLabels(svg, btn, vw / 2, by);

  // Drop shadow beneath button body
  svg.appendChild(svgEl("rect", {
    x: String(bx + 2), y: String(by + 3), width: String(bw), height: String(bh),
    rx: String(rx), fill: "rgba(0,0,0,0.5)",
    filter: "url(#bshadow)",
  }));

  // Filters (reuse if already defined)
  if (!svg.querySelector("#bshadow")) {
    const sf = svgEl("filter", { id: "bshadow", x: "-15%", y: "-15%", width: "140%", height: "140%" });
    sf.appendChild(svgEl("feGaussianBlur", { stdDeviation: "2.5" }));
    defs.appendChild(sf);
    const hf = svgEl("filter", { id: "bhlite", x: "-10%", y: "-10%", width: "130%", height: "130%" });
    hf.appendChild(svgEl("feGaussianBlur", { stdDeviation: "1.2" }));
    defs.appendChild(hf);
  }

  // Button body (dark for all buttons)
  svg.appendChild(svgEl("rect", {
    x: String(bx), y: String(by), width: String(bw), height: String(bh),
    rx: String(rx), fill: `url(#${gid})`, stroke: "#0c1018", "stroke-width": "1.5",
  }));

  // Shift keys: colored inset rect using global LS/RS colors
  if (isShift) {
    const inset = 7;
    const shiftCol = btn.arrowDir === "left" ? LS_COLOR : RS_COLOR;
    svg.appendChild(svgEl("rect", {
      x: String(bx + inset), y: String(by + inset),
      width: String(bw - 2 * inset), height: String(bh - 2 * inset),
      rx: String(Math.max(rx - inset, 6)), fill: shiftCol,
    }));
  }

  // Top edge highlight (light catching the top bevel of button) — blurred
  svg.appendChild(svgEl("rect", {
    x: String(bx + 3), y: String(by + 1), width: String(bw - 6), height: String(bh / 2),
    rx: String(rx - 2), fill: "rgba(255,255,255,0.09)",
    filter: "url(#bhlite)",
  }));

  // Bottom edge shadow (underside of button bevel) — blurred
  svg.appendChild(svgEl("line", {
    x1: String(bx + rx), y1: String(by + bh - 1.5),
    x2: String(bx + bw - rx), y2: String(by + bh - 1.5),
    stroke: "rgba(0,0,0,0.3)", "stroke-width": "2.5", "stroke-linecap": "round",
    filter: "url(#bhlite)",
  }));

  // Top highlight line on button face — blurred
  svg.appendChild(svgEl("line", {
    x1: String(bx + rx * 0.7), y1: String(by + 2.5),
    x2: String(bx + bw - rx * 0.7), y2: String(by + 2.5),
    stroke: "rgba(255,255,255,0.18)", "stroke-width": "1.5", "stroke-linecap": "round",
    filter: "url(#bhlite)",
  }));

  const cx = bx + bw / 2;
  const cy = by + bh / 2;

  // Face: shift arrow or text
  if (btn.arrowDir) {
    // Arrow gradient matching dark body (userSpaceOnUse for consistent mapping)
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
      // Thick stumpy backspace arrow
      const x1 = bx + bw * 0.25, x2 = bx + bw * 0.50, x3 = bx + bw * 0.75;
      const y1 = by + bh * 0.25, y2 = by + bh * 0.375, y3 = by + bh * 0.5;
      const y4 = by + bh * 0.625, y5 = by + bh * 0.75;
      svg.appendChild(svgEl("path", {
        d: `M ${x1} ${y3} L ${x2} ${y1} V ${y2} H ${x3} V ${y4} H ${x2} V ${y5} Z`,
        fill: "#fff",
      }));
    } else if (btn.face.includes("\u221A")) {
      // Radical with proper vinculum bar
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

  // Alpha label — bottom-right corner of button body
  if (btn.alpha) {
    const al = svgText(bx + bw + 2, by + bh + 4, btn.alpha, "#9a9a9a", "start", "bold", 22);
    al.setAttribute("font-stretch", "condensed");
    svg.appendChild(al
    );
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

  // Radical check + vinculum bar
  svg.appendChild(svgEl("path", {
    d: `M ${startX} ${midY} L ${startX + radW * 0.3} ${botY} L ${startX + radW} ${topY} H ${startX + totalW}`,
    fill: "none", stroke: "#fff", "stroke-width": "2.5",
    "stroke-linecap": "round", "stroke-linejoin": "round",
  }));

  // Radicand text under the bar
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
  const bot = by + bh - 5;      // account for stroke-linecap extending past endpoint
  const r = 3;                  // very tight bend
  // Tail at 25% from one side, arrowhead at 75% toward the other
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
// Audio — instruction-count based frequency/duration analysis
//
// The emulator toggles OUT[2] bit 3 to produce sound.  The C side measures
// half-period intervals in *instruction counts* (independent of emulation
// speed) and computes:
//   frequency = 4,000,000 / (2 * avg_half_period_instructions)
//   duration  = total_instruction_span * 1000 / 4,000,000   (ms)
//
// JS polls these values, plays an OscillatorNode at the correct frequency,
// and extends the tone to the correct wall-clock duration even if the
// emulation ran faster than real-time.
// ---------------------------------------------------------------------------

const AUDIO_POLL_MS = 20;

let audioCtx: AudioContext | null = null;
let oscillator: OscillatorNode | null = null;
let gainNode: GainNode | null = null;
let playing = false;

function initAudioOnGesture(): void {
  if (audioCtx) return;

  try {
    // Try webkitAudioContext for older WebKitGTK builds
    const AudioCtor = window.AudioContext
      ?? (window as unknown as Record<string, unknown>).webkitAudioContext as typeof AudioContext | undefined;
    if (!AudioCtor) { console.warn("hp48: no AudioContext support"); return; }

    audioCtx = new AudioCtor();
    // Must resume synchronously inside user-gesture handler
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
  const freq = Module._get_speaker_frequency();

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
// Auto-save
// ---------------------------------------------------------------------------

function startAutoSave(): void {
  setInterval(() => {
    try { Module._web_save_state(); } catch { /* ignore */ }
  }, AUTO_SAVE_INTERVAL_MS);

  window.addEventListener("beforeunload", () => {
    try { Module._web_save_state(); } catch { /* ignore */ }
  });
}

// ---------------------------------------------------------------------------
// Bootstrap — set Module global before hp48_emu.js loads
// ---------------------------------------------------------------------------

// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Emscripten expects a global Module
(window as any).Module = {
  onRuntimeInitialized(): void {
    console.log("HP-48 Emscripten runtime initialized");
    generateButtons();
    startDisplayLoop();
    setupButtonInput();
    setupKeyboardInput();
    setupAudio();
    showCalculator();
    startAutoSave();
  },
};
