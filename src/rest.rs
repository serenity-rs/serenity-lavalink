use hyper::client::{Body, Client, RequestBuilder};
use hyper::header::{ContentType, Headers};
use hyper::method::Method;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use serde_json;
use std::io::Read;
use super::nodes::NodeConfig;
use ::prelude::*;

#[derive(Debug, Default)]
pub struct HttpClient {
    client: Client,
    host: String,
    password: Vec<u8>,
}

impl HttpClient {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            client: Client::new(),
            host: config.http_host,
            password: config.password.into_bytes(),
        }
    }

    fn create_request<'a>(&'a self, method: Method, uri: &str, body: Option<(&'a [u8], ContentType)>) -> RequestBuilder {
        let mut builder = self.client.request(method, &(self.host.clone() + uri));

        let mut headers = Headers::new();

        // cant use hyper::header::Authorization because it requires prefix of Basic or Bearer
        headers.set_raw("Authorization", vec![self.password.clone()]);

        if let Some((body, content_type)) = body {
            builder = builder.body(Body::BufBody(body, body.len()));
            headers.set(content_type);
        }

        builder.headers(headers)
    }

    fn run_request(&self, request: RequestBuilder) -> Result<Vec<u8>> {
        match request.send() {
            Ok(response) => {
                Ok(response.bytes().fold(Vec::new(), |mut v: Vec<u8>, chunk| {
                    match chunk {
                        Ok(b) => v.push(b), // append the byte to the vec
                        Err(e) => {
                            error!("error parsing response body chunk {:?}", e);
                            return v;
                        },
                    };

                    v // return the vec as the final result
                }))
            },
            Err(e) => Err(From::from(e)),
        }
    }

    pub fn load_tracks(&self, identifier: &str) -> Result<Vec<LoadedTrack>> {
        // url encoding the identifier
        let identifier = utf8_percent_encode(identifier, DEFAULT_ENCODE_SET);

        let uri = format!("/loadtracks?identifier={}", identifier);
        let request = self.create_request(Method::Get, uri.as_ref(), None);

        self.run_request(request)
            .and_then(|resp| serde_json::from_slice(&resp).map_err(From::from))
            .map_err(From::from)
    }

    #[allow(unused)]
    pub fn decode_track(&self, track: &str) -> Result<LoadedTrack> {
        let uri = format!("/decodetrack?track={}", track);
        let request = self.create_request(Method::Get, uri.as_ref(), None);

        let response = self.run_request(request)?;

        let info = serde_json::from_slice(&response)?;

        Ok(LoadedTrack {
            track: track.to_string(),
            info,
        })
    }

    #[allow(unused)]
    pub fn decode_tracks(&self, tracks: Vec<String>) -> Result<Vec<LoadedTrack>> {
        let tracks = serde_json::to_vec(&tracks)?;
        let body = (tracks.as_ref(), ContentType::json());

        let request = self.create_request(Method::Post, "/decodetracks", Some(body));

        self.run_request(request)
            .and_then(|resp| serde_json::from_slice(&resp).map_err(From::from))
            .map_err(From::from)
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct LoadedTrack {
    pub track: String,
    pub info: LoadedTrackInfo,
}
