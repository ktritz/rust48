/*
 * hp48_emu.h - Modified for Emscripten web build.
 * Removed X11 Display/Window/GC externs.
 */

#ifndef _HP48_EMU_H
#define _HP48_EMU_H 1

#include "global.h"
#include "x48.h"
#include "hp48.h"

extern void		push_return_addr __ProtoType__((long addr));
extern long		pop_return_addr __ProtoType__((void));

extern void		init_annunc __ProtoType__((void));

extern void		init_saturn __ProtoType__((void));

extern void		check_timer __ProtoType__((void));

extern void		register_to_status __ProtoType__((unsigned char *r));
extern void		status_to_register __ProtoType__((unsigned char *r));
extern void		swap_register_status __ProtoType__((unsigned char *r));
extern void		clear_status __ProtoType__((void));

extern long		read_nibbles __ProtoType__((long addr, int len));
extern void		write_nibbles __ProtoType__((long addr, long val, int len));
extern void		dev_memory_init __ProtoType__((void));

extern void		set_program_stat __ProtoType__((int n));
extern void		clear_program_stat __ProtoType__((int n));
extern int		get_program_stat __ProtoType__((int n));

extern void		set_hardware_stat __ProtoType__((int op));
extern void		clear_hardware_stat __ProtoType__((int op));
extern int		is_zero_hardware_stat __ProtoType__((int op));

extern void		set_register_bit __ProtoType__((unsigned char *reg, int n));
extern void		clear_register_bit __ProtoType__((unsigned char *reg, int n));
extern int		get_register_bit __ProtoType__((unsigned char *reg, int n));

extern void		set_register_nibble __ProtoType__((unsigned char *reg, int n,
                                            unsigned char val));
extern unsigned char	get_register_nibble __ProtoType__((unsigned char *reg, int n));


extern void		register_to_address __ProtoType__((unsigned char *reg,
                                            word_20 *dat, int s));
extern void		address_to_register __ProtoType__((word_20 dat,
                                            unsigned char *reg, int s));
extern void		add_address __ProtoType__((word_20 *dat, int add));

extern char *		make_hexstr __ProtoType__((long addr, int n));
extern void		load_constant __ProtoType__((unsigned char *reg, int n,
                                      long addr));
extern void		load_address __ProtoType__((unsigned char *reg, long addr,
                                     int n));

extern void		store __ProtoType__((word_20 dat, unsigned char *reg,
                              int code));
extern void		store_n __ProtoType__((word_20 dat, unsigned char *reg,
                                int n));
extern void		recall __ProtoType__((unsigned char *reg, word_20 dat,
                               int code));
extern void		recall_n __ProtoType__((unsigned char *reg, word_20 dat,
			 int n));

extern long		dat_to_addr __ProtoType__((unsigned char *dat));
extern void		addr_to_dat __ProtoType__((long addr, unsigned char *dat));

extern void		do_in __ProtoType__((void));
extern void		do_reset __ProtoType__((void));
extern void		do_configure __ProtoType__((void));
extern void		do_unconfigure __ProtoType__((void));
extern void		do_inton __ProtoType__((void));
extern void		do_intoff __ProtoType__((void));
extern void		do_return_interupt __ProtoType__((void));
extern void		do_reset_interrupt_system __ProtoType__((void));
extern void		do_shutdown __ProtoType__((void));
extern int		get_identification __ProtoType__((void));

extern void		add_p_plus_one __ProtoType__((unsigned char *r));
extern void		add_register_constant __ProtoType__((unsigned char *res,
                                              int code, int val));
extern void		sub_register_constant __ProtoType__((unsigned char *res,
                                              int code, int val));
extern void		add_register __ProtoType__((unsigned char *res, unsigned char *r1,
                                     unsigned char *r2, int code));
extern void		sub_register __ProtoType__((unsigned char *res, unsigned char *r1,
                                     unsigned char *r2, int code));
extern void		complement_2_register __ProtoType__((unsigned char *r, int code));
extern void		complement_1_register __ProtoType__((unsigned char *r, int code));
extern void		inc_register __ProtoType__((unsigned char *r, int code));
extern void		dec_register __ProtoType__((unsigned char *r, int code));
extern void		zero_register __ProtoType__((unsigned char *r, int code));
extern void		or_register __ProtoType__((unsigned char *res, unsigned char *r1,
                                    unsigned char *r2, int code));
extern void		and_register __ProtoType__((unsigned char *res, unsigned char *r1,
                                     unsigned char *r2, int code));
extern void		copy_register __ProtoType__((unsigned char *to, unsigned char *from,
                                      int code));
extern void		exchange_register __ProtoType__((unsigned char *r1, unsigned char *r2,
                                          int code));

extern void		exchange_reg __ProtoType__((unsigned char *r, word_20 *d, int code));

extern void		shift_left_register __ProtoType__((unsigned char *r, int code));
extern void		shift_left_circ_register __ProtoType__((unsigned char *r, int code));
extern void		shift_right_register __ProtoType__((unsigned char *r, int code));
extern void		shift_right_circ_register __ProtoType__((unsigned char *r, int code));
extern void		shift_right_bit_register __ProtoType__((unsigned char *r, int code));
extern int		is_zero_register __ProtoType__((
					unsigned char *r,
					int code));
extern int		is_not_zero_register __ProtoType__((
					unsigned char *r,
					int code));
extern int		is_equal_register __ProtoType__((
					unsigned char *r1,
					unsigned char *r2,
                                          	int code));
extern int		is_not_equal_register __ProtoType__((
					unsigned char *r1,
                                              	unsigned char *r2,
					int code));
extern int		is_less_register __ProtoType__((
					unsigned char *r1,
					unsigned char *r2,
                                         	int code));
extern int		is_less_or_equal_register __ProtoType__((
					unsigned char *r1,
					unsigned char *r2,
                                                int code));
extern int		is_greater_register __ProtoType__((
					unsigned char *r1,
                                            	unsigned char *r2,
					int code));
extern int		is_greater_or_equal_register __ProtoType__((
					unsigned char *r1,
                                                unsigned char *r2,
                                                int code));

#endif /* !_HP48_EMU_H */
