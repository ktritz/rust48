/*
 * init.c - Modified for Emscripten web build.
 * Removed #include <pwd.h> (not available in Emscripten).
 * Simplified get_home_directory() to not use getpwuid().
 */

#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <unistd.h>
#include <sys/stat.h>
#include <string.h>
#include <sys/types.h>
#ifdef SUNOS
#include <memory.h>
#endif

#include "hp48.h"
#include "hp48_emu.h"
#include "device.h"
#include "resources.h"
#include "romio.h"

#define X48_MAGIC 0x48503438
#define NR_CONFIG 8

short rom_is_new = 1;
long  ram_size;
long  port1_size;
long  port1_mask;
short port1_is_ram;
long  port2_size;
long  port2_mask;
short port2_is_ram;

typedef struct old_keystate_t {
  int rows[9];
} old_keystate_t;

typedef struct old_saturn_t {
  unsigned char A[16], B[16], C[16], D[16];
  long          d[2];
  int           P;
  long          PC;
  unsigned char R0[16], R1[16], R2[16], R3[16], R4[16];
  unsigned char IN[4];
  unsigned char OUT[3];
  int           CARRY;
  unsigned char PSTAT[NR_PSTAT];
  unsigned char XM, SB, SR, MP;
  unsigned char hexmode;
  long          rstk[NR_RSTK];
  short         rstkp;
  old_keystate_t keybuf;
  unsigned char intenable;
  unsigned char int_pending;
  unsigned char kbd_ien;
  long          configs[NR_CONFIG];
  short         daisy_state;
  long          ram32k;
  long          devices;
  unsigned char disp_io;
  unsigned char contrast_ctrl;
  unsigned char disp_test;
  unsigned int  crc;
  unsigned char power_status;
  unsigned char power_ctrl;
  unsigned char mode;
  unsigned char annunc;
  unsigned char baud;
  unsigned char card_ctrl;
  unsigned char card_status;
  unsigned char io_ctrl;
  unsigned char rcs;
  unsigned char tcs;
  unsigned char rbr;
  unsigned char tbr;
  unsigned char sreq;
  unsigned char ir_ctrl;
  unsigned char base_off;
  unsigned char lcr;
  unsigned char lbr;
  unsigned char scratch;
  unsigned char base_nibble;
  long          disp_addr;
  long          line_offset;
  long          line_count;
  long          unknown;
  unsigned char t1_ctrl;
  unsigned char t2_ctrl;
  long          menu_addr;
  long          unknown2;
  int           timer1;
  long          timer2;
  long          t1_instr;
  long          t2_instr;
  unsigned char *rom;
  unsigned char *ram;
  unsigned char *port1;
  unsigned char *port2;
} old_saturn_t;

old_saturn_t old_saturn;

typedef struct saturn_0_3_0_t {
  unsigned long magic;
  char          version[4];
  unsigned char A[16], B[16], C[16], D[16];
  word_20       d[2];
  word_4        P;
  word_20       PC;
  unsigned char R0[16], R1[16], R2[16], R3[16], R4[16];
  unsigned char IN[4];
  unsigned char OUT[3];
  word_1        CARRY;
  unsigned char PSTAT[NR_PSTAT];
  unsigned char XM, SB, SR, MP;
  word_4        hexmode;
  word_20       rstk[NR_RSTK];
  short         rstkp;
  keystate_t    keybuf;
  unsigned char intenable;
  unsigned char int_pending;
  unsigned char kbd_ien;
  word_20	configs[NR_CONFIG];
  word_16	daisy_state;
  word_20	ram32k;
  word_20	devices;
  word_4        disp_io;
  word_4        contrast_ctrl;
  word_8        disp_test;
  word_16       crc;
  word_4        power_status;
  word_4        power_ctrl;
  word_4        mode;
  word_8        annunc;
  word_4        baud;
  word_4        card_ctrl;
  word_4        card_status;
  word_4        io_ctrl;
  word_4        rcs;
  word_4        tcs;
  word_8        rbr;
  word_8        tbr;
  word_8        sreq;
  word_4        ir_ctrl;
  word_4        base_off;
  word_4        lcr;
  word_4        lbr;
  word_4        scratch;
  word_4        base_nibble;
  word_20       disp_addr;
  word_12       line_offset;
  word_8        line_count;
  word_16       unknown;
  word_4        t1_ctrl;
  word_4        t2_ctrl;
  word_20       menu_addr;
  word_8        unknown2;
  char          timer1;
  word_32       timer2;
  long          t1_instr;
  long          t2_instr;
  short         t1_tick;
  short         t2_tick;
  long          i_per_s;
  unsigned char *rom;
  unsigned char *ram;
  unsigned char *port1;
  unsigned char *port2;
} saturn_0_3_0_t;

saturn_0_3_0_t saturn_0_3_0;

#include "config.h"

void
#ifdef __FunctionProto__
saturn_config_init(void)
#else
saturn_config_init()
#endif
{
  saturn.version[0] = VERSION_MAJOR;
  saturn.version[1] = VERSION_MINOR;
  saturn.version[2] = PATCHLEVEL;
  saturn.version[3] = COMPILE_VERSION;
  memset(&device, 0, sizeof(device));
  device.display_touched = 1;
  device.contrast_touched = 1;
  device.baud_touched = 1;
  device.ann_touched = 1;
  saturn.rcs = 0x0;
  saturn.tcs = 0x0;
  saturn.lbr = 0x0;
}

