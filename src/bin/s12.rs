use std::io::{self, Write};

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
    fn spare_capacity(&self) -> usize {
        BUF_SIZE - self.offset
    }
    fn view(&self) -> &[u8] {
        &self.data[..self.offset]
    }
    fn ripple_carry_add_ascii(&mut self, mut offset: usize, addend: u8) {
        let digit = &mut self.data[offset];
        *digit += addend;

        if *digit > b'9' {
            *digit -= 10;
            loop {
                offset -= 1;
                let digit = &mut self.data[offset];
                if *digit == b'9' {
                    *digit = b'0'
                } else {
                    *digit += 1;
                    break;
                }
            }
        }
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = buf.len();
        if n <= self.spare_capacity() {
            (&mut self.data[self.offset..self.offset + n]).copy_from_slice(buf);
            self.offset += n;
            Ok(n)
        } else {
            Err(io::Error::other("Buffer overrun"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn write_fizz_buzz<W: Write>(i: usize, out: &mut W) -> io::Result<()> {
    match (i % 3 == 0, i % 5 == 0) {
        (true, true) => writeln!(out, "FizzBuzz"),
        (true, false) => writeln!(out, "Fizz"),
        (false, true) => writeln!(out, "Buzz"),
        (false, false) => writeln!(out, "{}", i),
    }
}

fn find_lines_per_buf(digits: usize) -> usize {
    let bytes_per_cycle = 47 + 8 * digits;
    (2..)
        .flat_map(|exp| {
            let b = usize::pow(10, exp);
            [3 * b, 6 * b, 9 * b]
        })
        .take_while(|lines| ((lines / 15) * bytes_per_cycle) < BUF_SIZE)
        .last()
        .unwrap()
}

const fn find_lines_per_buf2(digits: usize) -> Option<(u8, usize)> {
    let bytes_per_cycle = 47 + 8 * digits;
    let max_cycles_per_buf = BUF_SIZE / bytes_per_cycle;
    let max_lines_per_buf = max_cycles_per_buf * 15;
    let mut n = max_lines_per_buf;
    let mut digits = 0;
    loop {
        if n > 30 {
            n /= 10;
            digits += 1;
        } else {
            break;
        }
    }
    if n >= 9 {
        Some((9, digits))
    } else if n >= 6 {
        Some((6, digits))
    } else if n >= 3 {
        Some((3, digits))
    } else {
        None
    }
}

fn fast_buzz<W: Write>(digits: usize, out: &mut W) -> io::Result<()> {
    let start = usize::pow(10, digits as u32 - 1);
    let end = 10 * start;

    let bytes_per_cycle = 47 + 8 * digits as usize;

    let (addend, suffix_digits) = find_lines_per_buf2(digits).unwrap();
    let lines_per_buf = addend as usize * 10_usize.pow(suffix_digits as u32);
    let cycles_per_buf = lines_per_buf / 15;
    let bytes_per_buf = cycles_per_buf * bytes_per_cycle;

    let lines = end - start;
    let full_batches = lines / lines_per_buf;

    if full_batches > 0 {
        let (addend, suffix_digits) = find_lines_per_buf2(digits).unwrap();
        let mut buf = Buffer::new();
        let mut i = start;
        // 1: fill
        for i in start..start + lines_per_buf {
            write_fizz_buzz(i, &mut buf)?;
        }
        assert_eq!(buf.offset, bytes_per_buf);
        assert_eq!(buf.spare_capacity(), BUF_SIZE - bytes_per_buf);

        // 2: send
        out.write_all(buf.view())?;
        i += lines_per_buf;

        assert_eq!(i % 15, 10);
        assert_eq!(
            find_lines_per_buf(digits),
            addend as usize * 10_usize.pow(suffix_digits as u32)
        );
        for i_batch in 0..full_batches - 1 {
            for i_cycle in 0..cycles_per_buf {
                // Buzz     | 5   | 0  | 0
                // 11       | n+1 | 5  | 0
                // Fizz     | 5   | 6  | 1
                // 13       | n+1 | 11 | 1
                // 14       | n+1 | 12 | 2
                // FizzBuzz | 9   | 13 | 3
                // 1        | n+1 | 22 | 3
                // 2        | n+1 | 23 | 4
                // Fizz     | 5   | 24 | 5
                // 4        | n+1 | 29 | 5
                // Buzz     | 5   | 30 | 6
                // Fizz     | 5   | 35 | 6
                // 7        | n+1 | 40 | 6
                // 8        | n+1 | 41 | 7
                // Fizz     | 5   | 42 | 8

                let offset = i_cycle * bytes_per_cycle - suffix_digits; // TODO add rather than mul
                buf.ripple_carry_add_ascii(offset + 4 + 1 * digits, addend); // 11
                buf.ripple_carry_add_ascii(offset + 10 + 2 * digits, addend); // 13
                buf.ripple_carry_add_ascii(offset + 11 + 3 * digits, addend); // 14
                buf.ripple_carry_add_ascii(offset + 21 + 4 * digits, addend); // 1
                buf.ripple_carry_add_ascii(offset + 22 + 5 * digits, addend); // 2
                buf.ripple_carry_add_ascii(offset + 28 + 6 * digits, addend); // 4
                buf.ripple_carry_add_ascii(offset + 39 + 7 * digits, addend); // 7
                buf.ripple_carry_add_ascii(offset + 40 + 8 * digits, addend);
                // 8
            }
            out.write_all(buf.view())?;
        }

        for i in start + full_batches * lines_per_buf..end {
            write_fizz_buzz(i, out)?;
        }
    } else {
        for i in start..end {
            write_fizz_buzz(i, out)?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    fast_buzz(1, &mut stdout)?;
    fast_buzz(2, &mut stdout)?;
    fast_buzz(3, &mut stdout)?;
    fast_buzz(4, &mut stdout)?;
    fast_buzz(5, &mut stdout)?;
    fast_buzz(6, &mut stdout)?;
    fast_buzz(7, &mut stdout)?;
    fast_buzz(8, &mut stdout)?;
    fast_buzz(9, &mut stdout)?;
    fast_buzz(10, &mut stdout)?;
    fast_buzz(11, &mut stdout)?;
    fast_buzz(12, &mut stdout)?;
    fast_buzz(13, &mut stdout)?;
    fast_buzz(14, &mut stdout)?;
    fast_buzz(15, &mut stdout)?;
    fast_buzz(16, &mut stdout)?;
    Ok(())
}
