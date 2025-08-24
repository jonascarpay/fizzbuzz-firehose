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

    for i in 1.. {
        counter.bump();
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => buf.write_all(b"FizzBuzz\n")?,
            (true, false) => buf.write_all(b"Fizz")?,
            (false, true) => buf.write_all(b"Buzz")?,
            (false, false) => {
                buf.write_all(counter.view_ascii())?;
                buf.write_all(b"\n")?;
            }
        }
    }
    Ok(())
}
