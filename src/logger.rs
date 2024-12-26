#[derive(Debug)]
pub struct Logger {}

impl Logger {
    pub fn start() {
        std::thread::spawn(|| {
            let mut logger = Logger {};
            while logger.run_one().is_ok() {}
        });
    }

    fn run_one(&mut self) -> orfail::Result<()> {
        todo!()
    }
}