void
#ifdef __FunctionProto__
init_saturn(void)
#else
init_saturn()
#endif
{
  int i;

  memset(&saturn, 0, sizeof(saturn) - 4 * sizeof(unsigned char *));
  saturn.PC = 0x00000;
  saturn.magic = X48_MAGIC;
  saturn.t1_tick = 8192;
  saturn.t2_tick = 16;
  saturn.i_per_s = 0;
  saturn.version[0] = VERSION_MAJOR;
  saturn.version[1] = VERSION_MINOR;
  saturn.version[2] = PATCHLEVEL;
  saturn.version[3] = COMPILE_VERSION;
  saturn.hexmode = HEX;
  saturn.rstkp = -1;
  saturn.intenable = 1;
  saturn.int_pending = 0;
  saturn.kbd_ien = 1;
  saturn.timer1 = 0;
  saturn.timer2 = 0x2000;
  saturn.bank_switch = 0;
  for (i = 0; i < NR_MCTL; i++)
    {
      if (i == 0)
        saturn.mem_cntl[i].unconfigured = 1;
      else if (i == 5)
        saturn.mem_cntl[i].unconfigured = 0;
      else
        saturn.mem_cntl[i].unconfigured = 2;
      saturn.mem_cntl[i].config[0] = 0;
      saturn.mem_cntl[i].config[1] = 0;
    }
  dev_memory_init();
}

void
#ifdef __FunctionProto__
copy_old_saturn(old_saturn_t *old, saturn_t *new)
#else
copy_old_saturn(old, new)
old_saturn_t *old;
saturn_t *new;
#endif
{
  int i;

  memcpy(&(new->A[0]), &(old->A[0]), 16);
  memcpy(&(new->B[0]), &(old->B[0]), 16);
  memcpy(&(new->C[0]), &(old->C[0]), 16);
  memcpy(&(new->D[0]), &(old->D[0]), 16);
  new->d[0] = old->d[0];
  new->d[1] = old->d[1];
  new->P = old->P;
  new->PC = old->PC;
  memcpy(&(new->R0[0]), &(old->R0[0]), 16);
  memcpy(&(new->R1[0]), &(old->R1[0]), 16);
  memcpy(&(new->R2[0]), &(old->R2[0]), 16);
  memcpy(&(new->R3[0]), &(old->R3[0]), 16);
  memcpy(&(new->R4[0]), &(old->R4[0]), 16);
  memcpy(&(new->IN[0]), &(old->IN[0]), 4);
  memcpy(&(new->OUT[0]), &(old->OUT[0]), 3);
  new->CARRY = old->CARRY;
  memcpy(&(new->PSTAT[0]), &(old->PSTAT[0]), NR_PSTAT);
  new->XM = old->XM;
  new->SB = old->SB;
  new->SR = old->SR;
  new->MP = old->MP;
  new->hexmode = old->hexmode;
  memcpy(&(new->rstk[0]), &(old->rstk[0]), NR_RSTK * sizeof(word_20));
  new->rstkp = old->rstkp;
  for (i = 0; i < 9; i++) {
    new->keybuf.rows[i] = old->keybuf.rows[i];
  }
  new->intenable = old->intenable;
  new->int_pending = old->int_pending;
  new->kbd_ien = old->kbd_ien;
  new->disp_io = old->disp_io;
  new->contrast_ctrl = old->contrast_ctrl;
  new->disp_test = old->disp_test;
  new->crc = old->crc;
  new->power_status = old->power_status;
  new->power_ctrl = old->power_ctrl;
  new->mode = old->mode;
  new->annunc = old->annunc;
  new->baud = old->baud;
  new->card_ctrl = old->card_ctrl;
  new->card_status = old->card_status;
  new->io_ctrl = old->io_ctrl;
  new->rcs = old->rcs;
  new->tcs = old->tcs;
  new->rbr = old->rbr;
  new->tbr = old->tbr;
  new->sreq = old->sreq;
  new->ir_ctrl = old->ir_ctrl;
  new->base_off = old->base_off;
  new->lcr = old->lcr;
  new->lbr = old->lbr;
  new->scratch = old->scratch;
  new->base_nibble = old->base_nibble;
  new->disp_addr = old->disp_addr;
  new->line_offset = old->line_offset;
  new->line_count = old->line_count;
  new->unknown = old->unknown;
  new->t1_ctrl = old->t1_ctrl;
  new->t2_ctrl = old->t2_ctrl;
  new->menu_addr = old->menu_addr;
  new->unknown2 = old->unknown2;
  new->timer1 = old->timer1;
  new->timer2 = old->timer2;
  new->t1_instr = old->t1_instr;
  new->t2_instr = old->t2_instr;
  new->bank_switch = 0;
  if (opt_gx)
    {
      new->mem_cntl[0].unconfigured = 0;
      new->mem_cntl[0].config[0] = 0x00100;
      new->mem_cntl[1].unconfigured = 0;
      new->mem_cntl[1].config[0] = 0x80000;
      new->mem_cntl[1].config[1] = 0xc0000;
      new->mem_cntl[2].unconfigured = 0;
      new->mem_cntl[2].config[0] = 0x7f000;
      new->mem_cntl[2].config[1] = 0xff000;
      new->mem_cntl[3].unconfigured = 0;
      new->mem_cntl[3].config[0] = 0xc0000;
      new->mem_cntl[3].config[1] = 0xc0000;
      new->mem_cntl[4].unconfigured = 0;
      new->mem_cntl[4].config[0] = 0xc0000;
      new->mem_cntl[4].config[1] = 0xc0000;
      new->mem_cntl[5].unconfigured = 0;
      new->mem_cntl[5].config[0] = 0x00000;
      new->mem_cntl[5].config[1] = 0x00000;
    }
  else
    {
      if (old->devices == 0x100)
        {
          new->mem_cntl[0].unconfigured = 0;
          new->mem_cntl[0].config[0] = old->devices;
        }
      else
        {
          new->mem_cntl[0].unconfigured = 1;
          new->mem_cntl[0].config[0] = 0x00000;
        }
      if (old->ram32k == 0x70000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xf0000;
        }
      else if (old->ram32k == 0xf0000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0xf0000;
          new->mem_cntl[1].config[1] = 0xf0000;
        }
      else if (old->ram32k == 0xfc000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xfc000;
        }
      else if (old->ram32k == 0xfe000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xfe000;
        }
      else
        {
          new->mem_cntl[1].unconfigured = 2;
          new->mem_cntl[1].config[0] = 0x00000;
          new->mem_cntl[1].config[1] = 0x00000;
        }
      new->mem_cntl[2].unconfigured = 0;
      new->mem_cntl[2].config[0] = 0x80000;
      new->mem_cntl[2].config[1] = 0xc0000;
      new->mem_cntl[3].unconfigured = 0;
      new->mem_cntl[3].config[0] = 0xc0000;
      new->mem_cntl[3].config[1] = 0xc0000;
      new->mem_cntl[4].unconfigured = 0;
      new->mem_cntl[4].config[0] = 0xd0000;
      new->mem_cntl[4].config[1] = 0xff000;
      new->mem_cntl[5].unconfigured = 0;
      new->mem_cntl[5].config[0] = 0x00000;
      new->mem_cntl[5].config[1] = 0x80000;
    }
}

