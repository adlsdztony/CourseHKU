mod course;

#[cfg(test)]
mod tests {
    use course::{CourseMap, CourseTable};
    use std::{collections::HashMap, path::PathBuf};

    use super::*;

    #[test]
    fn it_works() {
        let table = CourseTable::new(PathBuf::from("data.csv"));
        println!("{}", table);
    }

    #[test]
    fn test_session() {
        let table = CourseTable::new(PathBuf::from("data.csv"));

        let mut courses = CourseMap::new(HashMap::new());
        courses.add_course(
            "COMP1117".to_string(),
            table.get_course("COMP1117").unwrap(),
        );

        let table = CourseTable::new(PathBuf::from("data.csv")).no_conflict_with(&courses);
        println!("{}", table);
    }

    #[test]
    fn test_course_code() {
        let table = CourseTable::new(PathBuf::from("data.csv")).code_starts_with("COMP");
        println!("{}", table);
    }

    #[test]
    fn test_course_code_session() {
        let table = CourseTable::new(PathBuf::from("data.csv"));
        let mut courses = CourseMap::new(HashMap::new());
        courses.add_course(
            "COMP1117".to_string(),
            table.get_section("COMP1117", "1A").unwrap(),
        );

        let table = table.code_starts_with("COMP").no_conflict_with(&courses);
        println!("{}", table);
    }

    #[test]
    fn test_course_map() {
        let table = CourseTable::new(PathBuf::from("data.csv"));
        let map: CourseMap = table.into();
        println!("{:?}", map.get("COMP1117"));
        println!("{:?}", map.get_session("COMP1117", "1A"));
    }
}
