# QAMS Scorecard CSV Representation

QAMS uses the [CSV](https://en.wikipedia.org/wiki/Comma-separated_values) format to import scorecards. In the CSV scorecard representation:
- The header row stores the names of the options available on all criteria in the scorecard. They must be unique
- Each subsequent row represents a criterion in the scorecard
- The header column stores the names of the criteria. They must also be unique
- Each subsequent column represents a criterion option
- Each cell (excluding the header row and column) can either:
    - Store a number (the number of points associated with the given option on the given criterion)
    - Store the character `N` (for `NotApplicable`)
    - Store the character `F` (for `Autofail`)
    - Be left empty (indicating the option isn't available on the criterion).