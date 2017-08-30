use hyper::{Client, Request, Response, Method, Body, Result, Error};
use hyper::client::{HttpConnector, FutureResponse};
use hyper::header::ContentType;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use futures::{future, Future, Stream};

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

    pub fn create_request<'c>(&self, uri: &'c str, body: Option<Vec<u8>>) -> Request {
        let uri = (self.host.clone() + uri).parse().expect("could not parse uri");

        let mut req = Request::new(Method::Get, uri);
        req.headers_mut().set_raw("Authorization", self.password.clone());

        if let Some(body) = body {
            req.set_body(body);
        }

        req
    }

    pub fn run_request(&mut self, request: Request) -> Vec<u8> {
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
}

/*pub fn load_tracks(http_client: &HttpClient, identifier: &str) {
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let uri = format!("{}/loadtracks?identifier={}", host, identifier).parse()
        .expect("could not parse uri");

    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set_raw("Authorization", password);

    let task = client.request(req).map(|res| {
        println!("status: {}", res.status());
    });

    core.run(task).expect("an error occured when sending a request");
}*/