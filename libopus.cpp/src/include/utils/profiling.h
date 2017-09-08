#ifndef _APP_PROFILING_H_
#define _APP_PROFILING_H_

/**
 * Configure profiling settings
 */

/** APP_PROFILE
 *
 * Global switch for profiling. Comment out this define to disable application
 * profiling (all calls to profiling macros will be turned into NOOPS).
 * With profiling turned off, your application will incurr no time/space
 * overheads
 **
 * With this define commented out, all the other configuration defines will
 * have no effects.
 */
#define APP_PROFILE

#define PROFILE_WCT

//#define PROFILE_CPUT


/****/

#include "profiling_impl.h"

#endif /* _APP_PROFILING_H_ */
