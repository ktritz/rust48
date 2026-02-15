/*
 * hp.h - Stub: bitmap data removed (not used in web build)
 */

#ifndef _HP_H
#define _HP_H 1

#define STUB_BITMAP(name) \
  static unsigned int name##_width = 0; \
  static unsigned int name##_height = 0; \
  static unsigned char name##_bits[] = {0};

STUB_BITMAP(hp)
STUB_BITMAP(hp48sx)
STUB_BITMAP(hp48gx)
STUB_BITMAP(science)

#define gx_128K_ram_x_hot 1
#define gx_128K_ram_y_hot 8
STUB_BITMAP(gx_128K_ram)

#define gx_silver_x_hot 0
#define gx_silver_y_hot 8
STUB_BITMAP(gx_silver)

#define gx_green_x_hot 11
#define gx_green_y_hot 0
STUB_BITMAP(gx_green)

#endif /* !_HP_H */
