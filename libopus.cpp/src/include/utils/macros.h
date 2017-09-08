/**
 * This header file defines a series of macros used across the project.
 * Some of those define optimisations and depend on GCC extensions.
 */
#ifndef _MACROS_H_
#define _MACROS_H_


/*
 * Define cache alignment atributes for class members where false sharing
 * should be avoided (i.e each memeber should be placed in a different cache
 * line). Compile this source with
 *
 * -D DCACHE_L1_LINESIZE=`getconf LEVEL1_DCACHE_LINESIZE`
 */
#if !defined( DCACHE_L1_LINESIZE ) || !DCACHE_L1_LINESIZE
  #ifdef DCACHE_L1_LINESIZE
    #undef DCACHE_L1_LINESIZE
  #endif
  #define ___cacheline_aligned    __attribute__((aligned(64)))
#else
  #define ___cacheline_aligned    __attribute__((aligned(DCACHE_L1_LINESIZE)))
#endif

/*
 * GCC branch hints macros. Those are similar to what is defined in
 * linux/compiler.h (for the linux kernel).
 *
 * The hints allow GCC to optimize the ordering of generated assembly code in
 * a way that optimizes the use of the processor pipeline. The likeliest branch
 * is executed without any jmp instruction.
 */
#ifdef __GNUC__
  // double negation used for conversion to bool
  #define likely(x)       __builtin_expect(!!(x), 1)
  #define unlikely(x)     __builtin_expect(!!(x), 0)
#else
  #define likely(x)       (x)
  #define unlikely(x)     (x)
#endif

#define PAGE_SIZE 4096

#endif