void
#ifdef __FunctionProto__
copy_0_3_0_saturn(saturn_0_3_0_t *old, saturn_t *new)
#else
copy_0_3_0_saturn(old, new)
saturn_0_3_0_t *old;
saturn_t *new;
#endif
{
  int i;

  memcpy(&(new->A[0]), &(old->A[0]), 16);
  memcpy(&(new->B[0]), &(old->B[0]), 16);
  memcpy(&(new->C[0]), &(old->C[0]), 16);
  memcpy(&(new->D[0]), &(old->D[0]), 16);
  new->d[0] = old->d[0];
  new->d[1] = old->d[1];
  new->P = old->P;
  new->PC = old->PC;
  memcpy(&(new->R0[0]), &(old->R0[0]), 16);
  memcpy(&(new->R1[0]), &(old->R1[0]), 16);
  memcpy(&(new->R2[0]), &(old->R2[0]), 16);
  memcpy(&(new->R3[0]), &(old->R3[0]), 16);
  memcpy(&(new->R4[0]), &(old->R4[0]), 16);
  memcpy(&(new->IN[0]), &(old->IN[0]), 4);
  memcpy(&(new->OUT[0]), &(old->OUT[0]), 3);
  new->CARRY = old->CARRY;
  memcpy(&(new->PSTAT[0]), &(old->PSTAT[0]), NR_PSTAT);
  new->XM = old->XM;
  new->SB = old->SB;
  new->SR = old->SR;
  new->MP = old->MP;
  new->hexmode = old->hexmode;
  memcpy(&(new->rstk[0]), &(old->rstk[0]), NR_RSTK * sizeof(word_20));
  new->rstkp = old->rstkp;
  for (i = 0; i < 9; i++) {
    new->keybuf.rows[i] = old->keybuf.rows[i];
  }
  new->intenable = old->intenable;
  new->int_pending = old->int_pending;
  new->kbd_ien = old->kbd_ien;
  new->disp_io = old->disp_io;
  new->contrast_ctrl = old->contrast_ctrl;
  new->disp_test = old->disp_test;
  new->crc = old->crc;
  new->power_status = old->power_status;
  new->power_ctrl = old->power_ctrl;
  new->mode = old->mode;
  new->annunc = old->annunc;
  new->baud = old->baud;
  new->card_ctrl = old->card_ctrl;
  new->card_status = old->card_status;
  new->io_ctrl = old->io_ctrl;
  new->rcs = old->rcs;
  new->tcs = old->tcs;
  new->rbr = old->rbr;
  new->tbr = old->tbr;
  new->sreq = old->sreq;
  new->ir_ctrl = old->ir_ctrl;
  new->base_off = old->base_off;
  new->lcr = old->lcr;
  new->lbr = old->lbr;
  new->scratch = old->scratch;
  new->base_nibble = old->base_nibble;
  new->disp_addr = old->disp_addr;
  new->line_offset = old->line_offset;
  new->line_count = old->line_count;
  new->unknown = old->unknown;
  new->t1_ctrl = old->t1_ctrl;
  new->t2_ctrl = old->t2_ctrl;
  new->menu_addr = old->menu_addr;
  new->unknown2 = old->unknown2;
  new->timer1 = old->timer1;
  new->timer2 = old->timer2;
  new->t1_instr = old->t1_instr;
  new->t2_instr = old->t2_instr;
  new->t1_tick = old->t1_tick;
  new->t2_tick = old->t2_tick;
  new->i_per_s = old->i_per_s;
  new->bank_switch = 0;
  if (opt_gx)
    {
      new->mem_cntl[0].unconfigured = 0;
      new->mem_cntl[0].config[0] = 0x00100;
      new->mem_cntl[1].unconfigured = 0;
      new->mem_cntl[1].config[0] = 0x80000;
      new->mem_cntl[1].config[1] = 0xc0000;
      new->mem_cntl[2].unconfigured = 0;
      new->mem_cntl[2].config[0] = 0x7f000;
      new->mem_cntl[2].config[1] = 0xff000;
      new->mem_cntl[3].unconfigured = 0;
      new->mem_cntl[3].config[0] = 0xc0000;
      new->mem_cntl[3].config[1] = 0xc0000;
      new->mem_cntl[4].unconfigured = 0;
      new->mem_cntl[4].config[0] = 0xc0000;
      new->mem_cntl[4].config[1] = 0xc0000;
      new->mem_cntl[5].unconfigured = 0;
      new->mem_cntl[5].config[0] = 0x00000;
      new->mem_cntl[5].config[1] = 0x00000;
    }
  else
    {
      if (old->devices == 0x100)
        {
          new->mem_cntl[0].unconfigured = 0;
          new->mem_cntl[0].config[0] = old->devices;
        }
      else
        {
          new->mem_cntl[0].unconfigured = 1;
          new->mem_cntl[0].config[0] = 0x00000;
        }
      if (old->ram32k == 0x70000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xf0000;
        }
      else if (old->ram32k == 0xf0000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0xf0000;
          new->mem_cntl[1].config[1] = 0xf0000;
        }
      else if (old->ram32k == 0xfc000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xfc000;
        }
      else if (old->ram32k == 0xfe000)
        {
          new->mem_cntl[1].unconfigured = 0;
          new->mem_cntl[1].config[0] = 0x70000;
          new->mem_cntl[1].config[1] = 0xfe000;
        }
      else
        {
          new->mem_cntl[1].unconfigured = 2;
          new->mem_cntl[1].config[0] = 0x00000;
          new->mem_cntl[1].config[1] = 0x00000;
        }
      new->mem_cntl[2].unconfigured = 0;
      new->mem_cntl[2].config[0] = 0x80000;
      new->mem_cntl[2].config[1] = 0xc0000;
      new->mem_cntl[3].unconfigured = 0;
      new->mem_cntl[3].config[0] = 0xc0000;
      new->mem_cntl[3].config[1] = 0xc0000;
      new->mem_cntl[4].unconfigured = 0;
      new->mem_cntl[4].config[0] = 0xd0000;
      new->mem_cntl[4].config[1] = 0xff000;
      new->mem_cntl[5].unconfigured = 0;
      new->mem_cntl[5].config[0] = 0x00000;
      new->mem_cntl[5].config[1] = 0x80000;
    }
}

