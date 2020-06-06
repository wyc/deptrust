use crate::pest;              
#[macro_use]              
use crate::pest_derive;
use lazy_static::lazy_static;
use std::collections::BTreeMap as Map; // BTreeMap is ordered
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};                        
use regex::Regex;

use crate::version::{Version, VersionQuery as VQ, SemVer, SemVerField};

#[derive(Parser)]
#[grammar = "drivers/npm/npm_semver.pest"]
struct NpmSemVerParser;

lazy_static! {
    static ref VQ_PREC_CLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        use Assoc::*;
        PrecClimber::new(vec![
            Operator::new(Rule::logical_or, Left),
            Operator::new(Rule::logical_and, Left),
            Operator::new(Rule::hyphen, Left),
        ])
    };
}

fn coerce_partial(pair: Pair<Rule>) -> Version {
    // https://docs.npmjs.com/misc/semver#coercion
    assert_eq!(pair.as_rule(), Rule::partial);

    fn to_field(maybe_xr: Option<Pair<Rule>>) -> SemVerField {
        match maybe_xr {
            Some(xr) => {
                assert_eq!(xr.as_rule(), Rule::xr);
                match xr.into_inner().next() {
                    Some(nr) => {
                        assert_eq!(nr.as_rule(), Rule::nr);
                        SemVerField::Number(nr.as_str().parse::<u16>().unwrap())
                    },
                    None => SemVerField::Wildcard,
                }
            },
            None => SemVerField::Missing,
        }
    }
    let mut pairs = pair.into_inner();
    let major = to_field(pairs.next());
    let minor = to_field(pairs.next());
    let patch = to_field(pairs.next());
    let mut pre_release = None;
    let mut build = None;
    if let Some(qualifier_pair) = pairs.next() {
        let mut qps = qualifier_pair.into_inner();
        while let Some(p) = qps.next() {
            match p.as_rule() {
                Rule::pre => { pre_release = Some(String::from(p.as_str())); }
                Rule::build => { build = Some(String::from(p.as_str())); }
                _ => unreachable!(),
            }
        }
    }
    Version::SemVer(SemVer { major, minor, patch, pre_release, build })
}

