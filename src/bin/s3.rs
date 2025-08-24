use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for i in 1.. {
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => writeln!(handle, "FizzBuzz")?,
            (true, false) => writeln!(handle, "Fizz")?,
            (false, true) => writeln!(handle, "Buzz")?,
            (false, false) => writeln!(handle, "{}", i)?,
        }
    }

    Ok(())
}
