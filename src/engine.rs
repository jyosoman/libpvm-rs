use futures::executor::Executor;
use futures::future::Future;
use futures_cpupool::Builder;

struct StreamHashSplitter {}

impl Executor for ProcessingPool {
    fn execute(&self, r: Run) {}
}

fn cpu_pool_init() {
    let mut cpuBpool = Builder::new();
    let mut ioBpool = Builder::new();
    cpuBpool.pool_size += 2;
    cpuBpool.name_prefix = Some(String::from("opus-cpu"));
    cpuBpool.create();
    ioBpool.pool_size += 4;
    ioBpool.name_prefix = Some(String::from("opus-io"));
    ioBpool.create();
}
