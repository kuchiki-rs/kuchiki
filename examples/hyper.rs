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

    let url = "https://www.mozilla.org/en-US/";
    println!("{} - {} ", "Calling site ", url);

    // Fetch and parse the html page
    if let Ok(doc) = kuchiki::parse_html().from_http(url) {
        println!("{}", "Finding Easter egg");

        // Manually navigate to hidden comment
        let x = doc.children().nth(1).unwrap()
                    .first_child().unwrap()
                    .children().nth(3).unwrap();

        // Convert comment into RefString and borrow it
        let comment = x.as_comment().unwrap().borrow();

        println!("{}", *comment);
    } else {
        println!("{}", "The page couldn't be fetched");
    }
}
