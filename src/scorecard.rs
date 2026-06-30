use std::collections::HashMap;

type PointsType = u32;

pub trait ScorecardComponent<T: Clone> {

    fn get_numerator(&self, sel: T) -> PointsType;
    fn get_denominator(&self, sel: T) -> PointsType;
    fn is_autofail(&self, sel: T) -> bool;

    fn get_score(&self, sel: T) -> f64 {
        if self.is_autofail(sel.clone()) {
            return 0.0;
        }
        (self.get_numerator(sel.clone()) as f64) / self.get_denominator(sel.clone()) as f64
    }

}

#[derive(Clone)]
pub enum CriterionScore {
    Points(PointsType),
    Autofail,
    NotApplicable
}

pub struct Criterion {
    options: HashMap<String, CriterionScore>
}

impl ScorecardComponent<&str> for Criterion {
    fn get_numerator(&self, sel: &str) -> PointsType {
        match &self.options[sel] {
            CriterionScore::Points(n_points) => *n_points,
            _ => 0
        }
    }

    fn get_denominator(&self, sel: &str) -> PointsType {
        match &self.options[sel] {
            CriterionScore::Points(_) | CriterionScore::Autofail => {
                let mut denom = 0;
                for option in self.options.values() {
                    if let CriterionScore::Points(n_points) = option
                        && n_points > &denom {
                            denom = *n_points;
                    }
                }
                denom
            },
            _ => 0
        }
    }

    fn is_autofail(&self, sel: &str) -> bool {
        match &self.options[sel] {
            CriterionScore::Autofail => true,
            _ => false
        }
    }
}

impl Criterion {
    /// Constructs a `Criterion` from a map of option name to score type.
    pub fn new(options: HashMap<String, CriterionScore>) -> Self {
        Criterion { options }
    }

    pub fn get_avg_score(&self, sels: &Vec<&str>) -> f64 {
        let mut num = 0; let mut denom = 0;
        for sel in sels {
            num += self.get_numerator(sel);
            denom += self.get_denominator(sel);
        }
        if denom > 0 {
            num as f64 / denom as f64
        } else {
            100.0
        }
    }

    /// Returns the score type for a given option name, if it exists on this
    /// criterion (some options may be unavailable on a given criterion).
    pub fn option(&self, name: &str) -> Option<&CriterionScore> {
        self.options.get(name)
    }

    /// Iterates over all (option name, score) pairs defined on this criterion.
    pub fn options(&self) -> impl Iterator<Item = (&String, &CriterionScore)> {
        self.options.iter()
    }

    /// The highest `Points` value among this criterion's options (0 if none).
    /// This is the "full marks" value used both for scoring and for
    /// presentation (e.g. highlighting the best option in a review form).
    pub fn max_points(&self) -> PointsType {
        self.options.values()
            .filter_map(|s| if let CriterionScore::Points(p) = s { Some(*p) } else { None })
            .max()
            .unwrap_or(0)
    }
}

pub struct Scorecard {
    criteria: HashMap<String, Criterion>,
    /// Option names in CSV column order (needed for ordered HTML output).
    option_order: Vec<String>,
    /// Criterion names in CSV row order (needed for ordered HTML output).
    criterion_order: Vec<String>,
}

impl ScorecardComponent<&HashMap<String, String>> for Scorecard {
    fn get_denominator(&self, sel: &HashMap<String, String>) -> PointsType {
        let mut denom = 0;
        for (name, criterion) in &self.criteria {
            denom += criterion.get_denominator(&sel[name]);
        }
        denom
    }

    fn get_numerator(&self, sel: &HashMap<String, String>) -> PointsType {
        let mut num = 0;
        for (name, criterion) in &self.criteria {
            num += criterion.get_numerator(&sel[name]);
        }
        num
    }

    fn is_autofail(&self, sel: &HashMap<String, String>) -> bool {
        for (name, criterion) in &self.criteria {
            if criterion.is_autofail(&sel[name]) {
                return true;
            }
        }
        false
    }
}

