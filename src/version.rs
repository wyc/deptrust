use pest::Parser;
#[macro_use]
use pest_derive;

#[derive(Debug, PartialEq)]
pub enum VersionQuery {
    And(Box<Self>, Box<Self>),   // >1.2.3 <2.0
    Or(Box<Self>, Box<Self>),    // >1.0.0 || >=2.3.1
    Range(Box<Self>, Box<Self>), // 1.2.3 - 1.9
    Not(Box<Self>),              // !1.2.5
    Lt(Version),     // <1.2.3
    Lte(Version),    // <=1.2.3
    Gt(Version),     // >1.2.3
    Gte(Version),    // >=1.2.3
    Eq(Version),     // =1.2.3
    Approx(Version), // ~1.2.3
    Compat(Version), // ^1.2.3
    Version(Version), // 1.2.3
}

#[derive(Debug, PartialEq)]
pub struct SemVer {
    // https://semver.org/
    pub major: SemVerField,
    pub minor: SemVerField,
    pub patch: SemVerField,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum SemVerField {
    Number(u16),
    Wildcard,
    Missing,
}

#[derive(Debug, PartialEq)]
pub enum Version {
    SemVer(SemVer),
    Missing,
}

pub struct NumVer {
    // 1.02.3
    numbers: Vec<u8>,
}

pub type StrVer = String;

pub struct CalVer {
    // https://calver.org/
    year: u8,
    month: Option<u8>,
    week: Option<u8>,
    day: Option<u8>,
    major: Option<u8>,
    minor: Option<u8>,
    micro: Option<u8>,
    build: Option<String>,
    pre_release: Option<String>,
}

#[derive(Parser)]
#[grammar = "semver.pest"]
pub struct SemVerParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_successful_parse() {
        let inputs = vec!["1.2.3", "1.2.3-alpha", "1.2.3-alpha+001", "1.2.3+exp.46"];
        for i in inputs {
            match SemVerParser::parse(Rule::valid_semver, i) {
                Ok(p) => assert_eq!(p.as_str(), i), // ensure complete parsing
                Err(e) => panic!("error: {}", e),
            }
        }
    }

    #[test]
    fn test_semver_unsuccessful_parse() {
        let inputs = vec!["this is not a semver", "2.0", "1.2.3-+r46-a"];
        for i in inputs {
            match SemVerParser::parse(Rule::valid_semver, i) {
                Ok(p) => {
                    if p.as_str() != i {
                        // dangling chars means incomplete parse, this is fine
                        continue
                    } else {
                        panic!("should not have been successful: {} => {:#?}", i, p)
                    }
                },
                Err(_) => (),
            }
        }
    }
}
