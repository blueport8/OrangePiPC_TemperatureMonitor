use std::fs::File;
use std::fs::OpenOptions;
use std::collections::LinkedList;
use std::io::prelude::*;
use std::{thread, time};


struct TemperatureContext {
    current: f32,
    last15sec: LinkedList<f32>,
    last60sec: LinkedList<f32>,
    last300sec: LinkedList<f32>,
    last15sec_sum: f32,
    last60sec_sum: f32,
    last300sec_sum: f32
}

#[derive(Debug)]
struct TemperatureSnaphost {
	current: f32,
	last15sec_avg: f32,
	last60sec_avg: f32,
	last300sec_avg: f32
}

fn main() {
	let interval_sec: u64 = 1;
	let pooling_interval = time::Duration::from_millis(1000 * interval_sec);
	let temps_file_path = String::from("/var/www/html/temps");

	let mut temperature_context = TemperatureContext {
		current: 0.0,
		last15sec: LinkedList::new(),
		last60sec: LinkedList::new(),
		last300sec: LinkedList::new(),
		last15sec_sum: 0.0,
		last60sec_sum: 0.0,
		last300sec_sum: 0.0
	};

	// Main loop
	loop {
		let current_temp: f32 = match read_temperature() {
			Ok(t) => t,
			Err(e) => {
				println!("Failed to read temperature, reason: {}", e);
				continue;
			}
		};
		update_context(interval_sec, current_temp, &mut temperature_context);
		let snapshot: TemperatureSnaphost = get_temperature_snapshot(& temperature_context);
		println!("{:?}", snapshot);
		write_snapshot_to_file(& snapshot, & temps_file_path);
		thread::sleep(pooling_interval);
	}
}

fn read_temperature() -> std::io::Result<f32> {
	let mut file = File::open("/sys/class/thermal/thermal_zone0/temp")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    contents = contents.trim_right().to_string();
    let temperature: f32 = parse_temperature_string(contents);

    Ok(temperature)
}

fn parse_temperature_string(current_temperature: String) -> f32 {
	match current_temperature.parse::<f32>() {
		Ok(parsed_temperature) => {
			parsed_temperature / 1000 as f32
		},
		Err(_e) => {
			println!("Failed to convert temperature string");
			0.0
		}
	}
}

fn update_context(
	interval_sec: u64,
	current_temperature: f32,
	context: &mut TemperatureContext) {

	context.current = current_temperature;
	context.last15sec.push_back(current_temperature);
	context.last60sec.push_back(current_temperature);
	context.last300sec.push_back(current_temperature);

	context.last15sec_sum += current_temperature;
	context.last60sec_sum += current_temperature;
	context.last300sec_sum += current_temperature;

	if context.last15sec.len() > (15 / interval_sec) as usize {
		let oldest_val = context.last15sec.pop_front().unwrap();
		context.last15sec_sum -= oldest_val;
	}

	if context.last60sec.len() > (60 / interval_sec) as usize {
		let oldest_val = context.last60sec.pop_front().unwrap();
		context.last60sec_sum -= oldest_val;
	}

	if context.last300sec.len() > (300 / interval_sec) as usize {
		let oldest_val = context.last300sec.pop_front().unwrap();
		context.last300sec_sum -= oldest_val;
	}
}

fn get_temperature_snapshot(context: & TemperatureContext) -> TemperatureSnaphost {
	TemperatureSnaphost {
		current: context.current,
		last15sec_avg: context.last15sec_sum as f32 / context.last15sec.len() as f32,
		last60sec_avg: context.last60sec_sum as f32 / context.last60sec.len() as f32,
		last300sec_avg: context.last300sec_sum as f32 / context.last300sec.len() as f32
	}
}

fn write_snapshot_to_file(snapshot: & TemperatureSnaphost, path_to_write: & String) {
	let oppening_result = OpenOptions::new().write(true).truncate(true).open(path_to_write);
	match oppening_result {
		Ok(mut file) => {
			write!(&mut file, "{} {} {}",
				snapshot.last15sec_avg,
				snapshot.last60sec_avg,
				snapshot.last300sec_avg);
		},
		Err(e) => {
			println!("Failed to write current temperature to file: {:?}", e);
		},
	}
}
