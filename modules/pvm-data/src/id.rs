#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ID(u64);

impl ID {
    pub fn new(val: u64) -> ID {
        ID(val)
    }
    pub fn inner(self) -> u64 {
        self.0
    }
}
