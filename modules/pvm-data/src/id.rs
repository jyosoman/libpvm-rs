use std::fmt::Display;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ID(i64);

impl ID {
    pub fn new(val: i64) -> ID {
        ID(val)
    }
    pub fn inner(self) -> i64 {
        self.0
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
