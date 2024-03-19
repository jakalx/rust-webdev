use std::collections::HashMap;

use crate::error;

#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, error::Error> {
    match (params.get("start"), params.get("end")) {
        (Some(start), Some(end)) => {
            return Ok(Pagination {
                start: start.parse::<usize>().map_err(error::Error::ParseError)?,
                end: end.parse::<usize>().map_err(error::Error::ParseError)?,
            })
        }
        (None, _) => Err(error::Error::MissingParameter("start".into())),
        (_, None) => Err(error::Error::MissingParameter("end".into())),
    }
}
