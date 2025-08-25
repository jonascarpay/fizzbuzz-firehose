use std::io::{self, Write};

const COUNTER_DIGITS: usize = 15;
const COUNTER_BUF_SIZE: usize = COUNTER_DIGITS + 1;
struct AsciiCounter {
    digits: [u8; COUNTER_BUF_SIZE],
    head: usize,
}

impl AsciiCounter {
    fn new() -> AsciiCounter {
        let mut buf = [b'0'; COUNTER_BUF_SIZE];
        buf[COUNTER_BUF_SIZE - 1] = b'\n';
        AsciiCounter {
            digits: buf,
            head: COUNTER_BUF_SIZE - 2,
        }
    }
    fn bump(&mut self, incr: u8) {
        let ones = &mut self.digits[COUNTER_BUF_SIZE - 2];
        *ones += incr;

        if *ones > b'9' {
            *ones -= 10;
            for (i, digit) in self.digits.iter_mut().enumerate().rev().skip(2) {
                self.head = std::cmp::min(self.head, i);
                if *digit == b'9' {
                    *digit = b'0';
                } else {
                    *digit += 1;
                    break;
                }
            }
        }
    }
    fn view_ascii(&self) -> &[u8] {
        &self.digits[self.head..]
    }
}

const BUF_SIZE: usize = 64 * 1024;
struct Buffer {
    data: [u8; BUF_SIZE],
    offset: usize,
}

impl Buffer {
    fn new() -> Self {
        Buffer {
            data: [0; BUF_SIZE],
            offset: 0,
        }
    }
    fn write_unsafe(&mut self, bytes: &[u8]) {
        let src = bytes.as_ptr();
        let dst = self.data.as_mut_ptr().wrapping_byte_add(self.offset);
        let n = bytes.len();
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, n);
        }
        self.offset += n;
    }
    fn spare_capacity(&self) -> usize {
        BUF_SIZE - self.offset
    }
    #[must_use]
    fn flush(&mut self) -> &[u8] {
        let offset = self.offset;
        self.offset = 0;
        &self.data[..offset]
    }
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut buf = Buffer::new();

    let mut counter = AsciiCounter::new();
    counter.bump(1);

    loop {
        if buf.spare_capacity() < 166 {
            stdout.write_all(buf.flush());
        }
        // 1
        // 16 bytes
        buf.write_unsafe(counter.view_ascii());
        // 2
        // 16 bytes
        counter.bump(1);
        buf.write_unsafe(counter.view_ascii());
        // Fizz
        // 5 bytes
        buf.write_unsafe(b"Fizz\n");
        // 4
        // 16 bytes
        counter.bump(2);
        buf.write_unsafe(counter.view_ascii());
        // Buzz
        // Fizz
        // 10 bytes
        buf.write_unsafe(b"Buzz\nFizz\n");
        // 7
        // 16 bytes
        counter.bump(3);
        buf.write_unsafe(counter.view_ascii());
        // 8
        // 16 bytes
        counter.bump(1);
        buf.write_unsafe(counter.view_ascii());
        // Fizz
        // Buzz
        // 10 bytes
        buf.write_unsafe(b"Fizz\nBuzz\n");
        // 11
        // 16 bytes
        counter.bump(3);
        buf.write_unsafe(counter.view_ascii());
        // Fizz
        // 5 bytes
        buf.write_unsafe(b"Fizz\n");
        // 13
        // 16 bytes
        counter.bump(2);
        buf.write_unsafe(counter.view_ascii());
        // 14
        // 16 bytes
        counter.bump(1);
        buf.write_unsafe(counter.view_ascii());
        // FizzBuzz
        // 9 bytes
        buf.write_unsafe(b"FizzBuzz\n");
        counter.bump(2);
        // Total = 166 bytes
    }
}
