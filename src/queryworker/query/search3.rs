#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    /// Search all fields
    Everything,
    /// Restrict results to songs matching by title only
    ByTitle,
    /// Restrict results to songs matching by artist only
    ByArtist,
    /// Restrict results to songs matching by album only
    ByAlbum,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Search3Params {
    pub query: String,
    pub mode: SearchMode,
}
