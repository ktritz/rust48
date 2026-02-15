/*
 * debugger.h - Stub for web build (debugger excluded).
 * Provides the externs that other files reference, but with no-op implementations.
 */

#ifndef _DEBUGGER_H
#define _DEBUGGER_H 1

#include "global.h"
#include "hp48.h"

#define	USER_INTERRUPT		1
#define	ILLEGAL_INSTRUCTION	2
#define	BREAKPOINT_HIT		4
#define	TRAP_INSTRUCTION	8

#define EXEC_BKPT		1

extern int	enter_debugger;
extern int	in_debugger;
extern int	exec_flags;

/* Stubs - no debugger in web build */
static inline void init_debugger(void) {}
static inline int  debug(void) { return 0; }
static inline int  emulate_debug(void) { return 0; }

#endif /* !_DEBUGGER_H */
