use std::collections::BTreeMap;
use std::path::Path;

use crate::record::UserRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecordFixture {
    pub set: String,
    pub id: u64,
    pub name: String,
    pub note: String,
}

impl UserRecordFixture {
    pub fn to_user_record(&self) -> UserRecord {
        UserRecord {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

#[derive(Debug)]
pub enum FixtureError {
    Io(std::io::Error),
    BadLine {
        line_no: usize,
        line: String,
        reason: String,
    },
    BadId {
        line_no: usize,
        value: String,
    },
}

impl From<std::io::Error> for FixtureError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn load_user_record_fixtures(
    path: impl AsRef<Path>,
) -> Result<Vec<UserRecordFixture>, FixtureError> {
    let input = std::fs::read_to_string(path)?;
    parse_user_record_fixtures(&input)
}

pub fn parse_user_record_fixtures(input: &str) -> Result<Vec<UserRecordFixture>, FixtureError> {
    let mut fixtures = Vec::new();

    for (line_index, line) in input.lines().enumerate() {
        let line_no = line_index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "set\tid\tname\tnote" {
            continue;
        }

        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() != 4 {
            return Err(FixtureError::BadLine {
                line_no,
                line: line.to_string(),
                reason: "expected exactly 4 tab-separated fields: set, id, name, note".to_string(),
            });
        }

        let id = parts[1].parse::<u64>().map_err(|_| FixtureError::BadId {
            line_no,
            value: parts[1].to_string(),
        })?;

        fixtures.push(UserRecordFixture {
            set: parts[0].to_string(),
            id,
            name: parts[2].to_string(),
            note: parts[3].to_string(),
        });
    }

    Ok(fixtures)
}

pub fn group_by_set(fixtures: Vec<UserRecordFixture>) -> BTreeMap<String, Vec<UserRecordFixture>> {
    let mut sets = BTreeMap::new();

    for fixture in fixtures {
        sets.entry(fixture.set.clone())
            .or_insert_with(Vec::new)
            .push(fixture);
    }

    sets
}
