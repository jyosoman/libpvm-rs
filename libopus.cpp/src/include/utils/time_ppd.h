/** Timer Preprocessor directives
 */
#ifndef _TIME_PPD_H_
#define _TIME_PPD_H_


#define STR_EXPAND(tok) #tok
#define STR(tok) STR_EXPAND(tok)

#ifdef CLOCK_MONOTONIC_RAW
  #define WALL_CLOCK CLOCK_MONOTONIC_RAW
  #define WALL_CLOCKNAME "CLOCK_MONOTONIC_RAW"
#else
  #define WALL_CLOCK CLOCK_REALTIME
  #define WALL_CLOCKNAME "CLOCK_REALTIME"
  #define WC_RT
#endif

#define CPU_CLOCK CLOCK_PROCESS_CPUTIME_ID
#define CPU_CLOCKNAME "CLOCK_PROCESS_CPUTIME_ID"

#ifdef APP_PROFILE
  #define CREATE_PTIMER(timer_name) ptimer timer_name; timer_name.name=#timer_name

  #define PTIMER_START(timer_name) _internal_ptimer_start(timer_name)
  #define PTIMER_STOP(timer_name) _internal_ptimer_stop(timer_name)
  #define PRINT_PTIMER(timer_name) _internal_print_benchmark(timer_name)
  #define PRINT_PROFILE_META  _internal_print_bconfig()

#else

  #define CREATE_PTIMER(timer_name)

  #define PTIMER_START(timer_name)
  #define PTIMER_STOP(timer_name)
  #define PRINT_PTIMER(timer_name)
  #define PRINT_PROFILE_META

#endif /* APP_PROFILE */

#endif /* _TIME_PPD_H_ */


/*
 *#if defined(_POSIX_TIMERS) && (_POSIX_TIMERS > 0)
 *   {
 *    struct timespec ts;
 *#if defined(CLOCK_MONOTONIC_PRECISE)
 *    [> BSD. --------------------------------------------- <]
 *    const clockid_t id = CLOCK_MONOTONIC_PRECISE;
 *#elif defined(CLOCK_MONOTONIC_RAW)
 *    [> Linux. ------------------------------------------- <]
 *    const clockid_t id = CLOCK_MONOTONIC_RAW;
 *#elif defined(CLOCK_HIGHRES)
 *    [> Solaris. ----------------------------------------- <]
 *    const clockid_t id = CLOCK_HIGHRES;
 *#elif defined(CLOCK_MONOTONIC)
 *    [> AIX, BSD, Linux, POSIX, Solaris. ----------------- <]
 *    const clockid_t id = CLOCK_MONOTONIC;
 *#elif defined(CLOCK_REALTIME)
 *    [> AIX, BSD, HP-UX, Linux, POSIX. ------------------- <]
 *    const clockid_t id = CLOCK_REALTIME;
 *#else
 *    const clockid_t id = (clockid_t)-1; [> Unknown. <]
 *#endif [> CLOCK_* <]
 *    if ( id != (clockid_t)-1 && clock_gettime( id, &ts ) != -1 )
 *      return (double)ts.tv_sec + (double)ts.tv_nsec / 1000000000.0;
 *    [> Fall thru. <]
 *  }
 *#endif [> _POSIX_TIMERS <]
 */
