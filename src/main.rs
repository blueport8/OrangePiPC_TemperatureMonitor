use std::fs::File;
use std::collections::LinkedList;
use std::io::prelude::*;
use std::{thread, time};


struct TemperatureContext {
    current: f32,
    last15sec: LinkedList<f32>,
    last60sec: LinkedList<f32>,
    last300sec: LinkedList<f32>
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

	let mut temperature_context = TemperatureContext {
		current: 0.0,
		last15sec: LinkedList::new(),
		last60sec: LinkedList::new(),
		last300sec: LinkedList::new()
	};

	// Main loop
	loop {
		let temp: f32 = match read_temperature() {
			Ok(current_temperature) => {
				// println!("{:?}", current_temperature);
				current_temperature
			},
			Err(e) => {
				println!("Failed to read temperature, reason: {}", e);
				continue;
			}
		};
		update_context(interval_sec, temp, &mut temperature_context);
		let snapshot: TemperatureSnaphost = get_temperature_snapshot(& temperature_context);
		println!("{:?}", snapshot);

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

	if context.last15sec.len() > (15 / interval_sec) as usize {
		context.last15sec.pop_front();
	}

	if context.last60sec.len() > (60 / interval_sec) as usize {
		context.last60sec.pop_front();
	}

	if context.last300sec.len() > (300 / interval_sec) as usize {
		context.last300sec.pop_front();
	}
}

fn get_temperature_snapshot(temperature_context: & TemperatureContext) -> TemperatureSnaphost {
	let mut sum_last15sec: f32 = 0.0;
	for result in temperature_context.last15sec.iter() {
		sum_last15sec += result;
	}

	let mut sum_last60sec: f32 = 0.0;
	for result in temperature_context.last60sec.iter() {
		sum_last60sec += result;
	}

	let mut sum_last300sec: f32 = 0.0;
	for result in temperature_context.last300sec.iter() {
		sum_last300sec += result;
	}

	TemperatureSnaphost {
		current: temperature_context.current,
		last15sec_avg: sum_last15sec as f32 / temperature_context.last15sec.len() as f32,
		last60sec_avg: sum_last60sec as f32 / temperature_context.last60sec.len() as f32,
		last300sec_avg: sum_last300sec as f32 / temperature_context.last300sec.len() as f32
	}
}