impl Scorecard {
    /// Constructs a `Scorecard` directly from its parsed components.
    ///
    /// CSV parsing now lives in the CLI (`qams_cli`); this constructor lets
    /// any caller build a `Scorecard` from already-parsed data, keeping the
    /// internal `HashMap` fields private while still allowing construction
    /// from outside the crate.
    ///
    /// `option_order` and `criterion_order` must be consistent with the keys
    /// present in `criteria` (and each `Criterion`'s own options) — this is
    /// the caller's responsibility, mirroring how `from_csv_string` used to
    /// build them together.
    pub fn new(
        criteria: HashMap<String, Criterion>,
        option_order: Vec<String>,
        criterion_order: Vec<String>,
    ) -> Self {
        Scorecard { criteria, option_order, criterion_order }
    }

    /// Returns the criterion names in their original (CSV row) order.
    pub fn criterion_order(&self) -> &[String] {
        &self.criterion_order
    }

    /// Returns the option names in their original (CSV column) order.
    pub fn option_order(&self) -> &[String] {
        &self.option_order
    }

    /// Looks up a criterion by name.
    pub fn criterion(&self, name: &str) -> Option<&Criterion> {
        self.criteria.get(name)
    }

    /// Iterates over all (criterion name, criterion) pairs.
    pub fn criteria(&self) -> impl Iterator<Item = (&String, &Criterion)> {
        self.criteria.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::{Criterion, CriterionScore, Scorecard, ScorecardComponent};

    #[test]
    fn basic_criterion_test() {
        let crit = Criterion {
            options: HashMap::from([
                ("p1".into(), CriterionScore::Points(1)),
                ("p0".into(), CriterionScore::Points(0))
            ])
        };

        assert!(crit.get_numerator("p1") == 1);
        assert!(crit.get_numerator("p0") == 0);

        assert!(crit.get_denominator("p1") == 1);
        assert!(crit.get_denominator("p0") == 1);

        assert!(!crit.is_autofail("p1"));
        assert!(!crit.is_autofail("p0"));
    }

    #[test]
    fn na_criterion_test() {
        let crit = Criterion {
            options: HashMap::from([
                ("p1".into(), CriterionScore::Points(1)),
                ("p0".into(), CriterionScore::Points(0)),
                ("na".into(), CriterionScore::NotApplicable)
            ])
        };

        assert!(crit.get_numerator("p1") == 1);
        assert!(crit.get_numerator("p0") == 0);
        assert!(crit.get_numerator("na") == 0);

        assert!(crit.get_denominator("p1") == 1);
        assert!(crit.get_denominator("p0") == 1);
        assert!(crit.get_denominator("na") == 0);

        assert!(!crit.is_autofail("p1"));
        assert!(!crit.is_autofail("p0"));
        assert!(!crit.is_autofail("na"));
    }

    #[test]
    fn autofail_criterion_test() {
        let crit = Criterion {
            options: HashMap::from([
                ("p1".into(), CriterionScore::Points(1)),
                ("af".into(), CriterionScore::Autofail),
                ("na".into(), CriterionScore::NotApplicable)
            ])
        };

        assert!(crit.get_numerator("p1") == 1);
        assert!(crit.get_numerator("af") == 0);
        assert!(crit.get_numerator("na") == 0);

        assert!(crit.get_denominator("p1") == 1);
        assert!(crit.get_denominator("af") == 1);
        assert!(crit.get_denominator("na") == 0);

        assert!(!crit.is_autofail("p1"));
        assert!(crit.is_autofail("af"));
        assert!(!crit.is_autofail("na"));
    }

    #[test]
    fn get_avg_score_test() {
        let crit = Criterion {
            options: HashMap::from([
                ("p1".into(), CriterionScore::Points(1)),
                ("p0".into(), CriterionScore::Points(0)),
                ("na".into(), CriterionScore::NotApplicable),
            ])
        };

        // Two perfect scores → 1.0
        assert_eq!(crit.get_avg_score(&vec!["p1", "p1"]), 1.0);

        // One perfect, one zero → 0.5
        assert_eq!(crit.get_avg_score(&vec!["p1", "p0"]), 0.5);

        // All N/A → denom is 0, defaults to 100.0
        assert_eq!(crit.get_avg_score(&vec!["na", "na"]), 100.0);

        // Mix of N/A and points: N/A contributes 0 to both num and denom
        assert_eq!(crit.get_avg_score(&vec!["p1", "na"]), 1.0);
    }

    #[test]
    fn scorecard_test() {
        let sc = Scorecard {
            criteria: HashMap::from([
                ("crit1".into(), Criterion {
                    options: HashMap::from([
                        ("p1".into(), CriterionScore::Points(1)),
                        ("p0".into(), CriterionScore::Points(0))
                    ])
                }),
                ("crit2".into(), Criterion {
                    options: HashMap::from([
                        ("p1".into(), CriterionScore::Points(1)),
                        ("p0".into(), CriterionScore::Points(0)),
                        ("na".into(), CriterionScore::NotApplicable)
                    ])
                }),
                ("crit3".into(), Criterion {
                    options: HashMap::from([
                        ("p1".into(), CriterionScore::Points(1)),
                        ("p0".into(), CriterionScore::Points(0)),
                        ("na".into(), CriterionScore::NotApplicable),
                        ("af".into(), CriterionScore::Autofail)
                    ])
                })
            ]),
            option_order: vec!["p1".into(), "p0".into(), "na".into(), "af".into()],
            criterion_order: vec!["crit1".into(), "crit2".into(), "crit3".into()],
        };

        let sel1: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "p1".into()),
            ("crit2".into(), "p1".into()),
            ("crit3".into(), "p1".into())
        ]);
        assert!(sc.get_score(&sel1) == 1.0);

        let sel2: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "p1".into()),
            ("crit2".into(), "na".into()),
            ("crit3".into(), "na".into())
        ]);
        assert!(sc.get_score(&sel2) == 1.0);

        let sel3: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "p1".into()),
            ("crit2".into(), "na".into()),
            ("crit3".into(), "af".into())
        ]);
        assert!(sc.get_score(&sel3) == 0.0);
    }

    #[test]
    fn new_constructor_roundtrip() {
        let mut criteria = HashMap::new();
        criteria.insert(
            "crit1".to_string(),
            Criterion::new(HashMap::from([
                ("YES".to_string(), CriterionScore::Points(1)),
                ("NO".to_string(), CriterionScore::Points(0)),
            ])),
        );
        let sc = Scorecard::new(
            criteria,
            vec!["YES".to_string(), "NO".to_string()],
            vec!["crit1".to_string()],
        );

        assert_eq!(sc.criterion_order(), &["crit1".to_string()]);
        assert_eq!(sc.option_order(), &["YES".to_string(), "NO".to_string()]);
        assert!(sc.criterion("crit1").is_some());
        assert!(sc.criterion("nonexistent").is_none());

        let sel: HashMap<String, String> = HashMap::from([("crit1".into(), "YES".into())]);
        assert_eq!(sc.get_score(&sel), 1.0);
    }

    #[test]
    fn criterion_accessors() {
        let crit = Criterion::new(HashMap::from([
            ("YES".to_string(), CriterionScore::Points(1)),
            ("NO".to_string(), CriterionScore::Points(0)),
            ("N/A".to_string(), CriterionScore::NotApplicable),
        ]));

        assert_eq!(crit.max_points(), 1);
        assert!(matches!(crit.option("YES"), Some(CriterionScore::Points(1))));
        assert!(crit.option("MISSING").is_none());
        assert_eq!(crit.options().count(), 3);
    }
}