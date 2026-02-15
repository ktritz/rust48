#ifndef _GLOBAL_H
#define _GLOBAL_H 1

#include "config.h"

#ifdef __ProtoType__
#undef __ProtoType__
#endif

#ifdef __FunctionProto__
#undef __FunctionProto__
#endif

#if defined(__STDC__) || defined(__cplusplus)
#define __ProtoType__(x) x
#define __FunctionProto__ 1
#else
#define __ProtoType__(x) ()
#undef __FunctionProto__
#endif

#if !defined(__GNUC__) || defined(__STRICT_ANSI__)
#define inline
#if !defined(__STDC__)
#define const
#endif
#endif

#ifdef __EMSCRIPTEN__
/* Emscripten: use simple gettimeofday (no timezone struct needed) */
#define SYSV_TIME 1
#endif

#if defined(linux) && !defined(__EMSCRIPTEN__)
#ifndef LINUX
#define LINUX 1
#endif
#define SYSV_TIME 1
#endif

#ifdef SUNOS
#undef HAVE_STDIO
#else
#define HAVE_STDIO 1
#endif

#ifndef HAVE_STDIO
#include <stdio.h>
#include <sys/time.h>
#include <sys/types.h>
extern int      printf		__ProtoType__((char *, ...));
extern int      fprintf		__ProtoType__((FILE *, char *, ...));
extern int	sscanf		__ProtoType__((char *, char *, ...));
extern void     fflush		__ProtoType__((FILE *));
extern int      fseek		__ProtoType__((FILE *, long, int));
extern int      fread		__ProtoType__((void *, int, int, FILE*));
extern int      fwrite		__ProtoType__((void *, int, int, FILE*));
extern void     fclose		__ProtoType__((FILE *));
extern int	fgetc		__ProtoType__((FILE *));
extern void     bzero		__ProtoType__((void *, int));
extern time_t	time		__ProtoType__((time_t *));
extern int      select		__ProtoType__((int, fd_set *, fd_set *,
                                               fd_set *, struct timeval *));
extern int      setitimer	__ProtoType__((int, struct itimerval *,
                                               struct itimerval *));
extern int	gethostname	__ProtoType__((char *, int));
#endif

#endif /* !_GLOBAL_H */
