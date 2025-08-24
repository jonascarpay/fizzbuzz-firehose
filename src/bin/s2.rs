use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    for i in 1.. {
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => writeln!(stdout, "FizzBuzz")?,
            (true, false) => writeln!(stdout, "Fizz")?,
            (false, true) => writeln!(stdout, "Buzz")?,
            (false, false) => writeln!(stdout, "{}", i)?,
        }
    }
    Ok(())
}
