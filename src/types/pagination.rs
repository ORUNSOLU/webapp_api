use std::collections::HashMap;
use handle_error::Error; // internal library

// to capture the values of the *start & *end values in the URL
/// Pagination number range that is being extracted from query params
#[derive(Debug)]
pub struct Pagination {
    /// the index of the first item to be returned
    pub start: usize,
    /// the index of the last item to be returned
    pub end: usize
}

// performs checks on the Pagination struct & returns the struct if okay
/// Extract query parameters from the '/questions' route
/// GET requests to this route can extract a range of questions
/// `/questions?start=1&end=20`
/// # Example usage
/// ```rust
/// let mut query = HashMap::new();
/// query.insert("start".to_string(), "1".to_string());
/// query.insert("end".to_string(), "10".to_string());
/// let p = types::pagination::extract_pagination(query).unwrap();
/// assert_eq!(p.start, 1);
/// assert_eq!(p.end, 10);
/// ```
pub fn get_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params.get("start").unwrap().parse::<usize>().unwrap();
        let end = params.get("end").unwrap().parse::<usize>().unwrap();
        // check that the *end parameter is less than the total number of questions.
        if start < end && end < params.keys().count() {
        return Ok(
            Pagination {
                start: params.get("start").unwrap().parse::<usize>().map_err(Error::ParseError)?,
                end: params.get("end").unwrap().parse::<usize>().map_err(Error::ParseError)?
            }
        );
        }
    }
    Err(Error::MissingParameters)
}