/// [Wikipedia](https://en.wikipedia.org/wiki/Xorshift)
pub struct XorRand {
    state: u32,
}

impl XorRand {
    pub fn new(state: u32) -> Self {
        Self { state }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }

    pub fn next_between(&mut self, min: u32, max: u32) -> u32 {
        (self.next() + min) % max
    }
}