/* Read/write helper functions - same as original */
int read_8(FILE *fp, word_8 *var) { unsigned char tmp; if (fread(&tmp, 1, 1, fp) != 1) return 0; *var = tmp; return 1; }
int read_char(FILE *fp, char *var) { char tmp; if (fread(&tmp, 1, 1, fp) != 1) return 0; *var = tmp; return 1; }
int read_16(FILE *fp, word_16 *var) { unsigned char tmp[2]; if (fread(&tmp[0], 1, 2, fp) != 2) return 0; *var = tmp[0] << 8; *var |= tmp[1]; return 1; }
int read_32(FILE *fp, word_32 *var) { unsigned char tmp[4]; if (fread(&tmp[0], 1, 4, fp) != 4) return 0; *var = tmp[0] << 24; *var |= tmp[1] << 16; *var |= tmp[2] << 8; *var |= tmp[3]; return 1; }
int read_u_long(FILE *fp, unsigned long *var) { unsigned char tmp[4]; if (fread(&tmp[0], 1, 4, fp) != 4) return 0; *var = tmp[0] << 24; *var |= tmp[1] << 16; *var |= tmp[2] << 8; *var |= tmp[3]; return 1; }

int read_version_0_3_0_file(FILE *fp);
int read_version_0_4_0_file(FILE *fp);
int read_mem_file(char *name, word_4 *mem, int size);

int write_8(FILE *fp, word_8 *var) { unsigned char tmp = *var; return fwrite(&tmp, 1, 1, fp) == 1; }
int write_char(FILE *fp, char *var) { char tmp = *var; return fwrite(&tmp, 1, 1, fp) == 1; }
int write_16(FILE *fp, word_16 *var) { unsigned char tmp[2]; tmp[0] = (*var >> 8) & 0xff; tmp[1] = *var & 0xff; return fwrite(&tmp[0], 1, 2, fp) == 2; }
int write_32(FILE *fp, word_32 *var) { unsigned char tmp[4]; tmp[0] = (*var >> 24) & 0xff; tmp[1] = (*var >> 16) & 0xff; tmp[2] = (*var >> 8) & 0xff; tmp[3] = *var & 0xff; return fwrite(&tmp[0], 1, 4, fp) == 4; }
int write_u_long(FILE *fp, unsigned long *var) { unsigned char tmp[4]; tmp[0] = (*var >> 24) & 0xff; tmp[1] = (*var >> 16) & 0xff; tmp[2] = (*var >> 8) & 0xff; tmp[3] = *var & 0xff; return fwrite(&tmp[0], 1, 4, fp) == 4; }

