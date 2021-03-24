pub struct TestGenerator {
    length: usize
}

impl<'a> TestGenerator {
    pub fn new(length: usize) -> TestGenerator {
        TestGenerator { length }
    }
}

impl<'a> Iterator for TestGenerator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.length > 0 {
            self.length -= 1;
            Some(0xAAAAAAAA)
        } else {
            None
        }
    }
}
