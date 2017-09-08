/* Time measurement & application profiling
 *
 * link app with -lrt
 */
#ifndef _APP_PROFILING_IMPL_H_
#define _APP_PROFILING_IMPL_H_

#include <time.h>
#include "time_ppd.h"

#ifdef APP_PROFILE

#include <iostream>
#include <iomanip>

timespec diff(timespec start, timespec end)
{
  timespec temp;
  if ((end.tv_nsec - start.tv_nsec) < 0) {
    temp.tv_sec = end.tv_sec - start.tv_sec - 1;
    temp.tv_nsec = 1e9 + end.tv_nsec - start.tv_nsec;
  } else {
    temp.tv_sec = end.tv_sec - start.tv_sec;
    temp.tv_nsec = end.tv_nsec - start.tv_nsec;
  }
  return temp;
}

std::ostream& operator<< (std::ostream &out, const timespec &ts){
  return (out << std::fixed << ts.tv_sec * 1e9 + ts.tv_nsec);
}

inline void print_timespec(timespec &t){
  double tv_ms = t.tv_nsec * 1e-6;
  std::cout<<std::fixed<<t.tv_sec<<":"<<tv_ms<<" (s:ms) "<<std::endl;
}

struct ptimer{
  const char* name;
  bool is_active;
  #ifdef PROFILE_WCT
    timespec wct_start, wct_end;
  #endif
  #ifdef PROFILE_CPUT
    timespec cput_start, cput_end;
  #endif

  ptimer() : is_active(false) {}
};

struct stats{
  double avg;
};

void _internal_ptimer_start(ptimer& t){
  if(t.is_active){

  }
  #ifdef PROFILE_WCT
    clock_gettime(WALL_CLOCK, &t.wct_start);
  #endif
  #ifdef PROFILE_CPUT
    clock_gettime(WALL_CLOCK, &t.cput_start);
  #endif
}

void _internal_ptimer_stop(ptimer& t){
  #ifdef PROFILE_WCT
    clock_gettime(WALL_CLOCK, &t.wct_end);
  #endif
  #ifdef PROFILE_CPUT
    clock_gettime(WALL_CLOCK, &t.cput_end);
  #endif
}

inline void _internal_bconfig_compute(){
  static int a[] = { 2424, 234, 234, 5, 45, 6, 657, 567, 567, 657, 56, 75, 6, 5677, 567, 567 };
  static int b[]= {24, 456, 7878, 34, 44, 123, 657, 6123, 223, 65, 65, 5, 6, 1122, 567, 765 };
  long sum=0;

  for(int i=0; i<16; i++){
    sum += a[i] * b[i];
  }
}

void _internal_print_bconfig(){
  std::cout<<"== Profiling Timers =="<<std::endl;
  int k=0;
  #ifdef PROFILE_WCT
    k++;
    std::cout<<"+ Wall clock: "<<WALL_CLOCKNAME<<std::endl;
  #endif
  #ifdef PROFILE_CPUT
    k++;
    std::cout<<"+ CPU clock: "<<CPU_CLOCKNAME<<std::endl;
  #endif
  if(k==0){
    std::cout<<"No profile timers enabled"<<std::endl;
  }
  else{
    // Timer benchmarks
    std::cout<<std::endl<<"= Statistics"<<std::endl;

    #ifdef PROFILE_WCT

    std::cout<<"+ "<<WALL_CLOCKNAME<<std::endl;
    ptimer wc;
    ptimer wc_ohead;
    float o_iter = 1e7;

    std::cout<<"\tOverhead for "<<std::setprecision(0)<<std::scientific<<o_iter<<" calls: ";
    for(long i=0; i<1e5; i++){
      _internal_bconfig_compute(); // warmup cache
    }

    clock_gettime(WALL_CLOCK, &wc_ohead.wct_start);
    for(long i=0; i<o_iter; i++){
      _internal_bconfig_compute();
    }
    clock_gettime(WALL_CLOCK, &wc_ohead.wct_end);
    timespec wct_baseline = diff(wc_ohead.wct_start, wc_ohead.wct_end);

    clock_gettime(WALL_CLOCK, &wc_ohead.wct_start);
    for(long i=0; i<o_iter; i++){
      clock_gettime(WALL_CLOCK, &wc.wct_start);
      _internal_bconfig_compute();
    }
    clock_gettime(WALL_CLOCK, &wc_ohead.wct_end);
    timespec wct_ohead = diff(wc_ohead.wct_start, wc_ohead.wct_end);
    timespec wct_dif = diff(wct_baseline, wct_ohead);
    print_timespec(wct_dif);
    #endif
    #ifdef PROFILE_CPUT
    #endif
  }

  std::cout<<"============="<<std::endl<<std::endl;
}

void _internal_print_benchmark(ptimer& t){
  #ifdef PROFILE_WCT
    timespec wct_tdif = diff(t.wct_start, t.wct_end);
    double wtv_ms = wct_tdif.tv_nsec * 1e-6;
    std::cout<<t.name<<"(wall_clock) elapsed "<<std::fixed<<wct_tdif.tv_sec<<":"<<wtv_ms<<" (s:ms) "<<std::endl;
  #endif
  #ifdef PROFILE_CPUT
    timespec cput_tdif = diff(t.cput_start, t.cput_end);
    double ctv_ms = cput_tdif.tv_nsec * 1e-6;
    std::cout<<t.name<<"(cpu_clock) elapsed "<<std::fixed<<cput_tdif.tv_sec<<":"<<ctv_ms<<" (s:ms) "<<std::endl;
  #endif
}

#endif /* APP_PROFILE */

#endif /* _APP_PROFILING_IMPL_H_ */
