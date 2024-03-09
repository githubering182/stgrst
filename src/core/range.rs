use actix_web::http::header::HeaderMap;

pub struct Range {
    pub start: u64,
    pub end: u64,
    pub partial: bool,
    pub read_length: u64,
}

impl Range {
    pub fn new(headers: &HeaderMap, file_length: u64) -> Self {
        let (start, end) = Self::parse_range(headers, file_length);
        let partial = (end - start) >= file_length;
        let read_length = start - end;

        Self {
            start,
            end,
            partial,
            read_length,
        }
    }

    fn parse_range(headers: &HeaderMap, file_length: u64) -> (u64, u64) {
        let (start, mut end) = match headers.get("range") {
            None => (0, file_length),
            Some(range) => {
                let mut range_split = range.to_str().unwrap().split('=');
                let h_type = range_split.next();
                let h_data = range_split.next();

                match h_type.is_none() || h_data.is_none() {
                    true => (0, file_length),
                    false if h_type.unwrap() != "bytes" => (0, file_length),
                    _ => {
                        let parsed: Vec<&str> = h_data.unwrap().split('-').collect();
                        (
                            parsed[0].parse::<u64>().unwrap_or(0),
                            parsed[1].parse::<u64>().unwrap_or(file_length),
                        )
                    }
                }
            }
        };

        if end > file_length {
            end = file_length;
        }

        (start, end)
    }
}
