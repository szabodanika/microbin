const ANIMAL_NAMES: &[&str] = &[
    "ant", "eel", "mole", "sloth", "ape", "emu", "monkey", "snail", "bat", "falcon", "mouse",
    "snake", "bear", "fish", "otter", "spider", "bee", "fly", "parrot", "squid", "bird", "fox",
    "panda", "swan", "bison", "frog", "pig", "tiger", "camel", "gecko", "pigeon", "toad", "cat",
    "goat", "pony", "turkey", "cobra", "goose", "pug", "turtle", "crow", "hawk", "rabbit", "viper",
    "deer", "horse", "rat", "wasp", "dog", "jaguar", "raven", "whale", "dove", "koala", "seal",
    "wolf", "duck", "lion", "shark", "worm", "eagle", "lizard", "sheep", "zebra",
];
const ANIMAL_COUNT: u64 = ANIMAL_NAMES.len() as u64;

pub fn to_animal_names(number: u64) -> String {
    let mut result: Vec<&str> = Vec::new();

    if number == 0 {
        return ANIMAL_NAMES[0].parse().unwrap();
    }

    let mut value = number;
    while value != 0 {
        let digit = (value % ANIMAL_COUNT) as usize;
        value /= ANIMAL_COUNT;
        result.push(ANIMAL_NAMES[digit]);
    }

    // We calculated the numbers in Little-Endian,
    // now convert to Big-Endian for backwards compatibility with old data.
    result.reverse();

    result.join("-")
}

#[test]
fn test_to_animal_names() {
    assert_eq!(to_animal_names(0), "ant");
    assert_eq!(to_animal_names(1), "eel");
    assert_eq!(to_animal_names(64), "eel-ant");
    assert_eq!(to_animal_names(12345), "sloth-ant-lion");
}

pub fn to_u64(animal_names: &str) -> Result<u64, &str> {
    let mut result: u64 = 0;

    for animal in animal_names.split('-') {
        let animal_index = ANIMAL_NAMES.iter().position(|&r| r == animal);
        match animal_index {
            None => return Err("Failed to convert animal name to u64!"),
            Some(idx) => {
                result = result * ANIMAL_COUNT + (idx as u64);
            }
        }
    }

    Ok(result)
}

#[test]
fn test_animal_name_to_u64() {
    assert_eq!(to_u64("ant"), Ok(0));
    assert_eq!(to_u64("eel"), Ok(1));
    assert_eq!(to_u64("eel-ant"), Ok(64));
    assert_eq!(to_u64("sloth-ant-lion"), Ok(12345));
}
