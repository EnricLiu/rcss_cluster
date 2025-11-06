pub trait Message {
    fn as_bytes(&self) -> &[u8];
}

impl Message for String {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}