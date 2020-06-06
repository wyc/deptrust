extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;



mod drivers;

struct PkgInfo {
    name: String,
    version: Version,
    description: Option<String>,
    keywords: Option<Vec<String>>,
    homepage: Option<String>,
    bugs: Option<Vec<Bugs>>,
    license: Option<String>,
    people: Option<Vec<Person>>,
    repository: Repository,
    dependencies: Vec<DepGroup>,
    scripts: Option<Vec<ScriptGroup>>,
}

struct ScriptGroup {
    name: String,
    scripts: Vec<String>,
}

struct Repository {
    // @TODO shortcut resolver for "github:user/repo"
    type_: String,
    url: String,
}

struct DepGroup {
    name: String, // default, production, build, test, etc.
    deps: Vec<Dep>,
}

struct Dep {
    name: String,
    version: Version,
}

struct Person {
    name: String,
    role: String,
    homepage: String,
    email: String,
}

enum Bugs {
    Email(String),
    URL(String),
}

enum VersionQuery {
    VersionReq(VersionReq),        // >1.2.3
    And(VersionReq, VersionReq),   // >1.2.3 <2.0
    Or(VersionReq, VersionReq),    // >1.0.0 || >=2.3.1
    Range(VersionReq, VersionReq), // 1.2.3 - 1.9
    Not(VersionReq),               // !1.2.5
}

struct VersionReq {
    comparator: Comparator,
    version: Version,
}

enum Comparator {
    LT,     // <1.2.3
    LTE,    // <=1.2.3
    GT,     // >1.2.3
    GTE,    // >=1.2.3
    EQ,     // =1.2.3
    APPROX, // ~1.2.3
    COMPAT, // ^1.2.3
}

enum Version {
    SemVer(SemVer), // 1.2.3-alpha+r866
    //CalVer(CalVer), // 2018.08-beta
    //StrVer(StrVer), // latest
    //NumVer(NumVer), // 1.02.3
    Missing,
}

struct SemVer {
    // https://semver.org/
    major: u8,
    minor: u8,
    patch: u8,
    build: Option<String>,
    pre_release: Option<String>,
}

struct NumVer {
    // 1.02.3
    numbers: Vec<u8>,
}

type StrVer = String;

struct CalVer {
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
struct SemVerParser;

fn main() {
    println!("Hello, world!");
}

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
