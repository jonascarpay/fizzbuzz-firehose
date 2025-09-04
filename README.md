# fizzbuzz-firehose

Sunday morning Rust exercise:
see if we can start with a naive FizzBuzz, and then get it to the 1GiB/s mark.
No looking at any other solutions, or using any crates other than the standard library.
I'm not here to compete on [the global leaderboard](https://codegolf.stackexchange.com/questions/215216/high-throughput-fizz-buzz/), I just want to see how far I can get by myself.

Throughput is measured using `cargo run --release --bin s0 | pv > /dev/null`.

## Step 0: Baseline

Naive implementation.

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

Throughput: 16.7 MiB/s, which is pretty terrible since the baseline C implementation claims to reach 170 MiB/s.

## Step 1: Cleaning up

If/else chains are ugly.
This implementation actually produces the exact same assembly, but it's much nicer.

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

### Assembly peeping

Before we continue to step 2, a brief aside on assembly.

Assembly might seem intimidating, but you really don't need to be able to read it for looking at it to be useful.
Above is a great example, and something that I do all the time: make a change, and see if/how the assembly changes.
It teaches about what sort of things get optimized away, which is a great skill because it allows you to pick more readable alternatives, confident that you're not sacrificing performance[^manual_opt].

[^manual_opt]:
    Aside on the aside: I used to work in performance-sensitive environments, and one of my great great frustrations is people blindly hand-optimizing code.
    It turns out, code that has transparent data flows is easy to understand for both humans and the compiler, and the compiler is better at optimizing than you are.
    Conversely, if the compiler _doesn't_ optimize something, instead of trying to hand-optimize it yourself, it's usually more productive to find out _why_ the compiler didn't optimize it itself.
    It usually has a good reason, and understanding it allows you to tackle the root cause.

Of course there's [Compiler Explorer](https://godbolt.org/), but my recommendation is to get comfortable with the amazing [`cargo-show-asm`](https://github.com/pacak/cargo-show-asm).
Some `cargo-show-asm` tips:
- Run `cargo-show-asm` in the project root, and then find your function in the list. If it doesn't show up, add `#[inline(never)]`.
- Rule of thumb: more assembly is almost always worse.
- Focus on the hot path. The `--rust` flag can help you find it.
- If you can't read assembly (yet), LLM's are your friend. For example, "These two pieces of assembly are functionally equivalent. Comment on their differences, and the performance implications."
- Check the `--help`, there are many useful flags for making output more readable.

## Step 2: Ditching `println`

We're currently 10x slower than C, and there's really only one place for slowness to hide: `println` is doing a lot more than `printf`.
Let's start by ditching `println` and writing to `stdout` ourselves.

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

Here's a very naive baseline that's essentially the same thing as the `println` implementation.
There's more assembly code now, but that's mostly because of the more explicit error handling.
Too early to analyze in great detail.

TODO: talk about the differences

Throughput: 16.9 MiB/s.
Not even a blip.

## Step 3: Locking

My understanding is that when we write to `stdout` like we do above, we acquire and release a global `stdout` lock on every individual write.
Let's start by acquiring the lock once, and keep it for the duration of the entire program.

```rust
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // ...
}
```

Throughput: still 16.9 MiB/s.
I didn't expect locking to be too significant, but I thought we'd see at least a small bump.
On to the big one.

## Step 4: Buffering

Currently, every time we write we interrupt our program to tell the OS about it.
Let's stop doing that, and instead write to a buffer.

TODO: show with strace that the number of syscalls has gone down.

```rust
fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut buf = BufWriter::new(stdout.lock());
    // ...
}
```

Throughput: 650 MiB/s.
Gap successfully closed, although I'm a little sad we blew right by it.
I'd have been interested in seeing what Rust's exact equivalent of the naive C implementation would've looked like.
Oh well.

## Status check
Alright, we've beaten C, but buffering was _the_ low-hanging fruit.
From now, we'll have to work harder for our gains.
I can make some guesses about what's holding us back right now, but blindly optimizing things that you _think_ are slow is usually a bad idea.
Instead, let's get some actual data by profiling our program.

<!-- https://nnethercote.github.io/perf-book/profiling.html -->

Profiling is an art unto itself.
You can't measure a program without distorting it, and the smaller and faster the program, the more sensitive to distortion.
Still, as long as we're careful about interpreting the data, it can't really hurt to have more of it.
[flamegraph.svg](./flamegraph.svg) shows the result of profiling the above program with `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin s4 > /dev/null`.

Instead, we focus on what both the profiler and assembly suggest is causing us to be slow: serialization.
`writeln` does a lot of work other than putting bytes in buffers, and right now, we don't want that.
One such thing is costing us time, and preventing us from ditching `writeln` altogether: serializing integers, or binary-decimal conversion.

## Step 5: Serializing integers

This is a fun one.
Any base-10 math is unnatural for a computer.
For example: converting integers into a decimal format for printing, as we're doing inside our hot loop.
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

Throughput: 946 MiB/s.
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

Throughput: 2.09 GiB/s.
We did it!
It's not something that will make the global leaderboard, but this is fine for now.
Our [profile](./flamegraph_s6.svg) shows that right now, we're spending most of our time copying data, which I don't think is easily worked around.

Some thoughts:
  - The pattern repeats every 15 iterations
  - We can make a conservative guess about how many times we can write to a buffer

## Step 7: Unrolling the 15-step loop

It's now Monday, and I've been thinking about this.
If we unroll the loop, we don't need to keep track of the steps anymore.
Unrolling also allows us to aggregate the counter bumps into one, as well as the writes.

Throughput: 2.11 GiB/s.
No real change.
Not _too_ surprising, since we didn't get rid of any of the real slow calls, but this was also mostly just so we could do the _next_ big thing.

## Step 8: Aggregating calls

- Combine subsequent known string literal writes into one
- Combine subsequent counter bumps into one
- Make the newline a part of the counter buffer

Throughput: 2.27 GiB/s.
Not as much as I'd hoped or expected, to be honest.
Time for another profile, and see how much time we're spending where.

The [profile](./flamegraph_s8.svg) shows that we're spending _80%_ of our time now copying data.
The good news is that that means that everything else is pretty fast.
The bad news is that that is hard to fix.

## Step 9: Custom buffer and tuning

Out of desperation more than anything  else

Throughput: 2.70 GiB/s.

## Step 10: What if we only change the things that need to be changed

A thought came to me in the shower: what if we fill a buffer once, and then only change things that need to change?
If we pick the right buffer size, and we group things by number of digits, we can align things so that the fizzes, buzzes, and fizzbuzzes all stay in the same place.
Just thinking of all the corner cases makes my head spin a little, but it _should_ work.

Throughput: 3.0 GiB/s

But, we're no longer bound by copying!
We can optimize further!

## Step 11: Optimizing further

Assembly shows we're not inlining fast_buzz.
