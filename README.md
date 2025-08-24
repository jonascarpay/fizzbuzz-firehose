# fizzbuzz-firehose

Programming exercise to see if we can get FizzBuzz into the gigabytes per second without looking things up.
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
Instead, let's set up some profiling.
