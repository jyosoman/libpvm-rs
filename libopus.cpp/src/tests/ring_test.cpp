/* Ring Buffer Tests
 *
 * Note: Only the tests which have a name containing the word "Functional" will
 * be run by default (on make test or make check).
 */
#include "gtest/gtest.h"

#include <atomic>
#include <fstream>
#include <iostream>
#include <thread>

#include "utils/profiling.h"
#include "utils/ring.h"

static const unsigned long BUF_SIZE = 1024;
static const unsigned long N =  2 * 1024 * 512;
static const short NR_PRODUCERS = 2;
static const short NR_CONSUMERS = 2;

#define TEST_COMMENT "" //"1us between pushes of the same producer"

struct LatencyProbe {
  LatencyProbe() { }
  LatencyProbe(int init_payload) : payload(init_payload) { }

  int payload;
  timespec at_push, at_pop;

  bool operator==(const LatencyProbe& other) const {
    return payload == other.payload;
  }

  bool operator!=(const LatencyProbe& other) const {
    return payload != other.payload;
  }
};

struct CPayload {
  CPayload() { }
  CPayload(int init_payload) : payload(init_payload) { }

  int payload;

  bool operator==(const CPayload& other) const {
    return payload == other.payload;
  }

  bool operator!=(const CPayload& other) const {
    return payload != other.payload;
  }

};

template<typename T>
class buffer_test_param{};

template<typename T>
class buffer_test_events {
  public:
  buffer_test_events() {}

  void on_item_add(T*) const { }

  void on_item_consumed(size_t by_id, T* item) const {
    *item = (T)by_id;
  }
};

template<>
class buffer_test_param<unsigned char> {
  public:
  buffer_test_param(){} // allow const instances with default constructor

  // ERR_PROD_SKIP indicates a data item which was skipped by producers
  // The value should not appear in a correct buffer implementation
  const unsigned char ERR_PROD_SKIP = 0;
  // ERR_CONS_SKIP indicates a data item which was skipped by consumers
  // The value should not appear in a correct buffer implementation
  const unsigned char ERR_CONS_SKIP = 255;

};

template<>
class buffer_test_param<int> {
  public:
  buffer_test_param(){} // allow const instances with default constructor

  // ERR_PROD_SKIP indicates a data item which was skipped by producers
  // The value should not appear in a correct buffer implementation
  const int ERR_PROD_SKIP = -667;
  // ERR_CONS_SKIP indicates a data item which was skipped by consumers
  // The value should not appear in a correct buffer implementation
  const int ERR_CONS_SKIP = -666;

};

template<>
class buffer_test_param<LatencyProbe> {
  public:
  buffer_test_param(){} // allow const instances with default constructor

  // ERR_PROD_SKIP indicates a data item which was skipped by producers
  // The value should not appear in a correct buffer implementation
  const LatencyProbe ERR_PROD_SKIP = LatencyProbe(-1);
  // ERR_CONS_SKIP indicates a data item which was skipped by consumers
  // The value should not appear in a correct buffer implementation
  const LatencyProbe ERR_CONS_SKIP = LatencyProbe(-2);;

};

template<>
class buffer_test_param<CPayload> {
  public:
  buffer_test_param(){} // allow const instances with default constructor

  // ERR_PROD_SKIP indicates a data item which was skipped by producers
  // The value should not appear in a correct buffer implementation
  const CPayload ERR_PROD_SKIP = CPayload(-80);
  // ERR_CONS_SKIP indicates a data item which was skipped by consumers
  // The value should not appear in a correct buffer implementation
  const CPayload ERR_CONS_SKIP = CPayload(40);

  const CPayload NO_ERR = CPayload(25);

};


template<>
class buffer_test_events<unsigned char> {
  public:
  buffer_test_events(){} // allow const instances with default constructor

  void on_item_add(unsigned char*) const { }

  void on_item_consumed(size_t by_id, unsigned char* item) const {
    *item = by_id + 1;
  }

};

