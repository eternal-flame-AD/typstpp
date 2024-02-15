pub trait ReadExt: std::io::Read {
    fn read_until(&mut self, pattern: &[u8], buf: &mut Vec<u8>) -> std::io::Result<usize>;
}

impl<T: std::io::Read> ReadExt for T {
    fn read_until(&mut self, pattern: &[u8], buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut byte = [0];
        let mut count = 0;
        loop {
            self.read_exact(&mut byte)?;
            buf.push(byte[0]);
            count += 1;
            if buf.ends_with(pattern) {
                buf.truncate(buf.len() - pattern.len());
                break;
            }
        }
        Ok(count)
    }
}
