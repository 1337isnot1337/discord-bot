use rand::seq::{IteratorRandom, SliceRandom};
use std::fs::{self, OpenOptions};
use std::io::Write;

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

    let mut triplets: Vec<&str> = content.split('S').collect();

    let mut rng = rand::thread_rng();
    let mut reply = (*triplets
        .iter()
        .filter(|&&i| i.starts_with('B'))
        .choose(&mut rng)
        .expect("No valid starting triplet"))
    .to_string();

    triplets.retain(|x| **x != reply);

    loop {
        if reply.ends_with('E') {
            reply.pop();
            break;
        }

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
                if i.chars().take(NGRAM_NUMBER - 1).collect::<String>() == current_chars {
                    Some(i.chars().last().unwrap())
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            break;
        }
        let chosen = *candidates.choose(&mut rng).expect("No valid candidates");

        let current = current_chars + &chosen.to_string();
        triplets.retain(|i| **i != current);

        if chosen == 'E' {
            break;
        }
        reply.push(chosen);
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(triplets.join("S").as_bytes())
        .expect("Unable to write data");

    format!("{} :3", &reply[1..])
}


pub fn yapping() -> String {
    let contents = fs::read_to_string("txt_files/input.txt").expect("Should have been able to read the file");
    process(&contents);
    generate()
}
