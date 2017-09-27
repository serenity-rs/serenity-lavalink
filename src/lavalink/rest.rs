extern crate serde_json; // idk why this is required for serde_json's functions

use super::config::Config;

use std::io::Read;

use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use hyper::Error as HyperError;
use hyper::client::{Client, RequestBuilder, Body};
use hyper::header::{Headers, ContentType};
use hyper::method::Method;

pub struct HttpClient {
    client: Client,
    host: String,
    password: String,
}

impl HttpClient {
    pub fn new(config: &Config) -> Self {
        let client = Client::new();

        Self {
            client,
            host: config.http_host.clone(),
            password: config.password.clone(),
        }
    }

    fn create_request<'a>(&'a self, method: Method, uri: &str, body: Option<(&'a [u8], ContentType)>) -> RequestBuilder {
        let mut builder = self.client.request(method, &(self.host.clone() + uri));

        let mut headers = Headers::new();

        // cant use hyper::header::Authorization because it requires prefix of Basic or Bearer
        headers.set_raw("Authorization", vec![self.password.as_bytes().to_vec()]);

        match body {
            Some((body, content_type)) => {
                builder = builder.body(Body::BufBody(body, body.len()));
                headers.set(content_type);
            },
            None => {},
        }

        let builder = builder.headers(headers);

        builder
    }

    fn run_request(&self, request: RequestBuilder) -> Result<Vec<u8>, HyperError> {
        match request.send() {
            Ok(response) => {
                Ok(response.bytes().fold(Vec::new(), |mut v: Vec<u8>, chunk| {
                    match chunk {
                        Ok(b) => v.push(b), // append the byte to the vec
                        Err(e) => {
                            println!("error parsing response body chunk {:?}", e);
                            return v;
                        },
                    };

                    v // return the vec as the final result
                }))
            },
            Err(e) => Err(e),
        }
    }

    pub fn load_tracks<'a>(&self, identifier: &str) -> Result<Vec<LoadedTrack>, HyperError> {
        // url encoding the identifier
        let identifier = utf8_percent_encode(identifier, DEFAULT_ENCODE_SET);

        let uri = format!("/loadtracks?identifier={}", identifier);
        let request = self.create_request(Method::Get, uri.as_ref(), None);

        let response = match self.run_request(request) {
            Ok(response) => response, 
            Err(e) => return Err(e),
        };

        let deserialized: Vec<LoadedTrack> = serde_json::from_slice(&response).unwrap();

        Ok(deserialized)
    }

    #[allow(unused)]
    pub fn decode_track<'a>(&self, track: &str) -> Result<LoadedTrack, HyperError> {
        let uri = format!("/decodetrack?track={}", track);
        let request = self.create_request(Method::Get, uri.as_ref(), None);

        let response = match self.run_request(request) {
            Ok(response) => response,
            Err(e) => return Err(e),
        };

        let deserialized: LoadedTrackInfo = serde_json::from_slice(&response).unwrap();

        Ok(LoadedTrack {
            track: track.to_string(),
            info: deserialized,
        })
    }

    #[allow(unused)]
    pub fn decode_tracks<'a>(&self, tracks: Vec<String>) -> Result<Vec<LoadedTrack>, HyperError> {
        let tracks = serde_json::to_vec(&tracks).unwrap();
        let body = (tracks.as_ref(), ContentType::json());

        let request = self.create_request(Method::Post, "/decodetracks", Some(body));

        let response = match self.run_request(request) {
            Ok(response) => response,
            Err(e) => return Err(e),
        };

        let deserialized: Vec<LoadedTrack> = serde_json::from_slice(&response).unwrap();

        Ok(deserialized)
    }
}

#[derive(Debug, Deserialize)]
pub struct LoadedTrackInfo {
    pub title: String,
    pub author: String,
    pub length: i64,
    pub identifier: String,
    pub uri: String,
    #[serde(rename = "isStream")]
    pub is_stream: bool,
    #[serde(rename = "isSeekable")]
    pub is_seekable: bool,
    pub position: i64,
}

#[derive(Debug, Deserialize)]
pub struct LoadedTrack {
    pub track: String,
    pub info: LoadedTrackInfo,
}