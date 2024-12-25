extern crate libc;

use std::time::Duration;
use num::Float;
use libc::{c_double, sleep};

extern "C" {
    fn sin(x: c_double) -> c_double;
    fn cos(x: c_double) -> c_double;
}

fn main() {
    let angle: c_double = 3.14159 / 4.0; // 45 degrees in radians
    unsafe {
        for i in 0..3000 {
            let x: f64 = 2.0;
            println!("sqrt using num lib: {}", Float::sin(x));
            unsafe{
                sleep(1);
            }
            // Calculate square root using num crate, which may rely on libm.
            let sqrt_x = x.sqrt();
            println!("Square root of {} is {}", x, sqrt_x);

            // Calculate sine using num crate
            let sin_x = x.sin();
            println!("Sine of {} is {}", x, sin_x);
            // let sine = sin(angle);
            // let cosine = cos(angle);
            //
            // println!("sin(45°): {}", sine);
            // println!("cos(45°): {}", cosine);
        }

    }
}