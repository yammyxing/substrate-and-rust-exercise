extern crate clap;

use clap::{App, Arg, SubCommand};
use std::ffi::OsString;

struct Introduction {
    times: i32,
    name: String,
    age: String
}

impl Introduction {
    fn new() -> Self {
        Self::new_from(std::env::args_os().into_iter()).unwrap_or_else(|e| e.exit())
    }

    fn new_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        // basic app information with version and about
        let app = App::new("hello-ai")
            .version("1.0")
            .about("AI robot");

        // Define the name command line option
        let name_option = Arg::with_name("name")
            .long("name") // allow --name
            .short("n") // allow -n
            .takes_value(true)
            .help("What's the ai's name?")
            .required(true);

        let age_option = Arg::with_name("age")
            .long("age") // allow --age
            .short("a") // allow -a
            .takes_value(true)
            .help("How old is the ai?")
            .required(true);

        let sub_test_option = SubCommand::with_name("test")
            .about("controls testing features")
            .version("1.3")
            .arg(Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely"));

        // now add in the argument
        let app = app.arg(name_option)
            .arg(age_option)
            // .arg(times_option)
            .subcommand(sub_test_option);

        // extract the matches
        let matches = app.get_matches_from_safe(args)?;

        // Extract the actual name
        let name = matches
            .value_of("name")
            .expect("This can't be None, we said it was required");

        let age = matches
            .value_of("age")
            .expect("This can't be None!");

        if let Some(matches) = matches.subcommand_matches("test") {
            if matches.is_present("debug") {
                println!("Printing debug info...");
            } else {
                println!("Printing normally...");
            }
        }

        Ok(Introduction {
            times: 0,
            name: name.to_string(),
            age: age.to_string()
        })
    }
}

impl Iterator for Introduction {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let max_times = 5;
        if self.times < max_times {
            self.times += 1;
            Some(self.times)
        } else {
            None
        }
    }
}

pub trait Display {
    fn print(&self) -> String;
}

impl Display for Introduction {
    fn print(&self) -> String {
        format!("my name is {}, my age is {}", self.name, self.age)
    }
}

fn main() {
    let hello = Introduction::new();
    // let Introduction { times: _, name, age } = Introduction::new();

    // println!("{:?}", hello.next());

    // println!("Hello, I'm an ai, my name is {0}, my age is {1}.", name, age);
    println!("Hello, I'm an ai, {}.", hello.print());
    for _item in hello {
        println!("Have a good day!");
    }
}