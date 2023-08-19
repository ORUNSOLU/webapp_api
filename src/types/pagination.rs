use std::collections::HashMap;
use handle_errors::WarpError; // internal library

// to capture the values of the *start & *end values in the URL
// Pagination number range that is being extracted from query params
#[derive(Debug, Default)]
pub struct Pagination {
    // the no of items to return in addition to the `offset`   
    pub limit: Option<i32>,
    // the index of the first item to be returned
    pub offset: i32
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
pub fn get_pagination(params: HashMap<String, String>) -> Result<Pagination, WarpError> {
    if params.contains_key("limit") && params.contains_key("offset") {
        return Ok(Pagination {
            limit: Some(params.get("limit").unwrap().parse::<i32>().map_err(WarpError::ParseError)?),
            offset: params.get("offset").unwrap().parse::<i32>().map_err(WarpError::ParseError)?
        });
    }
    Err(WarpError::MissingParameters)
}