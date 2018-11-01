// Usually this kind of stuff goes into the background thread, but this is only used in the UI. And I'm tired, so this'll stay here for the moment.
// TODO: Move this to the settings submodule of the background thread.
// TODO2: Make this update-proof.

extern crate serde_json;

use RPFM_PATH;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use error::Result;

/// Name of the file to load/save from.
const TABLES_STATE_FILE: &str = "table_state.json";

/// This struct keeps the current state of the "configurable" stuff from a TableView.
/// - Filter: Keeps the `String` used for the filter, the column filtered and if it's case sensitive or not.
/// - Search: Keeps the `String` used search, the `String` used to replace, the column filtered, if it's case sensitive or not and the currently selected match.
/// - Columns: Keeps the order the user sets for the columns.
#[derive(Serialize, Deserialize)]
pub struct TableState {
    pub filter_state: FilterState,
    pub search_state: SearchState,
    pub columns_state: ColumnsState,
}

/// This Struct stores the last state of the filter of a TableView.
#[derive(Serialize, Deserialize)]
pub struct FilterState {
    pub text: String,
    pub column: i32,
    pub is_case_sensitive: bool
}

/// This Struct stores the last state of the search widget of a TableView.
#[derive(Serialize, Deserialize)]
pub struct SearchState {
    pub search_text: String,
    pub replace_text: String,
    pub column: i32,
    pub is_case_sensitive: bool,
}

/// This Struct stores the last state of the columns of a TableView. For sorting_column, ascending is false, descending is true.
#[derive(Serialize, Deserialize)]
pub struct ColumnsState {
    pub sorting_column: (i32, bool),
    pub visual_order: Vec<(i32, i32)>,
    pub hidden_columns: Vec<i32>,
}

/// Implementation of TableState.
impl TableState {

    /// This function creates a BTreeMap with the different TableStates needed for RPFM.
    pub fn new() -> BTreeMap<Vec<String>, Self> {
        BTreeMap::new()
    }

    /// This function creates a single empty TableState.
    pub fn new_empty() -> Self {
        Self {
            filter_state: FilterState::new(String::new(), 0, false),
            search_state: SearchState::new(String::new(), String::new(), 0, false),
            columns_state: ColumnsState::new((-1, false), vec![], vec![]),
        }
    }

    /// This function takes a table_state.json file and reads it into a "TableState" object.
    pub fn load() -> Result<BTreeMap<Vec<String>, Self>> {

        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(TABLES_STATE_FILE));
        let file = BufReader::new(File::open(path)?);
        let states: BTreeMap<String, Self> = serde_json::from_reader(file)?;

        // We need to process the states because serde only admits Strings as key.
        let mut states_processed = BTreeMap::new();
        for entry in states { states_processed.insert(entry.0.split('\\').map(|x| x.to_owned()).collect(), entry.1); }
        Ok(states_processed)
    }

    /// This function takes the Settings object and saves it into a settings.json file.
    pub fn save(states: &BTreeMap<Vec<String>, Self>) -> Result<()> {

        // Try to open the settings file.
        let path = RPFM_PATH.to_path_buf().join(PathBuf::from(TABLES_STATE_FILE));
        let mut file = BufWriter::new(File::create(path)?);

        // Same than when loading. We have to process the states to make them compatible with serde.
        let mut states_processed = BTreeMap::new();
        for entry in states { states_processed.insert(entry.0.join("\\"), entry.1); }
        let states = serde_json::to_string_pretty(&states_processed);
        file.write_all(states.unwrap().as_bytes())?;

        // Return success.
        Ok(())
    }
}

/// Implementation of FilterState.
impl FilterState {

    /// This function creates the FilterState of a TableView.
    pub fn new(text: String, column: i32, is_case_sensitive: bool) -> Self {
        Self {
            text,
            column,
            is_case_sensitive
        }
    }
}

/// Implementation of SearchState.
impl SearchState {

    /// This function creates the SearchState of a TableView.
    pub fn new(search_text: String, replace_text: String, column: i32, is_case_sensitive: bool) -> Self {
        Self {
            search_text,
            replace_text,
            column,
            is_case_sensitive
        }
    }
}

/// Implementation of ColumnsState.
impl ColumnsState {

    /// This function creates the ColumnState of a TableView.
    pub fn new(sorting_column: (i32, bool), visual_order: Vec<(i32, i32)>, hidden_columns: Vec<i32>) -> Self {
        Self {
            sorting_column,
            visual_order,
            hidden_columns,
        }
    }
}
