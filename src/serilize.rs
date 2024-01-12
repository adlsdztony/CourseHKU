use crate::course::{Course, CourseMap};


#[derive(serde::Serialize, Debug)]
pub struct Section {
    pub section: String,
    pub session: u64,
}


#[derive(serde::Serialize, Debug)]
pub struct CourseInfo {
    pub code: String,
    pub title: String,
    pub sections: Vec<Section>,
    pub prereq: String,
}

impl From<Course> for CourseInfo {
    fn from(course: Course) -> Self {
        course.to_couseinfo()
    }
}


#[derive(serde::Serialize, Debug)]
pub struct CourseList {
    pub courses: Vec<CourseInfo>,
}

impl From<CourseMap> for CourseList {
    fn from(map: CourseMap) -> Self {
        let mut courses = Vec::new();
        for course in map.values() {
            courses.push(course.to_couseinfo());
        }

        CourseList { courses }
    }
}
