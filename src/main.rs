extern crate dimensioned as dim;

use std::env;
use std::{f64, u8};
use std::fs::File;
use std::io::stdout;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;
use dim::{si, Dimensionless};

fn SAMPLE_RATE()             -> si::Hertz<f64>  { 44100.0 * si::HZ }
fn TUNING_NOTE()             -> si::Hertz<f64>  { 440.0 * si::HZ }
fn NOTES_IN_SCALE()          -> u64             { 12 }
fn BEAT_LENGTH()             -> si::Second<f64> { 0.5 * si::S }

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
}

fn to_displacement_sin(time: si::Second<f64>, freq: si::Hertz<f64>) -> f64 {
    (time * freq * 2.0 * f64::consts::PI).sin()
}

fn to_sample(displacement: f64) -> u8 {
    (((displacement * 127.0) as i8) as u8).wrapping_add(127)
}

fn main() {
    let mut time: si::Second<f64> = 0.0 * si::S;

    // White Lies - Time To Give
    let notes: Vec<Note> = vec![
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
        for note in notes.iter() {
            let note = note.clone().transpose((loop_num * 3) as Pitch);
            let num_samples = (note.time() * SAMPLE_RATE()).value().clone() as u64;
            eprintln!("{:?}, {}, {}", note, note.freq(), note.time());
            for i in 0..num_samples {
                time += TIME_PER_SAMPLE();
                let buf = [to_sample(to_displacement_sin(time, note.freq())); 1];
                stdout().write(&buf);
            }
        }
    }
}
