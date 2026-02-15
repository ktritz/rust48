/* config.h - Emscripten-specific configuration for x48 web port */

#define COMPILE_BY "web"
#define COMPILE_TIME "2024"
#define COMPILE_VERSION 0

#define HAVE_BZERO 1
#define HAVE_FCNTL_H 1
#define HAVE_GETHOSTNAME 1
#define HAVE_GETTIMEOFDAY 1
#define HAVE_INTTYPES_H 1
#define HAVE_MALLOC 1
#define HAVE_MEMORY_H 1
#define HAVE_MEMSET 1
#define HAVE_MKDIR 1
#define HAVE_SELECT 1
#define HAVE_STDINT_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRDUP 1
#define HAVE_STRINGS_H 1
#define HAVE_STRING_H 1
#define HAVE_STRRCHR 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TIME_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_UNISTD_H 1
#define HAVE_STDIO 1

#define STDC_HEADERS 1
#define TIME_WITH_SYS_TIME 1

#define PACKAGE "x48"
#define PACKAGE_NAME "x48"
#define PACKAGE_STRING "x48 0.6.1"
#define PACKAGE_TARNAME "x48"
#define PACKAGE_VERSION "0.6.1"
#define PACKAGE_BUGREPORT ""

#define PATCHLEVEL 1
#define VERSION "0.6.1"
#define VERSION_MAJOR 0
#define VERSION_MINOR 6

/* Emscripten: no X11, no SHM, no Solaris, no SunOS */
/* #undef HAVE_XSHM */
/* #undef SUNOS */
/* #undef SOLARIS */
/* #undef SYSV */
/* #undef SYSV_TIME */

#ifndef __cplusplus
/* #undef inline */
#endif
