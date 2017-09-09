/**
 * Lock-free multi-producer and multi-consumer ring buffer
 *
 * Ring buffer implementation based on
 *
 *  1. LinuxJournal article, A. Krizhanovsky, Lock-free multi-producer
 *     multi-consumer queue on ring buffer. github.com/krizhanovsky/NatSys-Lab
 *  2. M. Thompson D. Farley, M.Barker, P. Gee, A. Stewart, Disruptor: High
 *     performance alternative to  bounded queues for exchanging data between
 *     concurrent threads. github.com/LMAX-Exchange/disruptor
 *
 *
 */
#ifndef _RING_H_
#define _RING_H_

#ifndef __x86_64__
  #warning "ring buffer implementation requires an x86_64 architecture"
#endif


#include <limits>
#include <cstring>
#include <stdlib.h>
#include <thread>

#include "macros.h"

#define DEFAULT_R_SIZE  ( 32 * 1024 )
#define size_r unsigned long
#define SIZE_R_MAX std::numeric_limits<size_r>::max()

namespace utils {

  /**
   * We need user-controlled thread ids because POSIX threads do not
   * guarantee monotonically increasing values for thread ids (with
   * pthread_self()).
   *
   * The __thr_id thread local variable provides array indexing for algorithms
   * that need to store per thread data and respect a single writer principle.
   * Using this strategy implies defining an array v where the location v[i]
   * is owned (written by) the thread with __thr_id = i
   *
   * For a ring buffer, the ids are used for indexing the various producer and
   * consumer threads. Because we keep separate arrays for producers and
   * consumers, the ids have to be unique only amongst threads of the same
   * type (e.g. all producer threads should have unique ids).
   *
   * As a setup step, each producer/consumer thread should call set_thread_id
   * before doing operations on the ring buffer.
   *
   */
  static thread_local size_t __thr_id;

  // returns the current thread-specific id
  inline size_t get_thr_id() {
    return __thr_id;
  }

  // sets the current thread-specific id
  inline void set_thr_id(size_t id) {
    __thr_id = id;
  }

  /**
   * Ring buffer class
   *
   * The public interface of this class is threadsafe as long as each thread
   * sets an unique id which will function as an array index for the internal
   * thread-specific ring data members. This id should be retrievable using the
   * ThrId template parameter (see the default implementation for details).
   *
   * In the default ThrId implementation, each thread needs to call
   * utils::set_thr_id(size_t id). Using ids in a continuous range that starts
   * with 0 is both cache and memory friendly.
   *
   * --- Template parameters
   *
   *  T         The type of elements that are stored in the ring buffer
   *
   *  R_SIZE    The size of the ring buffer, in bytes. This needs to be
   *            a power of two, so that we can wrap around the buffer without
   *            using % operations
   *
   *  ThrId     A function of type size_t (func*)(void) which returns an
   *            indexable id for the current thread (small integer,
   *            monotonically increasing between threads of the same type
   *            i.e. producers/consumers)
   */
  template< typename T,
            size_r R_SIZE = DEFAULT_R_SIZE,
            decltype(get_thr_id) ThrId = get_thr_id >
  class Ring {
   private:
      static const size_r R_MASK = R_SIZE - 1;


   public:

      typedef T type;

      /**
       * \param max_prod_thrid specifies the maximum allowable value for
       * a producer ThrId. The ring buffer user needs to ensure that no
       * producer thread id has a value larger than this.
       *
       * \param max_cons_thrid specifies the maximum allowable value for
       * a consumer ThrId. The ring buffer user needs to ensure that no
       * consumer thread id has a value larger than this.
       */
      Ring(size_t max_prod_thrid, size_t max_cons_thrid);
      ~Ring();

      /* TODO(lc525): implement disruptor-like interface
       *
       * size_r getNext();
       * void commit(size_r entry);
       *
       */

      /**
       * As a producer, reserve a number of slots in the buffer. Make them
       * all available to a consumer at the same time by calling release
       */
      void reserve(size_t nr_slots);

