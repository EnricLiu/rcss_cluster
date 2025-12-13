//! https://github.com/rcsoccersim/rcssserver/blob/master/src/csvsaver.cpp#L139

use crate::create_config;

create_config! (CsvSaverConfig, "CSVSaver", {
    version: &'static str,
    save: bool,
    filename: &'static str,
});

// impl Default for CsvSaverConfig {
//     fn default() -> Self {
//         Self {
//             /* CSVSaver Configuration file */
//
//             // CSVSaver::version
//             version: "19.0.0",
//
//             // CSVSaver::save
//             /* If save is on/true, then the saver will attempt to save the results
//             to the database.  Otherwise it will do nothing. */
//             save: false,
//
//             // CSVSaver::filename
//             /* The file to save the results to.  If this file does not exist it
//             will be created.  If the file does exist, the results will be appended
//             to the end. */
//             filename: "rcssserver.csv",
//         }
//     }
// }
