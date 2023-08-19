use std::collections::HashMap;
use handle_errors::Error; // internal library

// to capture the values of the *start & *end values in the URL
/// Pagination number range that is being extracted from query params
#[derive(Debug)]
pub struct Pagination {
    /// the index of the last item to be returned
    pub limit: Option<u32>
    /// the index of the first item to be returned
    pub offset: u32
}

// performs checks on the Pagination struct & returns the struct if okay
/// Extract query parameters from the '/questions' route
/// GET requests to this route can extract a range of questions
/// `/questions?start=1&end=20`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.start, Some(1));
/// assert_eq!(p.end, 10);
/// ```
pub fn get_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // continue from here;
    if params.contains_key("limit") && params.contains_key("offset") {
        return Ok(Pagination {
            limit: Some(params.get("limit").unwrap().parse::<u32>().map_err(Error::ParseError)?),
            offset: params.get("offset").unwrap().parse::<u32>().map_err(Error::ParseError)?
        });
    }
    Err(Error::MissingParameters)
}