      /**
       * As a producer, release the reserved slots to consumers.
       */
      void release();

      /**
       * Add an element to the ring buffer (from a producer thread). This
       * method is threadsafe. Do not use if you have reserved slots in the
       * buffer and you haven't called release yet. Use push_reserved instead.
       * \param ptr the element to add
       */
      void push(T* ptr);

      /**
       * Add an element into a reserved ring buffer slot. This needs to be
       * called after reserve(..) and before release(..)
       */
      bool push_reserved(T* ptr, size_t slot);

      /**
       * Rmove element from the ring buffer (from a consumer thread). This
       * method is threadsafe.
       */
      T* pop();

      /**
       * Returns the number of elements that are in the ring. If no producers
       * or consumers are active, the returned value is accurate. Otherwise, it
       * should be interpreted as an approximation of the number of elements
       * not touched by any consumer thread, but possibly including some
       * elements currently being written by producers.
       */
      size_r getNrElementsInRing(){
        return head_ - tail_;
      }

   private:

      const size_t max_prod_thrid_, max_cons_thrid_;

      // head_ - the writing end of the ring, points to the next free position
      size_r head_         ___cacheline_aligned;

      // tail_ - the reading end of the ring, points to the next available
      //         element
      size_r tail_         ___cacheline_aligned;

       // first_head_ - the first producer on the writing end, consumers should
       //               not attempt reading after this position
      size_r first_head_   ___cacheline_aligned;

       // last_tail_ - the last consumer on the reading end (the slowest).
       //              producers should not attempt writing after this position
      size_r last_tail_    ___cacheline_aligned;

      // pos_thread_ is an array indexing the current position in the ring for
      // each producer and consumer. The indexing is done by current ThrId.
      //
      // Within pos_thread_, we first store the indexes for all producers and
      // then the indexes for all consumers. pos_prod_ and pos_cons point to
      // the relevant positions within the pos_thread array.
      //
      // pos_thread_'s memory is aligned on page boundary on allocation, in
      // the Ring constructor
      size_r *pos_thread_;
      size_r *pos_prod_, *pos_cons_;


      // Actual buffer storage. The memory is aligned on page boundary on
      // allocation, in the Ring constructor
      T **ring_buf_;

      // res_prod_ is another array indexed by current ThrId, maintaining the
      // number of slots reserved by the respective producer
      size_r *res_prod_;
  };

  /**
   * -------------------------  Ring implementation  -------------------------
   */

  // Ring::Ring()
  //
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  Ring<T, R_SIZE, ThrId>::Ring(size_t max_prod_thrid, size_t max_cons_thrid)
    : max_prod_thrid_(max_prod_thrid),
      max_cons_thrid_(max_cons_thrid),
      head_(0), tail_(0), first_head_(0), last_tail_(0) {

    ::posix_memalign((void**)&pos_thread_,
                     PAGE_SIZE,
                     sizeof(size_r) * (max_prod_thrid_ + max_cons_thrid_ + 2));
    assert(pos_thread_);

    std::memset((void*)pos_thread_,
             0xFF,
             sizeof(size_r) * (max_prod_thrid_ + max_cons_thrid_ + 2));
    pos_prod_ = pos_thread_;
    pos_cons_ = pos_thread_ + max_prod_thrid_ + 1;

    ::posix_memalign((void**)&res_prod_,
                     PAGE_SIZE,
                     sizeof(size_r) * max_prod_thrid_);
    std::memset((void*)res_prod_, 0x0, sizeof(size_r) * max_prod_thrid);

    ::posix_memalign((void**)&ring_buf_,
                     PAGE_SIZE,
                     sizeof(void*) * R_SIZE);
    assert(ring_buf_);
  }

  // Ring::~Ring()
  //
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  Ring<T, R_SIZE, ThrId>::~Ring() {
    ::free(ring_buf_);
    ::free(pos_thread_);
  }

