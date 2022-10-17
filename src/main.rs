mod lib; // lib.rs is a crate file. crate: compilation unit

// mod or module is similar to namespaces: logically groups code within a crate
// use: brings a rust item into the current scope. Similar to Using in C++ or import .. as x in python

use lib::test;
use lib::test::Print;

struct HHH {
    nnn: String,
}

impl test::Print for HHH {
    fn new(name: String) -> Self {
        return HHH {
            nnn: String::from("hello"),
        };
    }

    fn print(&self) {}

    fn print2(&self) {}
}

fn main() {
    let state_code = "MH";
    let state = match state_code {
        "MH" => {
            println!("Found match for MH");
            "Maharashtra"
        }
        "KL" => "Kerala",
        "KA" => "Karnadaka",
        "GA" => "Goa",
        _ => "Unknown",
    };
    println!("state = {}", state);

    let na = test::NewsArticle::new(String::from("NYT"));

    let hhh: HHH = HHH {
        nnn: String::from("hhh"),
    };
    hhh.print();
    println!("{:?}", na);

    na.print();
    na.print2();
    println!("NAME = {}", na.sucks);

    print((20, false, 100.6234));

    // Stack allocated array
    let mut xs: [i64; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    xs[0] = 12;

    let ea: [f64; 0] = [];
    assert_eq!(&ea, &[]);
    assert_eq!(&ea, &[][..]);

    for i in 0..xs.len() + 1 {
        match xs.get(i) {
            Some(xval) => println!("{}: {}", i, xval),
            None => println!("Slow down! {} is too far!", i),
        }
    }
}

fn print(tup: (i32, bool, f64)) -> String {
    println!("Inside print method!");
    let (age, is_male, cgpa) = tup;
    println!("Age is {}, isMale? {}, cpga = {}", age, is_male, cgpa);
    let out = tup.0.to_string();
    return out;
}
