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

impl Scorecard {
    /// Parses a scorecard from a CSV string.
    ///
    /// Format:
    /// - Header row: empty first cell, then option names (must be unique)
    /// - Each subsequent row: criterion name in first cell, then per-option values
    ///   - A number   → `Points(n)`
    ///   - `"N"`      → `NotApplicable`
    ///   - `"F"`      → `Autofail`
    ///   - empty      → option not available on this criterion (omitted)
    pub fn from_csv_string(csv: &str) -> Result<Self, String> {
        let mut lines = csv.lines();

        // --- header row ---
        let header_line = lines.next().ok_or("CSV is empty")?;
        let header_cells: Vec<&str> = header_line.split(',').collect();
        // first cell is the top-left corner (ignored)
        let option_names: Vec<&str> = header_cells[1..].to_vec();

        // Enforce unique option names
        {
            let mut seen = std::collections::HashSet::new();
            for name in &option_names {
                if !seen.insert(*name) {
                    return Err(format!("Duplicate option name: '{name}'"));
                }
            }
        }

        // --- criterion rows ---
        let mut criteria: HashMap<String, Criterion> = HashMap::new();

        for (row_idx, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let cells: Vec<&str> = line.split(',').collect();
            let crit_name = cells[0];
            if crit_name.is_empty() {
                return Err(format!("Row {} has an empty criterion name", row_idx + 2));
            }
            if criteria.contains_key(crit_name) {
                return Err(format!("Duplicate criterion name: '{crit_name}'"));
            }

            let mut options: HashMap<String, CriterionScore> = HashMap::new();

            for (col_idx, opt_name) in option_names.iter().enumerate() {
                let cell = cells.get(col_idx + 1).copied().unwrap_or("").trim();
                let score = match cell {
                    "" => continue, // option not available on this criterion
                    "N" => CriterionScore::NotApplicable,
                    "F" => CriterionScore::Autofail,
                    other => {
                        let points: PointsType = other.parse().map_err(|_| {
                            format!(
                                "Invalid cell value '{}' at criterion '{}', option '{}'",
                                other, crit_name, opt_name
                            )
                        })?;
                        CriterionScore::Points(points)
                    }
                };
                options.insert(opt_name.to_string(), score);
            }

            criteria.insert(crit_name.to_string(), Criterion { options });
        }

        Ok(Scorecard { criteria })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{Criterion, CriterionScore, Scorecard, ScorecardComponent};

    const VSC1_CSV: &str = include_str!("../test_artifacts/vsc1.csv");

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

    #[test]
    fn from_csv_vsc1() {
        let sc = Scorecard::from_csv_string(VSC1_CSV).expect("parse should succeed");

        // crit1: YES=1, NO=0, N/A=NotApplicable, FYI=1
        // crit3: YES=1, NO=0  (N/A and FYI absent)

        // Perfect score on both criteria
        let sel_perfect: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "YES".into()),
            ("crit3".into(), "YES".into()),
        ]);
        // denom = max(crit1) + max(crit3) = 1 + 1 = 2; num = 1 + 1 = 2
        assert_eq!(sc.get_score(&sel_perfect), 1.0);

        // Zero score
        let sel_zero: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "NO".into()),
            ("crit3".into(), "NO".into()),
        ]);
        assert_eq!(sc.get_score(&sel_zero), 0.0);

        // N/A on crit1 removes it from denominator; crit3 YES → 1/1 = 1.0
        let sel_na: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "N/A".into()),
            ("crit3".into(), "YES".into()),
        ]);
        assert_eq!(sc.get_score(&sel_na), 1.0);

        // FYI on crit1 counts as 1 point (same as YES); denom for crit1 = 1
        let sel_fyi: HashMap<String, String> = HashMap::from([
            ("crit1".into(), "FYI".into()),
            ("crit3".into(), "NO".into()),
        ]);
        // num = 1 + 0 = 1, denom = 1 + 1 = 2
        assert_eq!(sc.get_score(&sel_fyi), 0.5);
    }

    #[test]
    fn from_csv_duplicate_option_error() {
        let csv = ",A,A\ncrit1,1,0\n";
        assert!(Scorecard::from_csv_string(csv).is_err());
    }

    #[test]
    fn from_csv_duplicate_criterion_error() {
        let csv = ",YES,NO\ncrit1,1,0\ncrit1,1,0\n";
        assert!(Scorecard::from_csv_string(csv).is_err());
    }

    #[test]
    fn from_csv_invalid_cell_error() {
        let csv = ",YES,NO\ncrit1,1,BAD\n";
        assert!(Scorecard::from_csv_string(csv).is_err());
    }
}