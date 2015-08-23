extern crate hyper;
extern crate kuchiki;

use hyper::Client;

use kuchiki::Html;

fn main() {
    // Create a client.
    let client = Client::new();
    let url = "https://www.mozilla.org/en-US/";
    println!("{} - {} ", "Calling site ", url);

    // Get response
    let mut response = client.get(url).send().unwrap();

    // Parse the html page
    if let Ok(html) = Html::from_stream(&mut response) {
        println!("{}", "Finding Easter egg");
        let doc = html.parse();

        // Manually navigate to hidden comment
        let x = doc.children().nth(1).unwrap()
                    .first_child().unwrap()
                    .children().nth(3).unwrap();

        // Convert comment into RefString and borrow it
        let comment = x.as_comment().unwrap().borrow();

        println!("{}", *comment);
    } else {
        println!("{}", "The page couldn't be parsed");
    }
}
