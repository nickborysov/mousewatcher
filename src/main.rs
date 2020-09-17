use colored::*;
use device_query::{DeviceQuery, DeviceState};
use rdev::{display_size, listen, simulate, Button, Event, EventType, Key};
use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, thread};

const PARAM_DELAY: &str = "delay";
const MASTER_KEY: &Key = &Key::AltGr;
const CLICK_PERCENT: f32 = 0.01;
const COUNT_OF_CLICKS: i32 = 3;
const AREA_SIZE: i32 = 400;
static mut NO_ACTION_MS: u128 = 10_000;
static mut LAST_MOVE_MS: u128 = 0;
static mut STATE_WORKING: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1].starts_with(PARAM_DELAY) && args[1].len() > 6 {
        let delay_param = &args[1];
        let delay_str = &delay_param[6..];
        match delay_str.parse::<u128>() {
            Ok(delay) => unsafe { NO_ACTION_MS = delay * 1000 },
            Err(e) => {
                println!("Invalid param {:?} {:?}", PARAM_DELAY, e);
                exit(1)
            }
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        unsafe {
            LAST_MOVE_MS = now.as_millis();
        }
    }

    print_welcome_message();

    start_watcher();

    let res = listen(callback);
    match res {
        Ok(_) => {}
        Err(e) => {
            println!("error {:?}", e);
        }
    }
}

fn print_welcome_message() {
    println!();
    println!("{}", "Mouse Watcher ready!!!".bold().underline().cyan());
    println!();
    println!(
        "{} {} {}",
        "It moves mouse randomly, if user doesn't do any action for".bright_cyan(),
        unsafe { NO_ACTION_MS / 1000 }
            .to_string()
            .as_str()
            .bold()
            .underline()
            .bright_magenta(),
        "seconds.".bright_cyan(),
    );

    println!(
        "{} {} {}",
        "Press key".bright_cyan(),
        format!("{:?}", MASTER_KEY).as_str().bold().bright_magenta(),
        "to start or stop watching. It's close to arrows.".bright_cyan(),
    );
    println!();
    println!(
        "{}",
        "It's possible to set different delay by providing param like:".green()
    );
    println!(
        "{} {}",
        "mousewatcher delay=20".cyan(),
        "where delay is 20 seconds.".green()
    );
    println!();
    println!(
        "{}",
        "WARNING! If watcher doesn't start, allow `Terminal` control your computer.".yellow()
    );
    println!(
        "{}",
        "You can do it at System Preferences => Security & Privacy => Accessibility.".yellow()
    );
    println!(
        "{} {} {}",
        "After applying the change, please".yellow(),
        "RESTART".red(),
        "the app.".yellow()
    );
    println!();
}

fn start_watcher() {
    thread::spawn(|| loop {
        if unsafe { STATE_WORKING } {
            if should_move() {
                make_random_move();
            }
            sleep(Duration::from_secs(1));
        }
    });
}

fn should_move() -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    unsafe {
        let diff = now.as_millis() - LAST_MOVE_MS;
        println!("seconds since last move: {:?}", diff / 1000);
        diff > NO_ACTION_MS && LAST_MOVE_MS != 0
    }
}

fn callback(event: Event) {
    let _key = *MASTER_KEY;
    if let EventType::KeyRelease(_key) = event.event_type {
        unsafe {
            if STATE_WORKING {
                STATE_WORKING = false;
                exit(0);
            } else {
                STATE_WORKING = true;
                return;
            }
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    unsafe {
        LAST_MOVE_MS = now.as_millis();
    }
}

fn make_random_move() {
    let (size_x, size_y) = display_size().unwrap();
    let center_x = (size_x / 2) as i32;
    let center_y = (size_y / 2) as i32;

    let dest_x = (rand::random::<i32>() % (AREA_SIZE * 2)).abs() + center_x - AREA_SIZE;
    let dest_y = (rand::random::<i32>() % (AREA_SIZE * 2)).abs() + center_y - AREA_SIZE;
    let device_state = DeviceState::new();
    let (orig_mouse_x, orig_mouse_y) = device_state.get_mouse().coords;
    let mut mouse_x = orig_mouse_x;
    let mut mouse_y = orig_mouse_y;

    println!("Mouse moving to {:?}, {:?}", dest_x, dest_y);
    let mut count_of_clicks = 0;
    while mouse_x != dest_x || mouse_y != dest_y {
        if (mouse_x - dest_x) != 0 {
            mouse_x = if (mouse_x - dest_x).is_positive() {
                mouse_x - 1
            } else {
                mouse_x + 1
            };
        }
        if (mouse_y - dest_y) != 0 {
            mouse_y = if (mouse_y - dest_y).is_positive() {
                mouse_y - 1
            } else {
                mouse_y + 1
            };
        }
        // println!("Small move to {:?}, {:?}", mouse_x, mouse_y);
        let event_type = &EventType::MouseMove {
            x: mouse_x as f64,
            y: mouse_y as f64,
        };
        match simulate(event_type) {
            Ok(()) => (),
            Err(e) => {
                println!("We could not send {:?}, err {:?}", event_type, e);
            }
        }

        if rand::random::<f32>() < CLICK_PERCENT && count_of_clicks < COUNT_OF_CLICKS {
            count_of_clicks = count_of_clicks + 1;
            println!("Click at {:?}, {:?}", mouse_x, mouse_y);
            let event_type = &EventType::ButtonPress(Button::Left);
            match simulate(event_type) {
                Ok(()) => (),
                Err(e) => {
                    println!("We could not send {:?}, err {:?}", event_type, e);
                }
            }
        }
        sleep(Duration::from_millis(3));
    }
}
