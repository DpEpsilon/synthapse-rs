extern crate dimensioned as dim;

use std::env;
use std::{f64, u8};
use std::fs::File;
use std::io::stdout;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;
use dim::{si, Dimensionless, MapUnsafe};

fn SAMPLE_RATE()             -> si::Hertz<f64>  { 44100.0 * si::HZ }
fn TUNING_NOTE()             -> si::Hertz<f64>  { 440.0 * si::HZ }
fn NOTES_IN_SCALE()          -> u64             { 12 }
fn BEAT_LENGTH()             -> si::Second<f64> { 0.25 * si::S }

fn TIME_PER_SAMPLE()         -> si::Second<f64> { 1.0 / SAMPLE_RATE() }
fn SMALLEST_INTERVAL_RATIO() -> f64             { (2.0 as f64).powf(1.0 / (NOTES_IN_SCALE() as f64)) }

type Pitch = i16;
type Duration = u16;

#[derive(Debug, Copy, Clone)]
struct Note(Pitch, Duration);
impl Note {
    fn freq(self) -> si::Hertz<f64> {
        TUNING_NOTE() * SMALLEST_INTERVAL_RATIO().powi(self.0 as i32)
    }

    fn time(self) -> si::Second<f64> {
        BEAT_LENGTH() * (self.1 as f64)
    }

    fn transpose(self, change: Pitch) -> Note {
        Note(self.0 + change, self.1)
    }

    fn scale_time(self, factor: Duration) -> Note {
        Note(self.0, self.1 * factor)
    }

    fn envelope_at_time(self, time: si::Second<f64>) -> f64 {
        if time < 0.025 * si::S {
            (time / (0.025 * si::S)).value().clone()
        } else if time < 0.05 * si::S {
            ((time - 0.025 * si::S) / (0.025 * si::S) * 0.2 + 0.8).value().clone()
        } else if time < self.time() - 0.0125 * si::S {
            0.8
        } else {
            ((self.time() - time) / (0.0125 * si::S) * 0.8).value().clone()
        }
    }

    fn displacement_at_time(self, time: si::Second<f64>) -> f64 {
        to_displacement_sin_shepard(time, self.freq()) * self.envelope_at_time(time)
    }
}

fn to_displacement_sin(time: si::Second<f64>, freq: si::Hertz<f64>) -> f64 {
    (time * freq * 2.0 * f64::consts::PI).sin()
}

fn to_displacement_saw(time: si::Second<f64>, freq: si::Hertz<f64>) -> f64 {
    (to_displacement_sin(time, freq) +
        to_displacement_sin(time, freq.map_unsafe(|x| x * 2.0)) / 2.0 +
        to_displacement_sin(time, freq.map_unsafe(|x| x * 3.0)) / 3.0 +
        to_displacement_sin(time, freq.map_unsafe(|x| x * 4.0)) / 4.0 +
        to_displacement_sin(time, freq.map_unsafe(|x| x * 5.0)) / 5.0) / 2.284
}

fn to_displacement_sin_shepard_adjusted(time: si::Second<f64>, freq: si::Hertz<f64>) -> f64 {
    if freq > TUNING_NOTE().map_unsafe(|x| x * 4.0) || freq < TUNING_NOTE().map_unsafe(|x| x / 4.0) {
        0.0
    } else {
        to_displacement_saw(time, freq) * (2.0 - ((freq / TUNING_NOTE()).value().clone().log2()).abs()) * 0.1
    }
}

fn to_displacement_sin_shepard(time: si::Second<f64>, freq: si::Hertz<f64>) -> f64 {
    let freq = (2.0 as f64).powf(((freq / TUNING_NOTE()).value().clone().log2() + 1.0) % 2.0 - 1.0) * TUNING_NOTE();
    to_displacement_sin_shepard_adjusted(time, freq) +
        to_displacement_sin_shepard_adjusted(time, freq.map_unsafe(|x| x / 2.0)) +
        to_displacement_sin_shepard_adjusted(time, freq.map_unsafe(|x| x * 2.0)) +
        to_displacement_sin_shepard_adjusted(time, freq.map_unsafe(|x| x / 4.0)) +
        to_displacement_sin_shepard_adjusted(time, freq.map_unsafe(|x| x * 4.0))
}

fn to_sample(displacement: f64) -> u8 {
    (((displacement * 127.0) as i8) as u8).wrapping_add(127)
}

fn main() {
    let mut time: si::Second<f64> = 0.0 * si::S;

    // White Lies - Time To Give
    let melody: Vec<Note> = vec![
        Note(7, 1),
        Note(3, 1),
        Note(0, 1),
        Note(7, 2),
        Note(2, 1),
        Note(11 - 12, 1),
        Note(7, 2),
        Note(2, 1),
        Note(10 - 12, 1),
        Note(7, 1),
        Note(5, 1),
        Note(0, 1),
        Note(9 - 12, 1),
        Note(5, 2),
        Note(0, 1),
        Note(8 - 12, 1),
        Note(5, 1),
        Note(3, 1),
        Note(0, 1),
        Note(7 - 12, 1),
        Note(3, 2),
        Note(0, 1),
        Note(8 - 12, 1),
        Note(3, 1),
        Note(5, 1),
        Note(2, 1),
        Note(8, 1),
    ];

    for loop_num in 0..=12 {
        for note in melody.iter() {
            let note = note.clone().transpose((loop_num * 3 - 2) as Pitch).scale_time(2);
            let num_samples = (note.time() * SAMPLE_RATE()).value().clone() as u64;
            eprintln!("{:?}, {}, {}", note, note.freq(), note.time());
            for i in 0..num_samples {
                let buf = [to_sample(note.displacement_at_time((i as f64) * TIME_PER_SAMPLE())); 1];
                stdout().write(&buf);
            }
            time += note.time();
        }
    }
}