int read_version_0_3_0_file(FILE *fp) {
  int i;
  for (i = 0; i < 16; i++) if(!read_8(fp, &saturn_0_3_0.A[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.B[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.C[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.D[i])) return 0;
  if (!read_32(fp, &saturn_0_3_0.d[0])) return 0;
  if (!read_32(fp, &saturn_0_3_0.d[1])) return 0;
  if (!read_8(fp, &saturn_0_3_0.P)) return 0;
  if (!read_32(fp, &saturn_0_3_0.PC)) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.R0[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.R1[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.R2[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.R3[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn_0_3_0.R4[i])) return 0;
  for (i = 0; i < 4; i++) if (!read_8(fp, &saturn_0_3_0.IN[i])) return 0;
  for (i = 0; i < 3; i++) if (!read_8(fp, &saturn_0_3_0.OUT[i])) return 0;
  if (!read_8(fp, &saturn_0_3_0.CARRY)) return 0;
  for (i = 0; i < NR_PSTAT; i++) if (!read_8(fp, &saturn_0_3_0.PSTAT[i])) return 0;
  if (!read_8(fp, &saturn_0_3_0.XM)) return 0;
  if (!read_8(fp, &saturn_0_3_0.SB)) return 0;
  if (!read_8(fp, &saturn_0_3_0.SR)) return 0;
  if (!read_8(fp, &saturn_0_3_0.MP)) return 0;
  if (!read_8(fp, &saturn_0_3_0.hexmode)) return 0;
  for (i = 0; i < NR_RSTK; i++) if (!read_32(fp, &saturn_0_3_0.rstk[i])) return 0;
  if (!read_16(fp, (word_16 *)&saturn_0_3_0.rstkp)) return 0;
  for (i = 0; i < 9; i++) if (!read_16(fp, (word_16 *)&saturn_0_3_0.keybuf.rows[i])) return 0;
  if (!read_8(fp, &saturn_0_3_0.intenable)) return 0;
  if (!read_8(fp, &saturn_0_3_0.int_pending)) return 0;
  if (!read_8(fp, &saturn_0_3_0.kbd_ien)) return 0;
  for (i = 0; i < NR_CONFIG; i++) if (!read_32(fp, &saturn_0_3_0.configs[i])) return 0;
  if (!read_16(fp, (word_16 *)&saturn_0_3_0.daisy_state)) return 0;
  if (!read_32(fp, &saturn_0_3_0.ram32k)) return 0;
  if (!read_32(fp, &saturn_0_3_0.devices)) return 0;
  if (!read_8(fp, &saturn_0_3_0.disp_io)) return 0;
  if (!read_8(fp, &saturn_0_3_0.contrast_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.disp_test)) return 0;
  if (!read_16(fp, &saturn_0_3_0.crc)) return 0;
  if (!read_8(fp, &saturn_0_3_0.power_status)) return 0;
  if (!read_8(fp, &saturn_0_3_0.power_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.mode)) return 0;
  if (!read_8(fp, &saturn_0_3_0.annunc)) return 0;
  if (!read_8(fp, &saturn_0_3_0.baud)) return 0;
  if (!read_8(fp, &saturn_0_3_0.card_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.card_status)) return 0;
  if (!read_8(fp, &saturn_0_3_0.io_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.rcs)) return 0;
  if (!read_8(fp, &saturn_0_3_0.tcs)) return 0;
  if (!read_8(fp, &saturn_0_3_0.rbr)) return 0;
  if (!read_8(fp, &saturn_0_3_0.tbr)) return 0;
  if (!read_8(fp, &saturn_0_3_0.sreq)) return 0;
  if (!read_8(fp, &saturn_0_3_0.ir_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.base_off)) return 0;
  if (!read_8(fp, &saturn_0_3_0.lcr)) return 0;
  if (!read_8(fp, &saturn_0_3_0.lbr)) return 0;
  if (!read_8(fp, &saturn_0_3_0.scratch)) return 0;
  if (!read_8(fp, &saturn_0_3_0.base_nibble)) return 0;
  if (!read_32(fp, &saturn_0_3_0.disp_addr)) return 0;
  if (!read_16(fp, &saturn_0_3_0.line_offset)) return 0;
  if (!read_8(fp, &saturn_0_3_0.line_count)) return 0;
  if (!read_16(fp, &saturn_0_3_0.unknown)) return 0;
  if (!read_8(fp, &saturn_0_3_0.t1_ctrl)) return 0;
  if (!read_8(fp, &saturn_0_3_0.t2_ctrl)) return 0;
  if (!read_32(fp, &saturn_0_3_0.menu_addr)) return 0;
  if (!read_8(fp, &saturn_0_3_0.unknown2)) return 0;
  if (!read_char(fp, &saturn_0_3_0.timer1)) return 0;
  if (!read_32(fp, &saturn_0_3_0.timer2)) return 0;
  if (!read_32(fp, &saturn_0_3_0.t1_instr)) return 0;
  if (!read_32(fp, &saturn_0_3_0.t2_instr)) return 0;
  if (!read_16(fp, (word_16 *)&saturn_0_3_0.t1_tick)) return 0;
  if (!read_16(fp, (word_16 *)&saturn_0_3_0.t2_tick)) return 0;
  if (!read_32(fp, &saturn_0_3_0.i_per_s)) return 0;
  return 1;
}

int read_version_0_4_0_file(FILE *fp) {
  int i;
  for (i = 0; i < 16; i++) if(!read_8(fp, &saturn.A[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.B[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.C[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.D[i])) return 0;
  if (!read_32(fp, &saturn.d[0])) return 0;
  if (!read_32(fp, &saturn.d[1])) return 0;
  if (!read_8(fp, &saturn.P)) return 0;
  if (!read_32(fp, &saturn.PC)) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.R0[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.R1[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.R2[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.R3[i])) return 0;
  for (i = 0; i < 16; i++) if (!read_8(fp, &saturn.R4[i])) return 0;
  for (i = 0; i < 4; i++) if (!read_8(fp, &saturn.IN[i])) return 0;
  for (i = 0; i < 3; i++) if (!read_8(fp, &saturn.OUT[i])) return 0;
  if (!read_8(fp, &saturn.CARRY)) return 0;
  for (i = 0; i < NR_PSTAT; i++) if (!read_8(fp, &saturn.PSTAT[i])) return 0;
  if (!read_8(fp, &saturn.XM)) return 0;
  if (!read_8(fp, &saturn.SB)) return 0;
  if (!read_8(fp, &saturn.SR)) return 0;
  if (!read_8(fp, &saturn.MP)) return 0;
  if (!read_8(fp, &saturn.hexmode)) return 0;
  for (i = 0; i < NR_RSTK; i++) if (!read_32(fp, &saturn.rstk[i])) return 0;
  if (!read_16(fp, (word_16 *)&saturn.rstkp)) return 0;
  for (i = 0; i < 9; i++) if (!read_16(fp, (word_16 *)&saturn.keybuf.rows[i])) return 0;
  if (!read_8(fp, &saturn.intenable)) return 0;
  if (!read_8(fp, &saturn.int_pending)) return 0;
  if (!read_8(fp, &saturn.kbd_ien)) return 0;
  if (!read_8(fp, &saturn.disp_io)) return 0;
  if (!read_8(fp, &saturn.contrast_ctrl)) return 0;
  if (!read_8(fp, &saturn.disp_test)) return 0;
  if (!read_16(fp, &saturn.crc)) return 0;
  if (!read_8(fp, &saturn.power_status)) return 0;
  if (!read_8(fp, &saturn.power_ctrl)) return 0;
  if (!read_8(fp, &saturn.mode)) return 0;
  if (!read_8(fp, &saturn.annunc)) return 0;
  if (!read_8(fp, &saturn.baud)) return 0;
  if (!read_8(fp, &saturn.card_ctrl)) return 0;
  if (!read_8(fp, &saturn.card_status)) return 0;
  if (!read_8(fp, &saturn.io_ctrl)) return 0;
  if (!read_8(fp, &saturn.rcs)) return 0;
  if (!read_8(fp, &saturn.tcs)) return 0;
  if (!read_8(fp, &saturn.rbr)) return 0;
  if (!read_8(fp, &saturn.tbr)) return 0;
  if (!read_8(fp, &saturn.sreq)) return 0;
  if (!read_8(fp, &saturn.ir_ctrl)) return 0;
  if (!read_8(fp, &saturn.base_off)) return 0;
  if (!read_8(fp, &saturn.lcr)) return 0;
  if (!read_8(fp, &saturn.lbr)) return 0;
  if (!read_8(fp, &saturn.scratch)) return 0;
  if (!read_8(fp, &saturn.base_nibble)) return 0;
  if (!read_32(fp, &saturn.disp_addr)) return 0;
  if (!read_16(fp, &saturn.line_offset)) return 0;
  if (!read_8(fp, &saturn.line_count)) return 0;
  if (!read_16(fp, &saturn.unknown)) return 0;
  if (!read_8(fp, &saturn.t1_ctrl)) return 0;
  if (!read_8(fp, &saturn.t2_ctrl)) return 0;
  if (!read_32(fp, &saturn.menu_addr)) return 0;
  if (!read_8(fp, &saturn.unknown2)) return 0;
  if (!read_char(fp, &saturn.timer1)) return 0;
  if (!read_32(fp, &saturn.timer2)) return 0;
  if (!read_32(fp, &saturn.t1_instr)) return 0;
  if (!read_32(fp, &saturn.t2_instr)) return 0;
  if (!read_16(fp, (word_16 *)&saturn.t1_tick)) return 0;
  if (!read_16(fp, (word_16 *)&saturn.t2_tick)) return 0;
  if (!read_32(fp, &saturn.i_per_s)) return 0;
  if (!read_16(fp, &saturn.bank_switch)) return 0;
  for (i = 0; i < NR_MCTL; i++) {
    if (!read_16(fp, &saturn.mem_cntl[i].unconfigured)) return 0;
    if (!read_32(fp, &saturn.mem_cntl[i].config[0])) return 0;
    if (!read_32(fp, &saturn.mem_cntl[i].config[1])) return 0;
  }
  return 1;
}

int read_mem_file(char *name, word_4 *mem, int size) {
  struct stat st;
  FILE *fp;
  word_8 *tmp_mem;
  word_8 byte;
  int i, j;
  if (NULL == (fp = fopen(name, "r"))) { LOGE("can't open %s\n", name); return 0; }
  if (stat(name, &st) < 0) { LOGE("can't stat %s\n", name); return 0; }
  if (st.st_size == size) {
    if (fread(mem, 1, (size_t)size, fp) != size) { fclose(fp); return 0; }
  } else {
    if (st.st_size != size / 2) { fclose(fp); return 0; }
    if (NULL == (tmp_mem = (word_8 *)malloc((size_t)st.st_size))) {
      for (i = 0, j = 0; i < size / 2; i++) {
        if (1 != fread(&byte, 1, 1, fp)) { fclose(fp); return 0; }
        mem[j++] = (word_4)((int)byte & 0xf);
        mem[j++] = (word_4)(((int)byte >> 4) & 0xf);
      }
    } else {
      if (fread(tmp_mem, 1, (size_t)size / 2, fp) != size / 2) { fclose(fp); free(tmp_mem); return 0; }
      for (i = 0, j = 0; i < size / 2; i++) {
        mem[j++] = (word_4)((int)tmp_mem[i] & 0xf);
        mem[j++] = (word_4)(((int)tmp_mem[i] >> 4) & 0xf);
      }
      free(tmp_mem);
    }
  }
  fclose(fp);
  return 1;
}

int write_mem_file(char *name, word_4 *mem, int size) {
  FILE *fp;
  word_8 *tmp_mem;
  int i, j;
  if (NULL == (fp = fopen(name, "w"))) return 0;
  if (NULL == (tmp_mem = (word_8 *)malloc((size_t)size / 2))) {
    word_8 byte;
    for (i = 0, j = 0; i < size / 2; i++) {
      byte = (mem[j++] & 0x0f);
      byte |= (mem[j++] << 4) & 0xf0;
      if (1 != fwrite(&byte, 1, 1, fp)) { fclose(fp); return 0; }
    }
  } else {
    for (i = 0, j = 0; i < size / 2; i++) {
      tmp_mem[i] = (mem[j++] & 0x0f);
      tmp_mem[i] |= (mem[j++] << 4) & 0xf0;
    }
    if (fwrite(tmp_mem, 1, (size_t)size / 2, fp) != size / 2) { fclose(fp); free(tmp_mem); return 0; }
    free(tmp_mem);
  }
  fclose(fp);
  return 1;
}

int read_files(void) {
  char path[1024], fnam[1024];
  unsigned long v1, v2;
  int i, read_version, ram_sz;
  struct stat st;
  FILE *fp;

  strcpy(path, files_path);
  saturn.rom = (word_4 *)NULL;
  strcpy(fnam, path); strcat(fnam, rom_filename);
  if (!read_rom_file(fnam, &saturn.rom, &rom_size)) return 0;
  rom_is_new = 0;

  strcpy(fnam, path); strcat(fnam, conf_filename);
  if (NULL == (fp = fopen(fnam, "r"))) { LOGE("can't open %s\n", fnam); return 0; }

  read_u_long(fp, &saturn.magic);
  if (X48_MAGIC != saturn.magic) {
    fseek(fp, 0, SEEK_SET);
    if (fread((char *)&old_saturn, 1, sizeof(old_saturn), fp) == sizeof(old_saturn)) {
      copy_old_saturn(&old_saturn, &saturn);
      saturn.magic = X48_MAGIC;
      saturn.t1_tick = 8192; saturn.t2_tick = 16; saturn.i_per_s = 0;
      saturn.version[0] = VERSION_MAJOR; saturn.version[1] = VERSION_MINOR;
      saturn.version[2] = PATCHLEVEL; saturn.version[3] = COMPILE_VERSION;
    } else { init_saturn(); }
  } else {
    read_version = 1;
    for (i = 0; i < 4; i++) if (!read_char(fp, &saturn.version[i])) read_version = 0;
    if (read_version) {
      v1 = ((int)saturn.version[0] & 0xff) << 24;
      v1 |= ((int)saturn.version[1] & 0xff) << 16;
      v1 |= ((int)saturn.version[2] & 0xff) << 8;
      v1 |= ((int)saturn.version[3] & 0xff);
      v2 = ((int)VERSION_MAJOR & 0xff) << 24;
      v2 |= ((int)VERSION_MINOR & 0xff) << 16;
      v2 |= ((int)PATCHLEVEL & 0xff) << 8;
      v2 |= ((int)COMPILE_VERSION & 0xff);
      if (v1 < 0x00040000) {
        if (!read_version_0_3_0_file(fp)) init_saturn();
        else copy_0_3_0_saturn(&saturn_0_3_0, &saturn);
      } else {
        if (!read_version_0_4_0_file(fp)) init_saturn();
      }
    }
  }
  fclose(fp);
  dev_memory_init();
  saturn_config_init();

  ram_sz = opt_gx ? RAM_SIZE_GX : RAM_SIZE_SX;
  saturn.ram = (word_4 *)NULL;
  if (NULL == (saturn.ram = (word_4 *)malloc(ram_sz))) { LOGE("can't malloc RAM\n"); return 0; }

  strcpy(fnam, path); strcat(fnam, ram_filename);
  if (!read_mem_file(fnam, saturn.ram, ram_sz)) return 0;

  saturn.card_status = 0;
  port1_size = 0; port1_mask = 0; port1_is_ram = 0; saturn.port1 = (unsigned char *)0;
  port2_size = 0; port2_mask = 0; port2_is_ram = 0; saturn.port2 = (unsigned char *)0;

  return 1;
}

int write_files(void) {
  char path[1024], fnam[1024];
  struct stat st;
  int i, ram_sz;
  FILE *fp;

  strcpy(path, files_path);
  if (stat(path, &st) == -1) {
    if (errno == ENOENT) mkdir(path, 0777);
  }

  strcpy(fnam, path); strcat(fnam, conf_filename);
  LOGI("trying to save: %s\n", fnam);
  if ((fp = fopen(fnam, "w")) == NULL) { LOGE("can't open %s\n", fnam); return 0; }

  write_32(fp, (word_32 *)&saturn.magic);
  for (i = 0; i < 4; i++) write_char(fp, &saturn.version[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.A[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.B[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.C[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.D[i]);
  write_32(fp, &saturn.d[0]); write_32(fp, &saturn.d[1]);
  write_8(fp, &saturn.P); write_32(fp, &saturn.PC);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.R0[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.R1[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.R2[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.R3[i]);
  for (i = 0; i < 16; i++) write_8(fp, &saturn.R4[i]);
  for (i = 0; i < 4; i++) write_8(fp, &saturn.IN[i]);
  for (i = 0; i < 3; i++) write_8(fp, &saturn.OUT[i]);
  write_8(fp, &saturn.CARRY);
  for (i = 0; i < NR_PSTAT; i++) write_8(fp, &saturn.PSTAT[i]);
  write_8(fp, &saturn.XM); write_8(fp, &saturn.SB); write_8(fp, &saturn.SR); write_8(fp, &saturn.MP);
  write_8(fp, &saturn.hexmode);
  for (i = 0; i < NR_RSTK; i++) write_32(fp, &saturn.rstk[i]);
  write_16(fp, (word_16 *)&saturn.rstkp);
  for (i = 0; i < 9; i++) write_16(fp, (word_16 *)&saturn.keybuf.rows[i]);
  write_8(fp, &saturn.intenable); write_8(fp, &saturn.int_pending); write_8(fp, &saturn.kbd_ien);
  write_8(fp, &saturn.disp_io); write_8(fp, &saturn.contrast_ctrl); write_8(fp, &saturn.disp_test);
  write_16(fp, &saturn.crc);
  write_8(fp, &saturn.power_status); write_8(fp, &saturn.power_ctrl); write_8(fp, &saturn.mode);
  write_8(fp, &saturn.annunc); write_8(fp, &saturn.baud);
  write_8(fp, &saturn.card_ctrl); write_8(fp, &saturn.card_status);
  write_8(fp, &saturn.io_ctrl); write_8(fp, &saturn.rcs); write_8(fp, &saturn.tcs);
  write_8(fp, &saturn.rbr); write_8(fp, &saturn.tbr); write_8(fp, &saturn.sreq);
  write_8(fp, &saturn.ir_ctrl); write_8(fp, &saturn.base_off);
  write_8(fp, &saturn.lcr); write_8(fp, &saturn.lbr);
  write_8(fp, &saturn.scratch); write_8(fp, &saturn.base_nibble);
  write_32(fp, &saturn.disp_addr); write_16(fp, &saturn.line_offset); write_8(fp, &saturn.line_count);
  write_16(fp, &saturn.unknown); write_8(fp, &saturn.t1_ctrl); write_8(fp, &saturn.t2_ctrl);
  write_32(fp, &saturn.menu_addr); write_8(fp, &saturn.unknown2);
  write_char(fp, &saturn.timer1); write_32(fp, &saturn.timer2);
  write_32(fp, &saturn.t1_instr); write_32(fp, &saturn.t2_instr);
  write_16(fp, (word_16 *)&saturn.t1_tick); write_16(fp, (word_16 *)&saturn.t2_tick);
  write_32(fp, &saturn.i_per_s); write_16(fp, &saturn.bank_switch);
  for (i = 0; i < NR_MCTL; i++) {
    write_16(fp, &saturn.mem_cntl[i].unconfigured);
    write_32(fp, &saturn.mem_cntl[i].config[0]);
    write_32(fp, &saturn.mem_cntl[i].config[1]);
  }
  fclose(fp);

  if (rom_is_new) {
    strcpy(fnam, path); strcat(fnam, rom_filename);
    if (!write_mem_file(fnam, saturn.rom, rom_size)) return 0;
  }

  ram_sz = opt_gx ? RAM_SIZE_GX : RAM_SIZE_SX;
  strcpy(fnam, path); strcat(fnam, ram_filename);
  if (!write_mem_file(fnam, saturn.ram, ram_sz)) return 0;

  return 1;
}

int read_rom(const char *fname) {
  int ram_sz;
  if (!read_rom_file(fname, &saturn.rom, &rom_size)) return 0;
  dev_memory_init();
  ram_sz = opt_gx ? RAM_SIZE_GX : RAM_SIZE_SX;
  if (NULL == (saturn.ram = (word_4 *)malloc(ram_sz))) return 0;
  memset(saturn.ram, 0, ram_sz);
  port1_size = 0; port1_mask = 0; port1_is_ram = 0; saturn.port1 = (unsigned char *)0;
  port2_size = 0; port2_mask = 0; port2_is_ram = 0; saturn.port2 = (unsigned char *)0;
  saturn.card_status = 0;
  return 1;
}

void get_home_directory(char *path) {
  /* Simplified for Emscripten - just use homeDirectory directly */
  if (homeDirectory[0] == '/') {
    strcpy(path, homeDirectory);
  } else {
    char *p = getenv("HOME");
    if (p) { strcpy(path, p); strcat(path, "/"); }
    else { strcpy(path, "/tmp"); }
    strcat(path, homeDirectory);
  }
}

int init_emulator(void) {
  if (!initialize)
    if (read_files()) {
      if (resetOnStartup) saturn.PC = 0x00000;
      return 0;
    }
  init_saturn();
  char path[1024];
  strcpy(path, files_path);
  strcat(path, rom_filename);
  if (!read_rom(path)) { LOGE("Failed to load ROM\n"); return -1; }
  return 0;
}

void init_active_stuff(void) {
  serial_init();
  init_annunc();
  init_display();
}

int exit_emulator(void) {
  write_files();
  return 1;
}
