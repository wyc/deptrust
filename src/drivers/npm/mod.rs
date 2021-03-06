mod npm_semver;

use crate::pest;              
#[macro_use]              
use crate::pest_derive;
use lazy_static::lazy_static;

use std::collections::BTreeMap as Map; // BTreeMap is ordered

use serde::{Serialize, Deserialize};
use serde_json;
use serde_json::{Value, json};
use chrono::prelude::*;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};                        
use regex::Regex;

use crate::version::{Version, VersionQuery as VQ, SemVer, SemVerField};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageJson {
    // https://docs.npmjs.com/files/package.json
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<Repository>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    bugs: Option<Bugs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    main: Option<String>, // "main.js"
    #[serde(skip_serializing_if = "Option::is_none")]
    browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin: Option<Bin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    man: Option<Man>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    directories: Option<Directories>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scripts: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev_dependencies: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_dependencies: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundled_dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_dependencies: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engines: Option<Map<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publish_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefer_global: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bugs {
    url: Option<String>,
    email: Option<String>,
}

impl Bugs {
    fn validate(self) {
        if self.url == None && self.email == None {
            println!("must provide one or both of `bugs.url`, `bugs.email`");
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]                                                  
enum Bin {
    String(String),              // "./path/to/program"
    Object(Map<String, String>), // {"my-program": "./path/to/program"}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]                                                  
enum Man {
    String(String),     // "./man/foo.1"
    Array(Vec<String>), // ["./man/foo.1", "./man/bar.1"]
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Directories {
    #[serde(skip_serializing_if = "Option::is_none")]
    lib: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    man: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    test: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]                                                  
enum Repository {
    URL(String),
    RepositoryEntry(RepositoryEntry),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryEntry {
    type_: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    directory: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use std::io::BufReader;

    #[test]
    fn test_parse_npm() {
        let path = "src/drivers/npm/fixtures/npm-6.14.5-package.json";
        // let path = env::current_dir().unwrap();
        // println!("{}", path.to_str().unwrap());
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.is_file());
        let file = fs::File::open(path).unwrap();
        let reader = BufReader::new(file);
        let package_json: PackageJson = serde_json::from_reader(reader).unwrap();
        assert!(package_json.version == "6.14.5");
        let repo = match package_json.repository.unwrap() {
            Repository::RepositoryEntry(re) => re,
            Repository::URL(_) => panic!("expected RepositoryEntry, not URL"),
        };
        assert!(repo.type_ == "git");
        assert!(repo.url == "https://github.com/npm/cli");
        let bin = match package_json.bin.unwrap() {
            Bin::Object(o) => o,
            Bin::String(_) => panic!("expected Many, not Single"),
        };
        let bin_keys: Vec<_> = bin.keys().cloned().collect();
        assert_eq!(bin_keys, ["npm", "npx"]);
        let deps = package_json.dependencies.unwrap();
        assert_eq!(deps.get("JSONStream").unwrap(), "^1.3.5");
        let bundle_deps = package_json.bundle_dependencies.unwrap();
        assert_eq!(bundle_deps.len(), 123);
        let scripts = package_json.scripts.unwrap();
        assert_eq!(scripts.get("licenses").unwrap(),
                   "licensee --production --errors-only");
    }

    #[test]
    fn test_parse_express() {
        let path = "src/drivers/npm/fixtures/express-4.17.1-package.json";
        // let path = env::current_dir().unwrap();
        // println!("{}", path.to_str().unwrap());
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.is_file());
        let file = fs::File::open(path).unwrap();
        let reader = BufReader::new(file);
        let package_json: PackageJson = serde_json::from_reader(reader).unwrap();
    }
}
