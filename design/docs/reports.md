# QAMS Reports

QAMS reports are generated using the `report` command, and represented as HTML. Specifically, a QAMS report should be a 

QAMS reports show insights about the reviews contained within the report, as well as how the report compares to other previous reports. The `accumulation_period` option (see [QAMS Options](./options.md) defines the number of previous reports on which the new report should be based).

A QAMS report should contain the following:
- `index.html`: Contains links to all of the other pages described below.
- An overall summary view: presents a percentage score for each criteria and for each report in the past representing the average score on that criteria in that report across the whole team. Should be presented in a tabular format: criteria as columns, reports as rows.
- An agent-review index: presents each agents average percentage score on each report. Agents as rows, reports as columns again.
- Individual agent pages: presents all information about an given agent's reviews; criteria as rows, reports as columns again. Where an agent has multiple reviews in a single report, they're presented side-by-side, in chronological order of when they were reviewed. The cells should contain the names of the criterion options selected by QA. Clicking the cells should reveal the (optional) comment left by QA (if applicable).