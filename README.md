# fizzbuzz-firehose

Sunday morning programming exercise to see if we can get FizzBuzz to the 1GiB/s mark, without looking at any other solutions.
Inspired by [this infamous SO thread](https://codegolf.stackexchange.com/questions/215216/high-throughput-fizz-buzz/).

Performance is measured using `cargo run --release --bin s0 | pv > /dev/null`.

## Step 0: Baseline

Naive implementation.
Performance: 16.7 MiB/s, which is pretty terrible since the equivalent C implementation claims to reach 170 MiB/s.

```rust
fn main() {
    for i in 1.. {
        let m3 = i % 3 == 0;
        let m5 = i % 5 == 0;
        if m3 && m5 {
            println!("FizzBuzz");
        } else if m3 {
            println!("Fizz");
        } else if m5 {
            println!("Buzz");
        } else {
            println!("{}", i);
        }
    }
}
```

## Step 1: Cleaning up

If/else chains are ugly.
This implementation actually produces the exact same ASM, but it looks nicer.

```rust
fn main() {
    for i in 1.. {
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => println!("FizzBuzz"),
            (true, false) => println!("Fizz"),
            (false, true) => println!("Buzz"),
            (false, false) => println!("{}", i),
        }
    }
}
```

## Step 2: Ditching `println`

Let's start by closing the gap with C.
We're currently 10x slower than C (and it doesn't seem to matter whether we run in debug or release).
That means `println` is doing a lot more than `printf`.
Let's start by ditching it and writing to `stdout` directly.

Here's a very naive baseline that's essentially the same thing as the `println` implementation.
Looking at the assembly, we can see that there's a lot more housekeeping now, but it's too early to analyze in great detail.

Performance: 16.9 MiB/s.
Not even a blip.

```rust
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    for i in 1.. {
        match (i % 3 == 0, i % 5 == 0) {
            (true, true) => writeln!(stdout, "FizzBuzz")?,
            // ...
        }
    }
    Ok(())
}
```

## Step 3: Locking

When writing to `stdout` like we do above, we acquire and release the global `stdout` lock on every write call.
Let's start by locking once, for the duration of the entire program.

```rust
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // ...
}
```

Performance: still 16.9 MiB/s.
I didn't expect locking to be too significant, but I thought we'd see at least a small bump.
On to the big one.

## Step 4: Buffering

Currently, every time we write we interrupt our program to tell the OS about it.
Let's stop doing that, and instead write to a buffer.

```rust
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut buf = BufWriter::new(stdout.lock());
    // ...
}
```

Performance: 650 MiB/s.
Gap successfully closed.

## Status check
Alright, we've beaten C, but buffering was _the_ low-hanging fruit.
From now, we'll have to work harder for our gains, and our code will be less idiomatic.
I can make some guesses about what's holding us back right now, but blindly optimizing things that you _think_ are slow is usually a bad idea.
Instead, let's get some data.
We have two methods available to us: profiling, and looking at the assembly.

Profiling is an art unto itself.
You can't measure a program without distorting it, and the smaller and faster the program, the more sensitive to distortion.
Still, as long as we're careful about interpreting the data, it can't really hurt to have more of it.
[flamegraph.svg](./flamegraph.svg) shows the result of profiling the above program with `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin s4 > /dev/null`.

Assembly is less intrusive, but also less useful.
For example, `cargo asm` shows that a lot of our code is dedicated to error handling.
That suggests that we can speed up our program by optimizing that part, but since the profile doesn't agree, and a more careful reading shows that it's mostly outside the hot loop, let's not let ourselves get distracted quite yet.

Instead, we focus on what both the profiler and assembly suggest is causing us to be slow: serialization.
`writeln` does a lot of work other than putting bytes in buffers, and right now, we don't want that.
One such thing is costing us time, and preventing us from ditching `writeln` altogether: serializing integers, or binary-decimal conversion.

## Step 5: Serializing integers

This is a fun one.
Any base-10 math is unnatural for a computer, for example converting arbitrary integers into a decimal format, as we're doing inside our hot loop.
Fortunately, we can simplify most of it away by recognizing that the number we're printing doesn't really change much between loops.
Instead, we just store a base 10 representation, and alter that directly.

```rust

struct AsciiCounter { /* ... */ }

impl AsciiCounter {
    fn new() -> AsciiCounter { /* ... */ }
    fn bump(&mut self) { /* ... */ }
    fn view_ascii(&self) -> &[u8] { /* ... */ }
}
fn main() -> io::Result<()> {
    // ...
    let mut counter = AsciiCounter::new();

    for i in 1.. {
        counter.bump();
        match (i % 3 == 0, i % 5 == 0) {
            // ...
            (false, false) => {
                buf.write_all(counter.view_ascii())?;
                writeln!(buf)?;
            }
        }
    }
    // ...
}
```

Performance: 946 MiB/s.
As expected, we were spending a lot of time serializing integers, and now we're not.
We're on the cusp of the gigabyte mark!
But, more importantly, now we can start tackling the bigger issue: ditching `writeln`.

## Step 6: Ditching `writeln`

This is a simple change:

```rust
fn main() -> io::Result<()> {
    // ...
    for i in 1.. {
        // ...
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
    // ...
}
```

Performance: 1.80 GiB/s.
We did it!
It's not something that will make the global leaderboard, but this is fine for now.
Our [profile](./flamegraph_s6.svg) shows that right now, we're spending most of our time copying data, which I don't think is easily worked around.

Some thoughts:
  - The pattern repeats every 15 iterations
  - We can make a conservative guess about how many times we can write to a buffer
