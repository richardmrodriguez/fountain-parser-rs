use std::{collections::HashMap, default};

use uuid::Uuid;

use crate::fountain_line::FNLine;

/// ONLY Contains ranges within a `SelfContained` partial line at the `global_index`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FNPartialLineRange {
    pub id: Option<Uuid>,

    pub global_index: Option<usize>,

    pub local_start: Option<usize>,
    pub local_end: Option<usize>,

    pub visible_fnline: Option<FNLine>, // "stripped out" line; excludes all the invisibles across this range
}
/// This range struct must start with an `OrphanedOpen` line and end with an `OrphanedClose` line.
/// May also start or end with an `OrphanedOpenAndClose` line
#[derive(Debug, Default, PartialEq, Clone)]
pub struct FNPartialMultilineRange {
    pub id: Option<Uuid>,

    pub global_start: Option<usize>,
    pub local_start: Option<usize>,

    pub global_end: Option<usize>,
    pub local_end: Option<usize>,
}
