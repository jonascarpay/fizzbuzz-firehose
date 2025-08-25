use std::io::{self, BufWriter, Write};

struct AsciiCounter {
    digits: [u8; 16],
    head: usize,
}

impl AsciiCounter {
    fn new() -> AsciiCounter {
        AsciiCounter {
            digits: [b'0'; 16],
            head: 15,
        }
    }
    fn bump(&mut self) {
        for (i, digit) in self.digits.iter_mut().enumerate().rev() {
            self.head = std::cmp::min(self.head, i);
            if *digit == b'9' {
                *digit = b'0';
            } else {
                *digit += 1;
                break;
            }
        }
    }
    fn view_ascii(&self) -> &[u8] {
        &self.digits[self.head..]
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut buf = BufWriter::new(stdout.lock());

    let mut counter = AsciiCounter::new();

    loop {
        // 1
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // 2
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // Fizz
        counter.bump();
        buf.write_all(b"Fizz\n")?;
        // 4
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // Buzz
        counter.bump();
        buf.write_all(b"Buzz\n")?;
        // Fizz
        counter.bump();
        buf.write_all(b"Fizz\n")?;
        // 7
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // 8
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // Fizz
        counter.bump();
        buf.write_all(b"Fizz\n")?;
        // Buzz
        counter.bump();
        buf.write_all(b"Buzz\n")?;
        // 11
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // Fizz
        counter.bump();
        buf.write_all(b"Fizz\n")?;
        // 13
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // 14
        counter.bump();
        buf.write_all(counter.view_ascii())?;
        buf.write_all(b"\n")?;
        // FizzBuzz
        counter.bump();
        buf.write_all(b"FizzBuzz\n")?;
    }
}
