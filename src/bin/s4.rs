use std::io::{self, BufWriter, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut buf = BufWriter::new(stdout.lock());

    for i in 1.. {
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => writeln!(buf, "FizzBuzz")?,
            (true, false) => writeln!(buf, "Fizz")?,
            (false, true) => writeln!(buf, "Buzz")?,
            (false, false) => writeln!(buf, "{}", i)?,
        }
    }
    Ok(())
}
