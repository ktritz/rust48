/*
 *  This file is part of x48, an emulator of the HP-48sx Calculator.
 *  Copyright (C) 1994  Eddie C. Dost  (ecd@dressler.de)
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
 */

/*
 * lcd.c - Modified for Emscripten/WASM web build.
 *
 * Changes from the original droid48 lcd.c:
 *   - Removed all JNI functions (Java_org_ab_x48_X48_*)
 *   - Replaced RGB565 disp_buf_short[] with RGBA8888 display_rgba[] buffer
 *   - Added EMSCRIPTEN_KEEPALIVE exported functions for JS access
 *   - Kept core display logic (update_display, draw_nibble, draw_row, etc.)
 *
 * Display is 262 pixels wide x 142 pixels tall:
 *   14 rows for header/annunciators + 128 rows for LCD (64 nibble rows x 2).
 */

#include "global.h"

#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <stdint.h>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#endif

#include "hp48.h"
#include "hp48_emu.h"
#include "annunc.h"
#include "device.h"

/* --- Display buffer dimensions --- */

#define DISPLAY_WIDTH  262
#define DISPLAY_HEIGHT 142   /* 14 header + 128 LCD */

#define HEADER_HEIGHT  14

/* RGBA pixel colors */
#define PIXEL_ON_R   0x10
#define PIXEL_ON_G   0x20
#define PIXEL_ON_B   0x10
#define PIXEL_ON_A   0xFF

#define PIXEL_OFF_R  0xBC
#define PIXEL_OFF_G  0xC4
#define PIXEL_OFF_B  0xA5
#define PIXEL_OFF_A  0xFF

/* --- Static data --- */

static int last_annunc_state = -1;

display_t display;

#define DISP_ROWS            64
#define NIBS_PER_BUFFER_ROW  (NIBBLES_PER_ROW + 2)

unsigned char disp_buf[DISP_ROWS][NIBS_PER_BUFFER_ROW];
unsigned char lcd_buffer[DISP_ROWS][NIBS_PER_BUFFER_ROW];

/* RGBA display buffer: 142 rows x 262 cols x 4 bytes per pixel */
static uint8_t display_rgba[DISPLAY_HEIGHT * DISPLAY_WIDTH * 4];

/* Dirty flag: set when display content has changed, cleared by JS after reading */
static int display_dirty = 1;

/* Annunciator boolean state (6 indicators) */
uint8_t ann_boolean[6];

Pixmap nibble_maps[16];

unsigned char nibbles[16][2] =
{
  { 0x00, 0x00 },	/* ---- */
  { 0x03, 0x03 },	/* *--- */
  { 0x0c, 0x0c },	/* -*-- */
  { 0x0f, 0x0f },	/* **-- */
  { 0x30, 0x30 },	/* --*- */
  { 0x33, 0x33 },	/* *-*- */
  { 0x3c, 0x3c },	/* -**- */
  { 0x3f, 0x3f },	/* ***- */
  { 0xc0, 0xc0 },	/* ---* */
  { 0xc3, 0xc3 },	/* *--* */
  { 0xcc, 0xcc },	/* -*-* */
  { 0xcf, 0xcf },	/* **-* */
  { 0xf0, 0xf0 },	/* --** */
  { 0xf3, 0xf3 },	/* *-** */
  { 0xfc, 0xfc },	/* -*** */
  { 0xff, 0xff }	/* **** */
};

static unsigned char nibble_bits[16];

/* --- RGBA nibble rendering --- */

/*
 * fill_display_rgba - Write one nibble (4 pixels wide, 2 rows tall) into
 * the RGBA display buffer.
 *
 * Parameters:
 *   x   - nibble column (0..33)
 *   y   - nibble row (0..63)
 *   v   - nibble value (0..15), each bit = one pixel (MSB first in HP convention)
 *
 * Each HP-48 pixel is 2 display columns wide and 2 display rows tall.
 * So one nibble = 4 HP pixels = 8 display columns x 2 display rows.
 * The LCD area starts at display row 14 (below the header).
 */
static void
#ifdef __FunctionProto__
fill_display_rgba(int x, int y, int v)
#else
fill_display_rgba(x, y, v)
int x;
int y;
int v;
#endif
{
  int px, py;
  int row, col, bit;
  int offset;
  uint8_t r, g, b, a;

  px = x * 8;
  py = y * 2 + HEADER_HEIGHT;

  /* Row bounds check */
  if (py + 1 >= DISPLAY_HEIGHT)
    return;

  for (bit = 0; bit < 4; bit++) {
    col = px + bit * 2;

    /* Skip pixel pairs that fall entirely outside the buffer */
    if (col + 1 >= DISPLAY_WIDTH)
      break;

    if (v & (1 << bit)) {
      r = PIXEL_ON_R;  g = PIXEL_ON_G;  b = PIXEL_ON_B;  a = PIXEL_ON_A;
    } else {
      r = PIXEL_OFF_R;  g = PIXEL_OFF_G;  b = PIXEL_OFF_B;  a = PIXEL_OFF_A;
    }

    for (row = 0; row < 2; row++) {
      offset = ((py + row) * DISPLAY_WIDTH + col) * 4;

      display_rgba[offset + 0] = r;
      display_rgba[offset + 1] = g;
      display_rgba[offset + 2] = b;
      display_rgba[offset + 3] = a;

      if (col + 1 < DISPLAY_WIDTH) {
        display_rgba[offset + 4] = r;
        display_rgba[offset + 5] = g;
        display_rgba[offset + 6] = b;
        display_rgba[offset + 7] = a;
      }
    }
  }

  display_dirty = 1;
}