  // void reserve(size_t nr_slots);
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  void Ring<T, R_SIZE, ThrId>::reserve(size_t nr_slots) {
    assert(ThrId() <= max_prod_thrid_);
    pos_prod_[ThrId()] = head_;
    pos_prod_[ThrId()] = __sync_fetch_and_add(&head_, nr_slots);
    // wait while ring buffer is full
    while(unlikely(pos_prod_[ThrId()] >= last_tail_ + R_SIZE)) {
      std::this_thread::yield();
      //::usleep(10);

      size_r min = tail_;

      // update last_tail_;
      for(size_t i = 0; i < max_cons_thrid_; ++i) {
        size_r tmp_t = pos_cons_[i];

        asm volatile("" ::: "memory");

        if(tmp_t < min)
          min = tmp_t;
      }

      last_tail_ = min;
    }

    res_prod_[ThrId()] = nr_slots;
  }

  // void release();
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  void Ring<T, R_SIZE, ThrId>::release() {
    assert(ThrId() <= max_prod_thrid_);
    pos_prod_[ThrId()] = SIZE_R_MAX;
    res_prod_[ThrId()] = 0;
  }

  // void Ring::push(T* ptr)
  //
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  void Ring<T, R_SIZE, ThrId>::push(T* ptr) {
    assert(ThrId() <= max_prod_thrid_);

    pos_prod_[ThrId()] = head_;
    pos_prod_[ThrId()] = __sync_fetch_and_add(&head_, 1);

    // wait while ring buffer is full
    while(unlikely(pos_prod_[ThrId()] >= last_tail_ + R_SIZE)) {
      std::this_thread::yield();
      //::usleep(10);

      size_r min = tail_;

      // update last_tail_;
      for(size_t i = 0; i < max_cons_thrid_; ++i) {
        size_r tmp_t = pos_cons_[i];

        asm volatile("" ::: "memory");

        if(tmp_t < min)
          min = tmp_t;
      }

      // (*) Multiple threads might try to set last_tail_ simultanously, and
      // they might have determined different minimums in the array scan above.
      // Normally, this should be a synchronized CAS operation, but that
      // is expensive.
      //
      // We observe that although the value assigned here might not be the
      // "true" minimum, in the worst case scenario last_tail_ will end up
      // being smaller than it, thus sometimes forcing another pass through
      // the while loop. (the corectness is preserved)
      last_tail_ = min;
    }

    ring_buf_[ pos_prod_[ThrId()] & R_MASK ] = ptr;

    // consumers can process the added item
    pos_prod_[ThrId()] = SIZE_R_MAX;
  }

  //void push_reserved(T* ptr, size_t slot);
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  bool Ring<T, R_SIZE, ThrId>::push_reserved(T* ptr, size_t slot) {
    if(slot < res_prod_[ThrId()]){
      ring_buf_[ pos_prod_[ThrId() + slot] & R_MASK ] = ptr;
      return true;
    }
    return false;
  }

  // T* Ring::pop()
  //
  template< typename T, size_r R_SIZE, decltype(get_thr_id) ThrId>
  T* Ring<T, R_SIZE, ThrId>::pop() {
    assert(ThrId() <= max_cons_thrid_);

    pos_cons_[ThrId()] = tail_;
    pos_cons_[ThrId()] = __sync_fetch_and_add(&tail_, 1);

    // wait while ring buffer is empty
    while(unlikely(pos_cons_[ThrId()] >= first_head_)) {
      std::this_thread::yield();

      size_r min = head_;

      // update first_head_
      for(size_t i = 0; i < max_prod_thrid_; ++i) {
        size_r tmp_h = pos_prod_[i];

        asm volatile("" ::: "memory");

        if(tmp_h < min)
          min = tmp_h;
      }

      // This should be a CAS (expensive), but having first_head_ set to an
      // old(er) value does not compromise corectness; it just sometimes forces
      // another pass through the while loop. See the corresponding comment
      // (*) in the push function for more details.
      first_head_ = min;
    }

    T* ret = ring_buf_[ pos_cons_[ThrId()] & R_MASK ];

    // allow producers to overwrite the slot
    pos_cons_[ThrId()] = SIZE_R_MAX;
    return ret;

  }

}

#endif
