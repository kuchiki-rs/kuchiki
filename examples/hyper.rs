extern crate kuchiki;

#[cfg(not(feature = "hyper"))]
fn main() {
    // Intentionally trigger an unused_import warning,
    // with a message on the same line that will be visible in compiler output:
    use kuchiki::traits::*;  // This file requires the `hyper` feature to be enabled
}

#[cfg(feature = "hyper")]
fn main() {
    use kuchiki::traits::*;

    if let Ok(doc) = kuchiki::parse_html().from_http("https://www.mozilla.org/en-US/") {
        let comment = doc.descendants().comments().next().unwrap();
        println!("{}", *comment.borrow());
    } else {
        println!("{}", "The page couldn't be fetched");
    }
}
