use navigator_rs::{BoardVersion, Navigator, PiVersion};
use std::thread::sleep;
use std::time::Duration;

use rand::seq::SliceRandom;

fn main() {
    println!("Creating your navigator module!");

    let mut nav = Navigator::create()
    .with_board_version(BoardVersion::V1)
    .with_pi_version(PiVersion::Pi4)
    .with_rgb_led_strip_size(8)
    .build();

    println!("Setting up your navigator, ahoy!");
    nav.set_pwm_enable(true);
    nav.set_pwm_frequency(60.0);

    let colors = [
        [255u8, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [0, 255, 255],
        [255, 0, 255],
    ];

    loop {
        for value in [0.25, 0.5, 0.75, 0.5] {
            nav.set_neopixel(&[*colors.choose(&mut rand::thread_rng()).unwrap()]);
            nav.set_duty_cycle_all(value);
            sleep(Duration::from_millis(1000));
        }
    }
}
