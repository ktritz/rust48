/*
 * annunc.h - Stub: bitmap data removed (annunciators handled in web build)
 */

#ifndef _ANNUNC_H
#define _ANNUNC_H 1

#define STUB_BITMAP(name) \
  static unsigned int name##_width = 0; \
  static unsigned int name##_height = 0; \
  static unsigned char name##_bits[] = {0};

STUB_BITMAP(ann_alpha)
STUB_BITMAP(ann_battery)
STUB_BITMAP(ann_busy)
STUB_BITMAP(ann_io)
STUB_BITMAP(ann_left)
STUB_BITMAP(ann_right)

#endif /* !_ANNUNC_H */
