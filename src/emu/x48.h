/*
 * x48.h - Emscripten/Web replacement for the original JNI/Android/X11 header.
 * Provides minimal stubs and logging macros for the web build.
 */

#ifndef _X48_H
#define _X48_H 1

#include "global.h"

#include <stdio.h>

/* Logging macros - map to printf for web/console */
#define LOGV(...) printf(__VA_ARGS__)
#define LOGD(...) printf(__VA_ARGS__)
#define LOGI(...) printf(__VA_ARGS__)
#define LOGW(...) fprintf(stderr, __VA_ARGS__)
#define LOGE(...) fprintf(stderr, __VA_ARGS__)

/* Color indices (kept for compatibility, not used for rendering) */
#define WHITE		0
#define LEFT		1
#define RIGHT		2
#define BUT_TOP 	3
#define BUTTON  	4
#define BUT_BOT 	5
#define LCD		6
#define PIXEL		7
#define PAD_TOP 	8
#define PAD		9
#define PAD_BOT		10
#define DISP_PAD_TOP	11
#define DISP_PAD	12
#define DISP_PAD_BOT	13
#define LOGO		14
#define LOGO_BACK	15
#define LABEL		16
#define FRAME		17
#define UNDERLAY	18
#define BLACK		19

/* Stub types replacing X11/Android types */
typedef struct XColor { int dummy; } XColor;
typedef struct Window { int dummy; } Window;
typedef struct GC { int dummy; } GC;
typedef struct Display { int dummy; } Display;
typedef struct Pixmap {
  char *data;
  int width;
  int height;
} Pixmap;
typedef struct Colormap { int dummy; } Colormap;
typedef struct Atom { int dummy; } Atom;
typedef struct Visual { int dummy; } Visual;

typedef struct color_t {
  char *name;
  int r, g, b;
  int mono_rgb;
  int gray_rgb;
  XColor xcolor;
} color_t;

extern color_t *colors;

#define COLOR(c) (0)

#define UPDATE_MENU	1
#define UPDATE_DISP	2

typedef struct disp_t {
  unsigned int     w, h;
  Window           win;
  GC               gc;
  short            mapped;
  int		   offset;
  int		   lines;
  int	display_update;
} disp_t;

extern disp_t   disp;
extern Display *dpy;
extern int	screen;
extern int  exit_state;

extern int	InitDisplay	 __ProtoType__((int argc, char **argv));
extern int	CreateWindows    __ProtoType__((int argc, char **argv));
extern int	GetEvent	 __ProtoType__((void));

extern void	adjust_contrast  __ProtoType__((int contrast));
extern void	refresh_icon	 __ProtoType__((void));

extern void	ShowConnections	 __ProtoType__((char *w, char *i));

extern Pixmap XCreateBitmapFromData __ProtoType__((Display *dpy, Window win, char* data, int a, int b));
extern void XClearArea __ProtoType__((Display *dpy, Window win, int x, int y, int width, int height, int boo));
extern void XCopyPlane __ProtoType__((Display *dpy, Pixmap map, Window win, GC gc, int a, int b, int x, int y, int width, int height, int boo));
extern void XClearWindow __ProtoType__((Display *dpy, Window win));

extern int button_pressed __ProtoType__((int b));
extern int button_released __ProtoType__((int b));

/* Web-specific: no blocking condition variable, just a no-op */
static inline void blockConditionVariable(void) { /* no-op in web build */ }

#endif /* !_X48_H */
