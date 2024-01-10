use coursehku::course::{CourseMap, CourseTable};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

fn print_help() {
    println!("Commands:");
    println!("  ADD <course code> [section]");
    println!("  REMOVE <course code>");
    println!("  CLEAR");
    println!("  FIND [course code]");
    println!("  SCHEDULE");
    println!("  EXIT");
}

fn main() {
    let table = CourseTable::load(PathBuf::from("data.csv"));
    let mut courses = CourseMap::new(HashMap::new());
    loop {
        let course_codes: Vec<String> = courses.keys().cloned().collect();
        println!("Current courses: {:?}", course_codes);

        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let input = input.trim().to_uppercase();
        let input: Vec<&str> = input.split_whitespace().collect();

        if input.len() == 0 {
            print!("\x1B[2J\x1B[1;1H");
            println!("Please enter a command");
            print_help();
            continue;
        }

        match input[0] {
            "ADD" | "+" => {
                if input.len() == 2 {
                    let course = table.get_course(input[1]);
                    if course.is_none() {
                        println!("Course not found");
                        continue;
                    }
                    courses.add(input[1].to_string(), course.unwrap());
                } else if input.len() == 3 {
                    let course = table.get_section(input[1], input[2]);
                    if course.is_none() {
                        println!("Course not found");
                        continue;
                    }
                    courses.add(input[1].to_string(), course.unwrap());
                } else {
                    println!("Invalid command");
                }
            }
            "REMOVE" | "-" => {
                courses.remove(input[1]);
            }
            "CLEAR" | "--" => {
                courses.clear();
            }
            "FIND" | "LS" => {
                print!("\x1B[2J\x1B[1;1H");
                let mut lazy = table.to_lazy().no_conflict_with(courses.clone());
                if input.len() == 2 {
                    lazy = lazy.contains(&input[1..]);
                }
                let table = lazy.collect().unwrap();
                println!("{}", table);
            }
            "SCHEDULE" | "S" => {
                print!("\x1B[2J\x1B[1;1H");
                println!("{}", courses);
                courses
                    .keep_no_conflict()
                    .unwrap()
                    .iter()
                    .for_each(|course| {
                        println!("{}", course);
                    });
            }
            "EXIT" => {
                break;
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
                println!("Invalid command");
                print_help();
            }
        }
    }
}
