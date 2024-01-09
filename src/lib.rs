mod course;
mod scheduler;

#[cfg(test)]
mod tests {
    use course::{CourseMap, CourseTable};
    use std::{collections::HashMap, path::PathBuf, ops::Deref};

    use super::*;

    #[test]
    fn it_works() {
        let table = CourseTable::load(PathBuf::from("data.csv"));
        println!("{}", table);
    }

    #[test]
    fn test_session() {
        let table = CourseTable::load(PathBuf::from("data.csv"));

        let course = table.get_course("COMP1117").unwrap();
        let table = table.to_lazy().no_conflict_with(course).collect().unwrap();
        println!("{}", table);

        let courses = table.get_courses(&["COMP1117", "COMP2113"]).unwrap();
        let table = table.to_lazy().no_conflict_with(courses).collect().unwrap();
        println!("{}", table);
    }

    #[test]
    fn test_course_code() {
        let table = CourseTable::load(PathBuf::from("data.csv"))
            .to_lazy()
            .contains(&["COMP"])
            .collect()
            .unwrap();
        println!("{}", table);
    }

    #[test]
    fn test_get_courses() {
        let table = CourseTable::load(PathBuf::from("data.csv"));
        let courses = table.get_courses(&["COMP1117", "COMP2113"]);
        println!("{:?}", courses);
    }

    #[test]
    fn test_course_code_session() {
        let table = CourseTable::load(PathBuf::from("data.csv"));
        let mut courses = CourseMap::new(HashMap::new());
        courses.add(
            "COMP1117".to_string(),
            table.get_section("COMP1117", "1A").unwrap(),
        );

        let table = table
            .to_lazy()
            .fall()
            .contains(&["COMP"])
            .no_conflict_with(courses.clone())
            .collect()
            .unwrap();
        println!("{}", table);
        println!("{:?}", courses);
    }

    #[test]
    fn test_semester() {
        let table = CourseTable::load(PathBuf::from("data.csv"))
            .to_lazy()
            .spring()
            .collect()
            .unwrap();
        println!("{}", table);
    }

    #[test]
    fn test_course_map() {
        let table = CourseTable::load(PathBuf::from("data.csv"));
        let map = CourseMap::from(table);
        println!("{:?}", map.get("COMP1117"));
        println!("{:?}", map.get_session("COMP1117", "1A"));
    }

    #[test]
    fn test_lazy() {
        let table = CourseTable::load(PathBuf::from("data.csv"));
        let mut current_courses = CourseMap::new(HashMap::new());
        current_courses.add(
            "A".to_string(),
            table.get_section("COMP1117", "1A").unwrap(),
        );
        current_courses.extend(table.get_courses(&["COMP2113"]).unwrap());
        let table = table
            .to_lazy()
            .contains(&["COMP", "MATH", "ENGG"])
            .semester(1)
            .no_conflict_with(table.get_course("COMP1117").unwrap())
            .no_prereq(current_courses.clone())
            .collect()
            .unwrap();
        println!("{}", table);
    }
}
