extern crate pest;
#[macro_use]
extern crate pest_derive;
mod drivers;
mod version;

use pest::Parser;
use version::Version;

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

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
    }
}
