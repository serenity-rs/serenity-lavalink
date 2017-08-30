extern crate serde_json; // idk why this is required for serde_json's functions

use hyper::{Client, Request, Method, Body, Error};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use futures::{future, Future, Stream};
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub struct HttpClient<'a> {
    core: &'a mut Core,
    client: Client<HttpsConnector<HttpConnector>, Body>,
    host: String,
    password: String,
}

impl<'a> HttpClient<'a> {
    pub fn new<'b>(core: &'a mut Core, host: &'b str, password: &'b str) -> Self {
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        Self {
            core,
            client,
            host: host.to_owned(),
            password: password.to_owned(),
        }
    }

    fn create_request(&self, uri: &str, body: Option<Vec<u8>>) -> Request {
        let uri = (self.host.clone() + uri).parse().expect("could not parse uri");

        let mut req = Request::new(Method::Get, uri);
        req.headers_mut().set_raw("Authorization", self.password.clone());

        if let Some(body) = body {
            req.set_body(body);
        }

        req
    }

    fn run_request(&mut self, request: Request) -> Vec<u8> {
        let task = self.client.request(request).and_then(|response| {
            println!("response status: {}", response.status());

            // todo work out how the fuck this works
            response.body().fold(Vec::new(), |mut v: Vec<u8>, chunk| {
                v.extend(&chunk[..]);
                future::ok::<_, Error>(v)
            })
        });

        self.core.run(task).expect("an error occured when sending http request")
    }

    pub fn load_tracks(&mut self, identifier: &str) -> Vec<LoadedTrack> {
        // url encoding the identifier
        let identifier = utf8_percent_encode(identifier, DEFAULT_ENCODE_SET);

        let uri = format!("/loadtracks?identifier={}", identifier);
        let request = self.create_request(uri.as_ref(), None);

        let response = self.run_request(request);
        let deserialized: Vec<LoadedTrack> = serde_json::from_slice(&response).unwrap();

        deserialized
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