use polars::{lazy::dsl::col, prelude::*};
use std::{clone, collections::HashMap, ops::Deref, path::PathBuf};

#[derive(Debug, serde::Serialize, clone::Clone)]
pub struct Course {
    code: String,
    title: String,
    // section -> session
    sections: HashMap<String, u64>,
    prereq: Vec<String>,
}

impl Course {
    pub fn new(
        code: String,
        title: String,
        sections: HashMap<String, u64>,
        prereq: Vec<String>,
    ) -> Course {
        Course {
            code,
            title,
            sections,
            prereq,
        }
    }

    pub fn conflict_with(&self, session: u64) -> bool {
        self.sections.values().any(|x| x & session != 0)
    }
}

impl Deref for Course {
    type Target = HashMap<String, u64>;

    fn deref(&self) -> &Self::Target {
        &self.sections
    }
}

#[derive(Debug, serde::Serialize, clone::Clone)]
pub struct CourseMap {
    // code -> course
    courses: HashMap<String, Course>,
}

impl CourseMap {
    pub fn new(courses: HashMap<String, Course>) -> CourseMap {
        CourseMap { courses }
    }

    pub fn add_course(&mut self, code: String, course: Course) {
        let section = course.sections;
        let course = self.courses.entry(code.clone()).or_insert_with(|| Course {
            sections: HashMap::new(),
            ..course
        });
        course.sections.extend(section);
    }

    pub fn from_df(df: &DataFrame) -> Result<CourseMap, polars::prelude::PolarsError> {
        let mut courses = HashMap::new();

        let df = df.select(&[
            "COURSE CODE",
            "COURSE TITLE",
            "CLASS SECTION",
            "SESSIONS",
            "PREREQ",
        ])?;

        for i in 0..df.height() {
            let row = df.get(i).unwrap();

            let code = row.get(0).unwrap().to_string().replace("\"", "");
            let title = row.get(1).unwrap().to_string().replace("\"", "");
            let section = row.get(2).unwrap().to_string().replace("\"", "");
            let session = row.get(3).unwrap().try_extract::<u64>()?;
            let prereq = row
                .get(4)
                .unwrap()
                .to_string()
                .replace("\"", "")
                .split('&')
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            let course = courses
                .entry(code.clone())
                .or_insert_with(|| Course::new(code, title, HashMap::new(), prereq));

            course.sections.insert(section, session);
        }

        Ok(CourseMap::new(courses))
    }

    pub fn get_session(&self, code: &str, section: &str) -> Option<u64> {
        self.get(code)
            .and_then(|course| course.get(section))
            .copied()
    }

    pub fn conflict_with(&self, session: u64) -> bool {
        self.values().any(|course| course.conflict_with(session))
    }
}

impl Deref for CourseMap {
    type Target = HashMap<String, Course>;

    fn deref(&self) -> &Self::Target {
        &self.courses
    }
}


impl From<Course> for CourseMap {
    fn from(course: Course) -> Self {
        let mut courses = HashMap::new();
        courses.insert(course.code.clone(), course);
        CourseMap::new(courses)
    }
}

impl From<DataFrame> for CourseMap {
    fn from(df: DataFrame) -> Self {
        CourseMap::from_df(&df).expect("failed to convert DataFrame to CourseMap")
    }
}

impl From<CourseTable> for CourseMap {
    fn from(table: CourseTable) -> Self {
        CourseMap::from_df(&table).expect("failed to convert CourseTable to CourseMap")
    }
}

pub struct CourseTable {
    df: DataFrame,
}

impl CourseTable {
    pub fn load(file_path: PathBuf) -> CourseTable {
        let df = LazyCsvReader::new(file_path)
            .has_header(true)
            .finish()
            .unwrap()
            .with_column(col("SESSIONS").cast(DataType::UInt64))
            .collect()
            .unwrap();

        CourseTable { df }
    }

    pub fn to_lazy(&self) -> LazyTable {
        LazyTable::new(self.df.clone().lazy())
    }

    pub fn get_course(&self, code: &str) -> Option<Course> {
        let df = self.to_lazy().contains(&[code]).collect().unwrap();
        CourseMap::from_df(&df).unwrap().get(code).cloned()
    }

    pub fn get_section(&self, code: &str, section: &str) -> Option<Course> {
        let df = self
            .df
            .clone()
            .lazy()
            .filter(col("COURSE CODE").str().starts_with(lit(code)))
            .filter(col("CLASS SECTION").str().starts_with(lit(section)))
            .collect()
            .unwrap();

        CourseMap::from_df(&df).unwrap().get(code).cloned()
    }

    pub fn get_courses(&self, codes: &[&str]) -> Result<CourseMap, polars::prelude::PolarsError> {
        let df = self.to_lazy().contains(codes).collect()?;
        CourseMap::from_df(&df)
    }
}

impl Deref for CourseTable {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.df
    }
}

impl std::fmt::Display for CourseTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.df)
    }
}

pub struct LazyTable {
    lf: LazyFrame,
}

impl LazyTable {
    pub fn new(lf: LazyFrame) -> Self {
        LazyTable { lf }
    }

    pub fn contains(self, codes: &[&str]) -> Self {
        // return all courses that starts with codes
        let regex = format!("^({})", codes.join("|"));

        let df = self
            .lf
            .filter(col("COURSE CODE").str().contains(lit(regex), false));

        LazyTable { lf: df }
    }

    pub fn no_conflict_with(self, courses: CourseMap) -> Self {
        let df = self.lf.filter(col("SESSIONS").map(
            move |s: Series| {
                Ok(s.u64()
                    .unwrap()
                    .into_iter()
                    .map(|x| Some(!courses.conflict_with(x.unwrap())))
                    .collect())
            },
            GetOutput::from_type(DataType::Boolean),
        ));

        LazyTable { lf: df }
    }

    pub fn semester(self, semester: i8) -> Self {
        let df = self
            .lf
            .filter(col("CLASS SECTION").str().starts_with(lit(semester)));

        LazyTable { lf: df }
    }

    pub fn fall(self) -> Self {
        self.semester(1)
    }

    pub fn spring(self) -> Self {
        self.semester(2)
    }

    pub fn collect(self) -> Result<CourseTable, polars::prelude::PolarsError> {
        let df = self.lf.collect()?;
        Ok(CourseTable { df })
    }
}

impl Deref for LazyTable {
    type Target = LazyFrame;

    fn deref(&self) -> &Self::Target {
        &self.lf
    }
}
