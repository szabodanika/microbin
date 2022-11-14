const ANIMAL_NAMES: &[&str] = &[
    "ant", "eel", "mole", "sloth", "ape", "emu", "monkey", "snail", "bat", "falcon", "mouse",
    "snake", "bear", "fish", "otter", "spider", "bee", "fly", "parrot", "squid", "bird", "fox",
    "panda", "swan", "bison", "frog", "pig", "tiger", "camel", "gecko", "pigeon", "toad", "cat",
    "goat", "pony", "turkey", "cobra", "goose", "pug", "turtle", "crow", "hawk", "rabbit", "viper",
    "deer", "horse", "rat", "wasp", "dog", "jaguar", "raven", "whale", "dove", "koala", "seal",
    "wolf", "duck", "lion", "shark", "worm", "eagle", "lizard", "sheep", "zebra",
];

pub fn to_animal_names(mut number: u64) -> String {
    let mut result: Vec<&str> = Vec::new();

    if number == 0 {
        return ANIMAL_NAMES[0].parse().unwrap();
    }

    let mut power = 6;

    loop {
        let digit = number / ANIMAL_NAMES.len().pow(power) as u64;
        if !(result.is_empty() && digit == 0) {
            result.push(ANIMAL_NAMES[digit as usize]);
        }
        number -= digit * ANIMAL_NAMES.len().pow(power) as u64;
        if power > 0 {
            power -= 1;
        } else if power == 0 || number == 0 {
            break;
        }
    }

    result.join("-")
}

pub fn to_u64(animal_names: &str) -> Result<u64, &str> {
    let mut result: u64 = 0;

    let animals: Vec<&str> = animal_names.split('-').collect();

    let mut pow = animals.len();
    for animal in animals {
        pow -= 1;
        let animal_index = ANIMAL_NAMES.iter().position(|&r| r == animal);
        match animal_index {
            None => return Err("Failed to convert animal name to u64!"),
            Some(_) => {
                result += (animal_index.unwrap() * ANIMAL_NAMES.len().pow(pow as u32)) as u64
            }
        }
    }

    Ok(result)
}
