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
            CriterionScore::Points(_) => {
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

pub struct Scorecard {
    criteria: HashMap<String, Criterion>
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

use crate::{Criterion, CriterionScore, Scorecard, ScorecardComponent};

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
        assert!(crit.get_denominator("af") == 0);
        assert!(crit.get_denominator("na") == 0);

        assert!(!crit.is_autofail("p1"));
        assert!(crit.is_autofail("af"));
        assert!(!crit.is_autofail("na"));
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
            ])
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
}

//