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
