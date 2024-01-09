
use std::path::PathBuf;

use crate::course::*;


struct Scheduler {
    courses: CourseMap,
    table: CourseTable,
}

impl Scheduler {
    pub fn load(file_path: PathBuf) -> Scheduler {
        let table = CourseTable::load(file_path);
        let courses = CourseMap::from(table.clone());
        Scheduler { courses, table }
    }
}