/* --- EMSCRIPTEN_KEEPALIVE exported functions --- */

#ifdef __EMSCRIPTEN__

EMSCRIPTEN_KEEPALIVE
uint8_t* get_display_buffer(void)
{
  return display_rgba;
}

EMSCRIPTEN_KEEPALIVE
int get_display_width(void)
{
  return DISPLAY_WIDTH;
}

EMSCRIPTEN_KEEPALIVE
int get_display_height(void)
{
  return DISPLAY_HEIGHT;
}

EMSCRIPTEN_KEEPALIVE
int is_display_dirty(void)
{
  return display_dirty;
}

EMSCRIPTEN_KEEPALIVE
void clear_display_dirty(void)
{
  display_dirty = 0;
}

EMSCRIPTEN_KEEPALIVE
int get_annunciator_state(void)
{
  return display.annunc;
}

#endif /* __EMSCRIPTEN__ */

/* --- Core display functions --- */

void
#ifdef __FunctionProto__
init_nibble_maps(void)
#else
init_nibble_maps()
#endif
{
  int i;

  for (i = 0; i < 16; i++) {
    nibble_maps[i] = XCreateBitmapFromData(dpy, disp.win,
                                           (char *)nibbles[i], 8, 2);
  }
}

void
#ifdef __FunctionProto__
init_display(void)
#else
init_display()
#endif
{
  int i;

  disp.mapped = 1;
  display.on = (int)(saturn.disp_io & 0x8) >> 3;

  display.disp_start = (saturn.disp_addr & 0xffffe);
  display.offset = (saturn.disp_io & 0x7);
  disp.offset = 2 * display.offset;

  display.lines = (saturn.line_count & 0x3f);
  if (display.lines == 0)
    display.lines = 63;
  disp.lines = 2 * display.lines;
  if (disp.lines < 110)
    disp.lines = 110;

  if (display.offset > 3)
    display.nibs_per_line = (NIBBLES_PER_ROW+saturn.line_offset+2) & 0xfff;
  else
    display.nibs_per_line = (NIBBLES_PER_ROW+saturn.line_offset) & 0xfff;

  display.disp_end = display.disp_start +
               (display.nibs_per_line * (display.lines + 1));

  display.menu_start = saturn.menu_addr;
  display.menu_end = saturn.menu_addr + 0x110;

  display.contrast = saturn.contrast_ctrl;
  display.contrast |= ((saturn.disp_test & 0x1) << 4);

  display.annunc = saturn.annunc;

  memset(disp_buf, 0xf0, sizeof(disp_buf));
  memset(lcd_buffer, 0xf0, sizeof(lcd_buffer));

  /* Initialize the RGBA buffer to LCD background color */
  for (i = 0; i < DISPLAY_HEIGHT * DISPLAY_WIDTH; i++) {
    display_rgba[i * 4 + 0] = PIXEL_OFF_R;
    display_rgba[i * 4 + 1] = PIXEL_OFF_G;
    display_rgba[i * 4 + 2] = PIXEL_OFF_B;
    display_rgba[i * 4 + 3] = PIXEL_OFF_A;
  }

  display_dirty = 1;

  init_nibble_maps();
}

static inline void
#ifdef __FunctionProto__
draw_nibble(int c, int r, int val)
#else
draw_nibble(c, r, val)
int c;
int r;
int val;
#endif
{
  val &= 0x0f;
  if (val != lcd_buffer[r][c]) {
    lcd_buffer[r][c] = val;
    fill_display_rgba(c, r, val);
  }
}

static inline void
#ifdef __FunctionProto__
draw_row(long addr, int row)
#else
draw_row(addr, row)
long addr;
int row;
#endif
{
  int i, v;
  int line_length;

  line_length = NIBBLES_PER_ROW;
  if ((display.offset > 3) && (row <= display.lines))
    line_length += 2;
  for (i = 0; i < line_length; i++) {
    v = read_nibble(addr + i);
    if (v != disp_buf[row][i]) {
      disp_buf[row][i] = v;
      draw_nibble(i, row, v);
    }
  }
}

