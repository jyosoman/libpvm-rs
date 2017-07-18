extern crate opus;
extern crate futures;
extern crate futures_cpupool;

use futures::Future;
use futures_cpupool::CpuPool;


pub fn main() {
    let pool = CpuPool::new_num_cpus();

    pool.spawn_fn(|| {
        let res: Result<i32, ()> = Ok(1);
        res
    });

}