fn eval_vq(expression: Pairs<Rule>) -> VQ {
    /*let mut pexpr = expression.clone();
    while let Some(p) = pexpr.next() {
        println!("got rule: {:#?}", p.as_rule());
    }*/
    VQ_PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::range_set => eval_vq(pair.into_inner()),
            Rule::range => eval_vq(pair.into_inner()),
            Rule::hyphen_range => eval_vq(pair.into_inner()),
            Rule::simple => eval_vq(pair.into_inner()),
            Rule::partial => VQ::Version(coerce_partial(pair)),
            Rule::primitive => {
                let mut primitive_inner = pair.into_inner();
                let cmp_husk = primitive_inner.next().unwrap();
                assert_eq!(cmp_husk.as_rule(), Rule::comparator);
                let mut cmp_item = cmp_husk.into_inner();
                let comparator = cmp_item.next().unwrap().as_rule();
                let partial = primitive_inner.next().unwrap();
                let v = coerce_partial(partial);
                match comparator {
                    Rule::gte => VQ::Gte(v),
                    Rule::lte => VQ::Lte(v),
                    Rule::gt  => VQ::Gt(v),
                    Rule::lt  => VQ::Lt(v),
                    Rule::eq  => VQ::Eq(v),
                    _ => unreachable!(),
                }
            },
            Rule::approx => VQ::Approx(coerce_partial(pair.into_inner().next().unwrap())),
            Rule::compat => VQ::Compat(coerce_partial(pair.into_inner().next().unwrap())),
            a => unreachable!("got to {:?}", a),
        },
        |lhs: VQ, op: Pair<Rule>, rhs: VQ| match op.as_rule() {
            Rule::hyphen      => VQ::Range(Box::new(lhs), Box::new(rhs)),
            Rule::logical_and => VQ::And(Box::new(lhs), Box::new(rhs)),
            Rule::logical_or  => VQ::Or(Box::new(lhs), Box::new(rhs)),
            _ => unreachable!(),
        },
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npm_semver() {
        let inputs = vec![
            "1.2.3", "1.2.3-alpha", "1.2.3-alpha+001", "1.2.3+exp.46",
            "1.2.0 - 1.3.0",
            ">=1.2.3", "<=1.2.3", "1.2.3 || 1.2.4",
            ">=1.2.3 <2.0",
        ];
        for i in inputs {
            match NpmSemVerParser::parse(Rule::range_set, i) {
                Ok(p) => assert_eq!(p.as_str(), i), // ensure complete parsing
                Err(e) => panic!("error: {}", e),
            }
        }
    }

    #[test]
    fn test_coerce_partial() {
        use SemVerField::*;
        let mut expected_io = Map::new();
        expected_io.insert("1", Version::SemVer(SemVer{
            major: Number(1), minor: Missing, patch: Missing,
            pre_release: None, build: None,
        }));
        expected_io.insert("1.2", Version::SemVer(SemVer{
            major: Number(1), minor: Number(2), patch: Missing,
            pre_release: None, build: None,
        }));
        expected_io.insert("1.2.3", Version::SemVer(SemVer{
            major: Number(1), minor: Number(2), patch: Number(3),
            pre_release: None, build: None,
        }));
        expected_io.insert("1.2.3.4", Version::SemVer(SemVer{
            major: Number(1), minor: Number(2), patch: Number(3),
            pre_release: None, build: None,
        }));
        expected_io.insert("1.2.3-alpha+r77", Version::SemVer(SemVer{
            major: Number(1), minor: Number(2), patch: Number(3),
            pre_release: Some("alpha".to_string()), build: Some("r77".to_string()),
        }));
        for (k, v) in expected_io.iter() {
            let mut pair = match NpmSemVerParser::parse(Rule::partial, k) {
                Ok(p) => p,
                Err(e) => panic!("error: {}", e),
            };
            assert_eq!(*v, coerce_partial(pair.next().unwrap()));
        }
    }

    #[test]
    fn test_eval_vq_gte() {
        use SemVerField::*;
        let input = ">=5.2";
        let mut pair = match NpmSemVerParser::parse(Rule::primitive, input) {
            Ok(p) => p,
            Err(e) => panic!("error: {}", e),
        };
        let vq = eval_vq(pair);
        if let VQ::Gte(Version::SemVer(v)) = vq {
            assert_eq!(v.major, Number(5));
            assert_eq!(v.minor, Number(2));
            assert_eq!(v.patch, Missing);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eval_vq_compat() {
        use SemVerField::*;
        let input = "^5.1.6";
        let mut pair = match NpmSemVerParser::parse(Rule::compat, input) {
            Ok(p) => p,
            Err(e) => panic!("error: {}", e),
        };
        let vq = eval_vq(pair);
        if let VQ::Compat(Version::SemVer(v)) = vq {
            assert_eq!(v.major, Number(5));
            assert_eq!(v.minor, Number(1));
            assert_eq!(v.patch, Number(6));
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_eval_vq_logical() {
        use VQ::{And, Or};
        use SemVerField::*;
        let input = "^5.1.6 ^6.1.2 || <=1.*";
        let mut pairs = match NpmSemVerParser::parse(Rule::range_set, input) {
            Ok(p) => p,
            Err(e) => panic!("error: {}", e),
        };
        let vq = eval_vq(pairs);
        let expected_vq = Or(
            Box::new(And(
                Box::new(VQ::Compat(Version::SemVer(SemVer {
                    major: Number(5),
                    minor: Number(1),
                    patch: Number(6),
                    pre_release: None,
                    build: None,
                }))),
                Box::new(VQ::Compat(Version::SemVer(SemVer {
                    major: Number(6),
                    minor: Number(1),
                    patch: Number(2),
                    pre_release: None,
                    build: None,
                }))),
            )),
            Box::new(VQ::Lte(Version::SemVer(SemVer {
                major: Number(1),
                minor: Wildcard,
                patch: Missing,
                pre_release: None,
                build: None,
            }))),
        );
        assert_eq!(expected_vq, vq);
    }

    #[test]
    fn test_eval_vq_range() {
        use SemVerField::*;
        let input = "1 - 2";
        let mut pairs = match NpmSemVerParser::parse(Rule::hyphen_range, input) {
            Ok(p) => p,
            Err(e) => panic!("error: {}", e),
        };
        let vq = eval_vq(pairs);
        let expected_vq = VQ::Range(
            Box::new(VQ::Version(Version::SemVer(SemVer {
                major: Number(1),
                minor: Missing,
                patch: Missing,
                pre_release: None,
                build: None,
            }))),
            Box::new(VQ::Version(Version::SemVer(SemVer {
                major: Number(2),
                minor: Missing,
                patch: Missing,
                pre_release: None,
                build: None,
            }))),
        );
        assert_eq!(expected_vq, vq);
    }
}