template<>
class buffer_test_events<LatencyProbe> {
  public:
  buffer_test_events(){} // allow const instances with default constructor

  void on_item_add(LatencyProbe* item) const {
    clock_gettime(WALL_CLOCK, &item->at_push);
  }

  void on_item_consumed(size_t by_id, LatencyProbe* item) const {
    clock_gettime(WALL_CLOCK, &item->at_pop);
    item->payload = by_id + 1;
  }

};

template<>
class buffer_test_events<CPayload> {
  public:
  buffer_test_events(){} // allow const instances with default constructor

  void on_item_add(CPayload* item) const {
    item->payload -= 80;
  }

  void on_item_consumed(size_t, CPayload* item) const {
    EXPECT_EQ(-40, item->payload);
    item->payload += 65;
  }

};


template <typename T>
class TestProperties {
  public:
    static const buffer_test_param<T> buf_param;
    static const buffer_test_events<T> buf_ev;

    // the array holding test data for all producers; each producer will insert
    // N elements into the buffer under test
    static T *prod_buf;
};

template <typename T>
const buffer_test_param<T> TestProperties<T>::buf_param;

template <typename T>
const buffer_test_events<T> TestProperties<T>::buf_ev;

template <typename T>
T *TestProperties<T>::prod_buf = new T[N * NR_PRODUCERS];

template<typename Buf>
class Worker {
  public:
    Worker(Buf *b, size_t thr_id)
      : b_(b),
        thr_id_(thr_id) { }

    Buf *b_;
    size_t thr_id_;
};

template<typename Buf>
class Producer : public Worker<Buf> {
  typedef TestProperties<typename Buf::type> TestProp;
  public:
    Producer(Buf *b, size_t id)
      : Worker<Buf>(b, id) {}

    void operator()() {
      utils::set_thr_id(this->thr_id_);

      for(auto i = this->thr_id_; i < N * NR_PRODUCERS; i += NR_PRODUCERS) {
        TestProp::prod_buf[i] = TestProp::buf_param.ERR_CONS_SKIP;
        TestProp::buf_ev.on_item_add(&TestProp::prod_buf[i]);
        this->b_->push(&TestProp::prod_buf[i]);
        //::usleep(1);
      }
    }
};


template<typename Buf>
class Consumer : public Worker<Buf> {
  typedef typename Buf::type buf_inner_type;
  typedef TestProperties<buf_inner_type> TestProp;
  public:
    static std::atomic<unsigned long> op_;

    Consumer(Buf *b, size_t id)
      : Worker<Buf>(b, id) {}

    void operator()() {
      utils::set_thr_id(this->thr_id_);

      while(op_.fetch_add(1) < N * NR_PRODUCERS) {
        buf_inner_type *v = this->b_->pop();
        TestProp::buf_ev.on_item_consumed(this->thr_id_, v);
      }
    }
};

// static initializer for Consumer::op_
template<typename Buf>
std::atomic<unsigned long> Consumer<Buf>::op_(0);


/** **
 * Ring buffer unit tests
 */
template<typename T>
class RingBufferTest : public ::testing::Test {
  public:
    utils::Ring<T, BUF_SIZE> ring_;
    std::thread thr_[NR_PRODUCERS + NR_CONSUMERS];

    RingBufferTest() : ring_(NR_PRODUCERS, NR_CONSUMERS) {}

    virtual void SetUp() {
      for( unsigned long i = 0; i < N * NR_PRODUCERS; ++i) {
        TestProperties<T>::prod_buf[i] =
          TestProperties<T>::buf_param.ERR_PROD_SKIP;
      }
      Consumer<utils::Ring<T, BUF_SIZE>>::op_=0; // reset operation count
    }

