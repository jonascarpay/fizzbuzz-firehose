use std::io::{self, BufWriter, Write};

struct AsciiCounter {
    digits: [u8; 16],
    head: usize,
}

impl AsciiCounter {
    fn new() -> AsciiCounter {
        let mut buf = [b'0'; 16];
        buf[15] = b'\n';
        AsciiCounter {
            digits: buf,
            head: 14,
        }
    }
    fn bump(&mut self, incr: u8) {
        let ones = &mut self.digits[14];
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

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut buf = BufWriter::new(stdout.lock());

    let mut counter = AsciiCounter::new();
    counter.bump(1);

    loop {
        // 1
        buf.write_all(counter.view_ascii())?;
        // 2
        counter.bump(1);
        buf.write_all(counter.view_ascii())?;
        // Fizz
        buf.write_all(b"Fizz\n")?;
        // 4
        counter.bump(2);
        buf.write_all(counter.view_ascii())?;
        // Buzz
        // Fizz
        buf.write_all(b"Buzz\nFizz\n")?;
        // 7
        counter.bump(3);
        buf.write_all(counter.view_ascii())?;
        // 8
        counter.bump(1);
        buf.write_all(counter.view_ascii())?;
        // Fizz
        // Buzz
        buf.write_all(b"Fizz\nBuzz\n")?;
        // 11
        counter.bump(3);
        buf.write_all(counter.view_ascii())?;
        // Fizz
        buf.write_all(b"Fizz\n")?;
        // 13
        counter.bump(2);
        buf.write_all(counter.view_ascii())?;
        // 14
        counter.bump(1);
        buf.write_all(counter.view_ascii())?;
        // FizzBuzz
        buf.write_all(b"FizzBuzz\n")?;
        counter.bump(2);
    }
}
