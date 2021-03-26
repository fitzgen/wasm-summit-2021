use pulldown_cmark::{html, Parser};
use std::io::Read;

fn main() {
    let mut stdin = std::io::stdin();
    let mut markdown_input = String::new();
    stdin.read_to_string(&mut markdown_input).unwrap();

    let parser = Parser::new(&markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    println!("{}", html_output);
}
