pub struct FNDocument {
    raw_fnlines: Vec<FNLine>,
    stripped_fnlines: Vec<FNLine>,

    // The key in this Hashmap is the index of a stripped line
    // The value in this HashMap is a tuple of start, end indices for raw_fnlines;
    // Any FNLine in the stripped_fnlines has
    // a corresponding line or set of lines in the raw_fnlines vec
    // this is why only the second part of the tuple is Optional.
    stripped_fnlines_map: HashMap<usize, (usize, Option<usize>)>,
}

impl FNDocument {
    /// When the editor makes some change, it may change a range of text from a local "stripped view,"
    /// But those changes need to be made as part of the "raw lines", so that the data can be saved in proper foutnain formatting.
    /// So, if the editor wants to delete characters 7 through 26 on stripped line 54,
    /// that might actually correspond to a non-consequtive
    /// set of characters 12 through 31 on raw line 56,
    /// because of potential inline Notes or Boneyards.
    pub fn get_raw_index_from_stripped_index(&self) {
        todo!()
    }
}
