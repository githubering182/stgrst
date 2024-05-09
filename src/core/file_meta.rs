use rocket::{
    http::HeaderMap,
    request::{FromRequest, Outcome, Request},
};

#[derive(Debug)]
pub struct FileMeta<'r> {
    pub _type: &'r str,
    pub extension: &'r str,
    pub name: &'r str,
}

impl<'r> FileMeta<'r> {
    pub fn new(headers: &'r HeaderMap) -> Self {
        let (_type, extension) = Self::parse_content_type(headers);
        let name = headers.get("x-file-name").next().unwrap_or("no_name");

        Self {
            _type,
            extension,
            name,
        }
    }

    fn parse_content_type(headers: &'r HeaderMap) -> (&'r str, &'r str) {
        let default = ("application", "octet-stream");

        match headers.get("content-type").next() {
            None => default,
            Some(content) => {
                let mut content_split = content.split('/');

                let file_type = content_split.next().unwrap_or(default.0);
                let file_extension = content_split.next().unwrap_or(default.1);

                (file_type, file_extension)
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FileMeta<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers();
        Outcome::Success(Self::new(headers))
    }
}
