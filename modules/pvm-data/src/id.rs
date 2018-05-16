use std::fmt::Display;

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

impl Display for ID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
