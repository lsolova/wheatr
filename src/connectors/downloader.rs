use reqwest::{self, blocking::Client, header};
use std::io::{Error, ErrorKind};

pub fn download_content(url: &String, api_key: &String) -> Result<Vec<u8>, Error> {
    let client_builder = Client::builder();
    let mut headers = header::HeaderMap::new();
    headers.insert("api_key", header::HeaderValue::from_str(api_key.as_str()).unwrap());
    let client_result = client_builder.default_headers(headers).build();
    let client = match client_result {
        Ok(cl) => cl,
        Err(_err) => return Err(Error::new(ErrorKind::Other, "Client setup failed")),
    };

    let response_result = client.get(url).send();
    let mut response = match response_result {
        Ok(resp) => resp,
        Err(err) => return Err(Error::new(ErrorKind::Other, err)),
    };

    if response.status().is_success() {
        let mut content: Vec<u8> = vec![];
        match response.copy_to(&mut content) {
            Ok(_) => return Ok(content),
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Loading failed: {}", err),
                ));
            }
        };
    } else {
        return Err(Error::new(ErrorKind::Other, "Response status is unsuccess"));
    }
}
