/*
 * resources.c - Modified for Emscripten web build.
 * Set initialize=0 so read_files() is called (not fresh init).
 * Set homeDirectory="/assets/".
 * Removed disasm.h dependency.
 */

#include "global.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "resources.h"
#include "errors.h"

int	verbose;
int	quiet;
int     useTerminal;
int     useSerial;
char   *serialLine;
int     useXShm;
int     useDebugger;
int     initialize;
int     resetOnStartup;
char   *romFileName;
char   *homeDirectory;

/* Stub: disassembler_mode referenced by resources but excluded from build */
int disassembler_mode;

void
#ifdef __FunctionProto__
get_resources(void)
#else
get_resources()
#endif
{
  verbose = 0;
  quiet = 0;
  useXShm = 0;
  useTerminal = 0;
  useSerial = 0;
  serialLine = 0;

  initialize = 0;   /* Load existing files, don't fresh-init */
  resetOnStartup = 0;
  romFileName = "rom";
  homeDirectory = "/assets/";

  useDebugger = 0;
  disassembler_mode = 0;
}
