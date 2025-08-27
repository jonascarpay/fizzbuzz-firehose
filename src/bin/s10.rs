use std::{
    io::{self, Write},
    ops::{AddAssign, SubAssign},
};

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
    // TODO Addend should be known! See if this optimizes nicely. Maybe unroll a few iters manually
    fn ripple_carry_add_ascii(&mut self, mut offset: usize, mut addend: usize) {
        while addend > 0 {
            let addend_ones = addend % 10;
            addend = addend / 10;
            let digit = &mut self.data[offset];
            digit.add_assign(addend_ones as u8);
            if *digit > b'9' {
                digit.sub_assign(10);
                addend += 1;
            }
            offset -= 1;
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

fn fast_buzz<W: Write>(digits: usize, out: &mut W) -> io::Result<()> {
    let start = usize::pow(10, digits as u32 - 1);
    let end = 10 * start;

    let bytes_per_15 = 47 + 8 * digits as usize;
    let periods_per_buf = BUF_SIZE / bytes_per_15 as usize;
    let lines_per_buf = periods_per_buf * 15;
    let bytes_per_buf = periods_per_buf * bytes_per_15;

    let lines = end - start;
    let full_batches = lines / lines_per_buf;

    if full_batches > 0 {
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
        for i_batch in 0..full_batches - 1 {
            for i_period in 0..periods_per_buf {
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

                let offset = i_period * bytes_per_15; // TODO add rather than mul
                buf.ripple_carry_add_ascii(offset + 4 + 1 * digits, lines_per_buf); // 11
                buf.ripple_carry_add_ascii(offset + 10 + 2 * digits, lines_per_buf); // 13
                buf.ripple_carry_add_ascii(offset + 11 + 3 * digits, lines_per_buf); // 14
                buf.ripple_carry_add_ascii(offset + 21 + 4 * digits, lines_per_buf); // 1
                buf.ripple_carry_add_ascii(offset + 22 + 5 * digits, lines_per_buf); // 2
                buf.ripple_carry_add_ascii(offset + 28 + 6 * digits, lines_per_buf); // 4
                buf.ripple_carry_add_ascii(offset + 39 + 7 * digits, lines_per_buf); // 7
                buf.ripple_carry_add_ascii(offset + 40 + 8 * digits, lines_per_buf);
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
