pub fn key(key: tuinix::KeyInput) -> impl std::fmt::Display {
    crate::keymatcher::KeyMatcher::Literal(key)
}

pub fn filler(ch: char, count: usize) -> impl std::fmt::Display {
    Filler { ch, count }
}

#[derive(Debug)]
struct Filler {
    ch: char,
    count: usize,
}

impl std::fmt::Display for Filler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.count {
            write!(f, "{}", self.ch)?;
        }
        Ok(())
    }
}