void
#ifdef __FunctionProto__
update_display(void)
#else
update_display()
#endif
{
  int i, j;
  long addr;
  static int old_offset = -1;
  static int old_lines = -1;

  if (display.on) {
    addr = display.disp_start;

    if (display.offset != old_offset) {
      memset(disp_buf, 0xf0,
             (size_t)((display.lines+1) * NIBS_PER_BUFFER_ROW));
      memset(lcd_buffer, 0xf0,
             (size_t)((display.lines+1) * NIBS_PER_BUFFER_ROW));
      old_offset = display.offset;
    }
    if (display.lines != old_lines) {
      memset(&disp_buf[56][0], 0xf0, (size_t)(8 * NIBS_PER_BUFFER_ROW));
      memset(&lcd_buffer[56][0], 0xf0, (size_t)(8 * NIBS_PER_BUFFER_ROW));
      old_lines = display.lines;
    }
    for (i = 0; i <= display.lines; i++) {
      draw_row(addr, i);
      addr += display.nibs_per_line;
    }

    if (i < DISP_ROWS) {
      addr = display.menu_start;
      for (; i < DISP_ROWS; i++) {
        draw_row(addr, i);
        addr += NIBBLES_PER_ROW;
      }
    }
  } else {
    memset(disp_buf, 0xf0, sizeof(disp_buf));
    for (i = 0; i < 64; i++) {
      for (j = 0; j < NIBBLES_PER_ROW; j++) {
        draw_nibble(j, i, 0x00);
      }
    }
  }
}

void
#ifdef __FunctionProto__
redraw_display(void)
#else
redraw_display()
#endif
{
  XClearWindow(dpy, disp.win);
  memset(disp_buf, 0, sizeof(disp_buf));
  memset(lcd_buffer, 0, sizeof(lcd_buffer));
  update_display();
}

void
#ifdef __FunctionProto__
disp_draw_nibble(word_20 addr, word_4 val)
#else
disp_draw_nibble(addr, val)
word_20 addr;
word_4 val;
#endif
{
  long offset;
  int x, y;

  offset = (addr - display.disp_start);
  x = offset % display.nibs_per_line;
  if (x < 0 || x > 35)
    return;
  if (display.nibs_per_line != 0) {
    y = offset / display.nibs_per_line;
    if (y < 0 || y > 63)
      return;
    if (val != disp_buf[y][x]) {
      disp_buf[y][x] = val;
      draw_nibble(x, y, val);
    }
  } else {
    for (y = 0; y < display.lines; y++) {
      if (val != disp_buf[y][x]) {
        disp_buf[y][x] = val;
        draw_nibble(x, y, val);
      }
    }
  }
}

void
#ifdef __FunctionProto__
menu_draw_nibble(word_20 addr, word_4 val)
#else
menu_draw_nibble(addr, val)
word_20 addr;
word_4 val;
#endif
{
  long offset;
  int x, y;

  offset = (addr - display.menu_start);
  x = offset % NIBBLES_PER_ROW;
  y = display.lines + (offset / NIBBLES_PER_ROW) + 1;
  if (val != disp_buf[y][x]) {
    disp_buf[y][x] = val;
    draw_nibble(x, y, val);
  }
}

/* --- Annunciators --- */

struct ann_struct {
  int            bit;
  int            x;
  int            y;
  unsigned int   width;
  unsigned int   height;
  unsigned char *bits;
  Pixmap         pixmap;
} ann_tbl[] = {
  { ANN_LEFT, 16, 4, ann_left_width, ann_left_height, ann_left_bits },
  { ANN_RIGHT, 61, 4, ann_right_width, ann_right_height, ann_right_bits },
  { ANN_ALPHA, 106, 4, ann_alpha_width, ann_alpha_height, ann_alpha_bits },
  { ANN_BATTERY, 151, 4, ann_battery_width, ann_battery_height,
                         ann_battery_bits },
  { ANN_BUSY, 196, 4, ann_busy_width, ann_busy_height, ann_busy_bits },
  { ANN_IO, 241, 4, ann_io_width, ann_io_height, ann_io_bits },
  { 0 }
};

void
#ifdef __FunctionProto__
draw_annunc(void)
#else
draw_annunc()
#endif
{
  int val;
  int i;

  val = display.annunc;

  if (val == last_annunc_state)
    return;
  last_annunc_state = val;

  for (i = 0; ann_tbl[i].bit; i++) {
    ann_boolean[i] = ((ann_tbl[i].bit & val) == ann_tbl[i].bit);
  }

  display_dirty = 1;
}

void
#ifdef __FunctionProto__
redraw_annunc(void)
#else
redraw_annunc()
#endif
{
  last_annunc_state = -1;
  draw_annunc();
}

void
#ifdef __FunctionProto__
init_annunc(void)
#else
init_annunc()
#endif
{
  int i;

  for (i = 0; ann_tbl[i].bit; i++) {
    ann_tbl[i].pixmap = XCreateBitmapFromData(dpy, disp.win,
                                              (char *)ann_tbl[i].bits,
                                              ann_tbl[i].width,
                                              ann_tbl[i].height);
  }
}
