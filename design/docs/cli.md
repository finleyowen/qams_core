# QAMS Command Line Interface

The QAMS CLI is intended to be an intermediary step before implementing QAMS as a full-stack application. 

## Commands

### `init`

Sets up a new QAMS instance in the current directory.

**Options**:
- `-s`/`--path-to-scorecard` (required): path to scorecard as CSV (see [here](./scorecard_csv.md)).
- `-a`/`--path-to-agents` (required): path to a list of agents (as rows) and optional metadata about them (as columns). The first column must contain the agent's full name or some other unique identifier.
- `-r`/`--path-to-metadata` (optional): if included, reviews in this QAMS instance will have inputs for the metadata fields described in the report. The metadata should only consist of short-mid length strings data.

**Note 1**: see [Scorecard CSV Representation](./scorecard_csv.md)
**Note 2**: the review metadata CSV defines a list of string fields that each review in the QAMS instance will contain

Sets up the current directory as a QAMS instance with:
- `reviews` directory: will contain reviews as JSON representations (initially empty)
- `reports` directory: will contain reports as HTML representations (initially empty)
- `scorecard.html`: see [QAMS Review form](./scorecard_html.md)
- `.qams` hidden directory: contains
    - Internal representation of past reports used to compute future reports
    - **Note**: on Windows, this should actually be hidden by the `init` command. Feel free to use a subprocess to avoid unnecessary dependencies.

## `report`

**Options**:
- `-s`/`--start-date`
- `-e`/`--end-date`
- `-p`/`--path-to-previous-reports`

Generates a report from all reviews dated within the range specified. See [QAMS Reports](./reports.md).

## `update`

**Options**: same as options for [init](#init)

Allows the user to update the scorecard, agents, or review metadata for future reviews and reports.