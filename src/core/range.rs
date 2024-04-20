use rocket::{
    http::HeaderMap,
    request::{FromRequest, Outcome, Request},
};

pub struct Range {
    pub start: u64,
    pub end: Option<u64>,
    pub read_length: Option<u64>,
}

impl Range {
    pub fn new(headers: &HeaderMap) -> Self {
        let (start, end) = Self::parse_range(headers);
        let read_length = end.map(|_end| _end - start + 1);

        Self {
            start,
            end,
            read_length,
        }
    }

    fn parse_range(headers: &HeaderMap) -> (u64, Option<u64>) {
        match headers.get("range").next() {
            None => (0, None),
            Some(range) => {
                let mut range_split = range.split('=');

                match range_split.next() {
                    Some("bytes") => (),
                    _ => {
                        return (0, None);
                    }
                }

                let h_data = range_split.next();

                match h_data.map(Self::parse_data) {
                    Some(result) => result,
                    None => (0, None),
                }
            }
        }
    }

    fn parse_data(bytes_data: &str) -> (u64, Option<u64>) {
        let parsed = bytes_data.split('-').collect::<Vec<&str>>();
        let start = parsed[0].parse::<u64>().unwrap_or(0);

        if parsed.len() < 2 {
            return (start, None);
        }

        let end = match parsed[1].parse::<u64>() {
            Ok(end) => Some(end),
            _ => None,
        };

        (start, end)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Range {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();
        Outcome::Success(Self::new(headers))
    }
}
