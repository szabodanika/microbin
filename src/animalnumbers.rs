const animal_names: &[&str] = &[
	"ant", "eel", "mole", "sloth",
	"ape", "emu", "monkey", "snail",
	"bat", "falcon", "mouse", "snake",
	"bear", "fish", "otter", "spider",
	"bee", "fly", "parrot", "squid",
	"bird", "fox", "panda", "swan",
	"bison", "frog", "pig", "tiger",
	"camel", "gecko", "pigeon", "toad",
	"cat", "goat", "pony", "turkey",
	"cobra", "goose", "pug", "turtle",
	"crow", "hawk", "rabbit", "viper",
	"deer", "horse", "rat", "wasp",
	"dog", "jaguar", "raven", "whale",
	"dove", "koala", "seal", "wolf",
	"duck", "lion", "shark", "worm",
	"eagle", "lizard", "sheep", "zebra",
];

pub fn to_animal_names(mut n: u64) -> String {
	let mut result: Vec<&str> = Vec::new();

	if n == 0 {
		return animal_names[0].parse().unwrap();
	} else if n == 1 {
		return animal_names[1].parse().unwrap();
	}

	// max 4 animals so 6 * 6 = 64 bits
	let mut power = 6;
	loop {
		let d = n / animal_names.len().pow(power) as u64;

		if !(result.is_empty() && d == 0) {
			result.push(animal_names[d as usize]);
		}

		n -= d * animal_names.len().pow(power) as u64;

		if power > 0 {
			power -= 1;
		} else { break; }
	}

	result.join("-")
}

pub fn to_u64(n: &str) -> u64 {
	let mut result: u64 = 0;

	let mut animals: Vec<&str> = n.split("-").collect();

	let mut pow = animals.len();
	for i in 0..animals.len() {
		pow -= 1;
		result += (animal_names.iter().position(|&r| r == animals[i]).unwrap() * animal_names.len().pow(pow as u32)) as u64;
	}

	result
}

