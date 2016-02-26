extern crate hyper;

use html5ever;
use html5ever::driver::BytesOpts;
use html5ever::encoding::label::encoding_from_whatwg_label;
use html5ever::tendril::TendrilSink;
use parser::Sink;
use self::hyper::client::{Client, Response};
use self::hyper::header::ContentType;
use self::hyper::mime::Attr::Charset;
use self::hyper::{Result, Url};
use tree::NodeRef;

/// Things than can be turned into an Hyper HTTP response.
pub trait IntoResponse {
    /// Make an HTTP request in necessary and return a response.
    fn into_response(self) -> Result<Response>;
}

impl IntoResponse for Response {
    fn into_response(self) -> Result<Response> { Ok(self) }
}

impl IntoResponse for Url {
    fn into_response(self) -> Result<Response> {
        Client::new().get(self).send()
    }
}

impl<'a> IntoResponse for &'a str {
    fn into_response(self) -> Result<Response> {
        Client::new().get(self).send()
    }
}

impl<'a> IntoResponse for &'a String {
    fn into_response(self) -> Result<Response> {
        Client::new().get(self).send()
    }
}

/// Additional methods for html5ever::Parser
pub trait ParserExt {
    /// Fetch an HTTP or HTTPS URL with Hyper and parse,
    /// giving the `charset` parameter of a `Content-Type` response header, if any,
    /// as a character encoding hint to html5ever.
    ///
    /// This feature (and the dependency on Hyper) is optional and not enabled by default.
    /// To enable it, your `Cargo.toml` file should look like:
    ///
    /// ```toml
    /// [dependencies]
    /// kuchiki = {version = "...", features = ["hyper"]}
    /// ```
    ///
    /// See [Cargo features](http://doc.crates.io/manifest.html#the-features-section).
    fn from_http<R: IntoResponse>(self, response: R) -> Result<NodeRef>;
}

impl ParserExt for html5ever::Parser<Sink> {
    fn from_http<R: IntoResponse>(self, response: R) -> Result<NodeRef> {
        let mut response = try!(response.into_response());
        let opts = BytesOpts {
            transport_layer_encoding: response.headers.get::<ContentType>()
                .and_then(|content_type| content_type.get_param(Charset))
                .and_then(|charset| encoding_from_whatwg_label(charset))
        };
        Ok(try!(self.from_bytes(opts).read_from(&mut response)))
    }
}
