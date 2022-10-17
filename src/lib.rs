// every file is a module, directories are sub-modules
//

pub mod test {

    pub trait Print {
        fn new(name: String) -> Self;

        fn print(&self);

        fn print2(&self);

        fn divide(&self, a: f64, b: f64) -> Option<f64>;
    }

    struct Color(i32, i32, i32);

    #[derive(Debug)]
    pub struct NewsArticle {
        name: String,
        year: i32,
        pub sucks: bool,
    }

    // impl NewsArticle {
    //     pub fn new(name: String, year: i32, sucks: bool) -> Self {
    //         return NewsArticle {
    //             name: name,
    //             year: year,
    //             sucks: sucks,
    //         };
    //     }
    // }

    // self is used as argument to struct functions, and is essentially equivalent to self: Self
    // in the same way, &self is equivalent to self: &Self. Self refers to the current type, self to the instance
    // Can use either Self or NewsArticle as return type of the struct that impl Print
    impl Print for NewsArticle {
        fn new(name: String) -> Self {
            return NewsArticle {
                name: name,
                year: 1942,
                sucks: false,
            };
        }

        fn print(&self) {
            println!("The news are garbage");
        }

        fn print2(&self) {
            println!("Again, the news are garbage | {:?}", self);
        }

        fn divide(&self, a: f64, b: f64) -> Option<f64> {
            if b < 0.001 {
                None
            } else {
                Some(a / b)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test;
    use super::test::Print;

    #[test]
    fn test_divide() {
        let na: test::NewsArticle = test::NewsArticle::new(String::from("hello"));

        assert_eq!(na.divide(100.0, 0.0), None);
        assert_eq!(test::NewsArticle::divide(&na, 100.0, 0.0), None);
    }
}
