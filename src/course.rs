use polars::prelude::*;
use std::{collections::HashMap, ops::Deref, path::PathBuf, clone};

#[derive(Debug, serde::Serialize, clone::Clone)]
pub struct Course {
    title: String,
    sections: HashMap<String, u64>,
    prereq: Vec<String>,
}

impl Course {
    pub fn new(title: String, sections: HashMap<String, u64>, prereq: Vec<String>) -> Course {
        Course {
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


#[derive(Debug, serde::Serialize)]
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
                .or_insert_with(|| Course::new(title, HashMap::new(), prereq));

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

    pub fn head(&self, n: usize) -> HashMap<String, &Course> {
        self.iter()
            .take(n)
            .map(|(k, v)| (k.clone(), v))
            .collect::<HashMap<_, _>>()
    }
}

impl Deref for CourseMap {
    type Target = HashMap<String, Course>;

    fn deref(&self) -> &Self::Target {
        &self.courses
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

fn is_session_conflict(a: u64, b: u64) -> bool {
    a & b != 0
}

pub struct CourseTable {
    df: DataFrame,
}

impl CourseTable {
    pub fn new(file_path: PathBuf) -> CourseTable {
        let df = LazyCsvReader::new(file_path)
            .has_header(true)
            .finish()
            .unwrap()
            .with_column(col("SESSIONS").cast(DataType::UInt64))
            .collect()
            .unwrap();

        CourseTable { df }
    }

    pub fn code_starts_with(&self, code: &str) -> CourseTable {
        // filter by course code contains code
        let df = self
            .df
            .clone()
            .lazy()
            .filter(col("COURSE CODE").str().starts_with(lit(code)))
            .collect()
            .unwrap();

        CourseTable { df }
    }

    pub fn no_conflict_with(&self, courses: &CourseMap) -> CourseTable {
        let df = self.df.clone();

        let mask = df
            .column("SESSIONS")
            .unwrap()
            .u64()
            .unwrap()
            .into_iter()
            .map(|x| !courses.conflict_with(x.unwrap()))
            .collect::<Vec<_>>();

        // cast mask to ChunkedArray<BooleanType>
        let mask = mask.into_iter().collect::<BooleanChunked>();

        CourseTable {
            df: df.filter(&mask).unwrap(),
        }
    }

    pub fn get_course(&self, code: &str) -> Option<Course> {
        let df = self
            .df
            .clone()
            .lazy()
            .filter(col("COURSE CODE").str().starts_with(lit(code)))
            .collect()
            .unwrap();

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
