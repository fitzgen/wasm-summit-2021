use dym::Lexicon;
use once_cell::sync::OnceCell;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

static LEXICON: OnceCell<Lexicon> = OnceCell::new();

#[cfg_attr(feature = "wizer", export_name = "wizer.initialize")]
fn init() {
    let mut lexicon = Lexicon::new();
    let mut dictionary = open_dictionary_file("./words.txt");

    let mut buf = String::new();
    while let Some(word) = read_word(&mut dictionary, &mut buf) {
        lexicon.insert(word);
    }

    LEXICON
        .set(lexicon)
        .expect("should not be already initialized, we only init once");
}

fn open_dictionary_file(path: impl AsRef<Path>) -> io::BufReader<fs::File> {
    let dictionary = fs::File::open(path).unwrap_or_else(|err| {
        panic!("failed to open dictionary file: {}", err);
    });

    io::BufReader::with_capacity(512, dictionary)
}

fn read_word<'a>(
    dictionary: &mut io::BufReader<fs::File>,
    word: &'a mut String,
) -> Option<&'a str> {
    word.clear();
    match dictionary.read_line(word) {
        Ok(0) => None,
        Ok(_) => Some(word),
        Err(e) => panic!("error reading from dictionary file: {}", e),
    }
}

fn main() {
    if cfg!(not(feature = "wizer")) {
        init();
    }

    let lexicon = LEXICON.get().expect("lexicon should be initialized");

    for word in env::args().skip(1) {
        if lexicon.contains(&word) {
            continue;
        }

        let corrections = lexicon.corrections_for(&word);
        if corrections.is_empty() {
            println!("{} is misspelled", word);
        } else {
            println!("{} is misspelled, did you mean {}?", word, corrections[0]);
        }
    }
}
