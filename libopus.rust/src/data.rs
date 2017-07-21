#[derive(Debug)]
pub struct Process {
    pub db_id: u64,
    pub uuid: String,
    pub cmdline: String,
    pub pid: i32,
    pub thin: bool,
}
