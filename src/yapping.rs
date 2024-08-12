use rand::seq::{IteratorRandom, SliceRandom};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const NGRAM_NUMBER: usize = 5;

fn process(text: &str) {
    if text.chars().count() < NGRAM_NUMBER - 1 {
        return;
    }

    let text = format!("B{}E", text.to_lowercase());
    let path = format!("yapping{NGRAM_NUMBER}.txt");
    let mut content = fs::read_to_string(&path).unwrap_or_default();

    if content.len() > 100_000 {
        return;
    }

    let chars: Vec<char> = text.chars().collect();
    for i in (NGRAM_NUMBER - 1)..chars.len() {
        if !content.is_empty() {
            content.push('S');
        }
        let substring: String = chars[((i + 1) - NGRAM_NUMBER)..=i].iter().collect();
        content.push_str(&substring);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(content.as_bytes())
        .expect("Unable to write data");
}

fn generate() -> String {
    let path = format!("yapping{NGRAM_NUMBER}.txt");
    let content = fs::read_to_string(&path).expect("Unable to read file");

    assert!(!content.is_empty(), "Content file is empty. Processing may have failed.");

    let mut triplets: Vec<&str> = content.split('S').collect();

    let mut rng = rand::thread_rng();
    let mut attempts = 0;
    let mut reply: String = loop {
        attempts += 1;
        let options = triplets
            .iter()
            .filter(|&&i| i.starts_with('B'))
            .choose(&mut rng);
        assert!(attempts <= 5, "Generation attempts exceeded the limit.");
        if let Some(result_one) = options {
            break (*result_one).to_string();
        }
    };

    triplets.retain(|x| **x != reply);

    while !reply.ends_with('E') {
        let current_chars: String = reply
            .chars()
            .rev()
            .take(NGRAM_NUMBER - 1)
            .collect::<Vec<_>>()
            .iter()
            .rev()
            .collect();

        let candidates: Vec<char> = triplets
            .iter()
            .filter_map(|i| {
                let prefix: String = i.chars().take(NGRAM_NUMBER - 1).collect();
                if prefix == current_chars {
                    i.chars().last()
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            break;
        }

        let chosen = *candidates.choose(&mut rng).expect("No valid candidates");

        reply.push(chosen);

        let current = format!("{current_chars}{chosen}");
        triplets.retain(|i| **i != current);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(triplets.join("S").as_bytes())
        .expect("Unable to write data");

    format!("{} :3", &reply[1..reply.len() - 1])
}

pub fn yapping() -> String {
    truncate_file("yapping5.txt");
    let contents = fs::read_to_string("txt_files/input.txt")
        .expect("Should have been able to read the file");
    process(&contents);
    generate()
}

fn truncate_file<P: AsRef<Path>>(path: P) -> bool {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .is_ok()
}