    virtual void RunProducersConsumers() {
        using namespace utils;

        // Run producers
        for(int i = 0; i < NR_PRODUCERS; ++i) {
          thr_[i] = std::thread(Producer<Ring<T, BUF_SIZE>>(&ring_, i));
        }

        //::usleep(10 * 1000); // Allow producers to fill some of the buffer

        // Run consumers
        for(int i = 0; i < NR_CONSUMERS; ++i) {
          thr_[NR_PRODUCERS + i] =
            std::thread(Consumer<Ring<T, BUF_SIZE>>(&ring_, i));
        }

        // Wait for all threads to complete
        for(int i = 0; i < NR_PRODUCERS + NR_CONSUMERS; ++i) {
          thr_[i].join();
        }
    }
};


class RingBufferCorectnessTest : public RingBufferTest<CPayload> {};

TEST_F(RingBufferCorectnessTest, FunctionalNProdMCons){
  using namespace utils;
  typedef TestProperties<CPayload> TestProp;

  this->RunProducersConsumers();

  // Check for correct number of operations
  ASSERT_EQ(N * NR_PRODUCERS + NR_CONSUMERS,
           (Consumer<Ring<CPayload, BUF_SIZE>>::op_.load()));

  // Check that all the elements in the ring were consumed
  ASSERT_EQ(0UL, this->ring_.getNrElementsInRing());

  // Check results.
  // If an item was skipped by a producer, it will have its value
  // ERR_PROD_SKIP
  // If an item was skipped by a consumer, it will have its value ERR_CONS_SKIP
  // If an item has been added and removed only once, it will have value 25
  //
  for(unsigned long i = 0; i < N * NR_PRODUCERS; ++i) {
    EXPECT_NE(TestProp::buf_param.ERR_CONS_SKIP, TestProp::prod_buf[i]);
    EXPECT_NE(TestProp::buf_param.ERR_PROD_SKIP, TestProp::prod_buf[i]);
    EXPECT_EQ(TestProp::buf_param.NO_ERR, TestProp::prod_buf[i]);
  }

}
/**
 * TestTypes defines a list of types whith which the ring buffer should be
 * tested with in the performance tests.
 */
typedef ::testing::Types<  unsigned char,
                           int             > TestTypes;
TYPED_TEST_CASE(RingBufferTest, TestTypes);

/**
 * Inside the type-parameterized test, we refer to the type parameter through
 * the special name TypeParam. This will be iteratively replaced by each type
 * in the TestTypes list.
 */
TYPED_TEST(RingBufferTest, FunctionalNProdMConsPerf){
  using namespace utils;
  typedef TestProperties<TypeParam> TestProp;

  CREATE_PTIMER(ringbuf_timer);
  PTIMER_START(ringbuf_timer);

  this->RunProducersConsumers();

  PTIMER_STOP(ringbuf_timer);
  PRINT_PTIMER(ringbuf_timer);

}

class RingBufferLatencyTest : public RingBufferTest<LatencyProbe> {};

TEST_F(RingBufferLatencyTest, NProdMConsLatency){
  using namespace std;
  typedef TestProperties<LatencyProbe> TestProp;

  this->RunProducersConsumers();

  cout<<"Saving latency results in file: ringbuf_latency.dat"<<endl;
  ofstream out("ringbuf_latency.dat", fstream::app);
  ofstream ix("ringbuf_latency.ix", fstream::app);
  out<<"<<EXPERIMENT"<<endl;
  ix<<out.tellp()<<" ";
  out<<"<< "<<TEST_COMMENT<<endl;
  out<<"bufsize "<<BUF_SIZE<<endl;
  out<<"producers "<<NR_PRODUCERS<<endl;
  out<<"consumers "<<NR_CONSUMERS<<endl;
  out<<"pushes_per_producer "<<N<<endl;
  timespec base;
  for(unsigned long i = 0; i < N * NR_PRODUCERS; ++i) {
    timespec lati = diff(TestProp::prod_buf[i].at_push,
                         TestProp::prod_buf[i].at_pop);
    if(i == 0) base = TestProp::prod_buf[0].at_pop;
    out<<TestProp::prod_buf[i].payload<<" "<<
         diff(base, TestProp::prod_buf[i].at_pop)<<" "<<lati<<endl;
  }
  ix.close();
  out.close();

}
