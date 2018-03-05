// < begin copyright > 
// Copyright Ryan Marcus 2018
// 
// This file is part of perlin.
// 
// perlin is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// perlin is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with perlin.  If not, see <http://www.gnu.org/licenses/>.
// 
// < end copyright > 
#![allow(unknown_lints)] // for clippy
#![allow(explicit_iter_loop, needless_return)]

extern crate rand;
extern crate image;
extern crate rayon;
extern crate clap;
extern crate mktemp;


mod perlin;
mod tracer;

use std::fs::File;
use rayon::prelude::*;
use std::cmp;
use std::path::Path;
use rand::distributions::{IndependentSample, Range};


use perlin::perlin;
use tracer::Tracer;
use clap::{App, Arg};
use mktemp::Temp;
use std::process::Command;
use std::f64;

fn _lerp(a: f64, b: f64, w: f64) -> f64 {
    assert!(w >= 0.0);
    assert!(w <= 1.0);
    return w*a + (1.0 - w)*b;
}

fn write_image(data: &[f64],
               img_width: usize, img_height: usize,
               name: &Path,
               color_func: u8) {

    let mut imgbuf = image::ImageBuffer::new(img_width as u32,
                                             img_height as u32);
    
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let val = data[(y*img_width as u32 + x) as usize];

        match color_func {
            0 => {
                *pixel = image::Rgb([(val.sqrt()*180.0) as u8,
                                     20,
                                     (val.sqrt()*225.0) as u8]);
            },

            1 => {
                *pixel = image::Rgb([(val.sqrt()*20.0) as u8,
                                     (val.sqrt()*180.0) as u8,
                                     (val.sqrt()*225.0) as u8]);
            },

            2 => {                
                *pixel = image::Rgb([(val.sqrt()*200.0) as u8,
                                     (val.sqrt()*180.0) as u8,
                                     (val.sqrt()*90.0) as u8]);
            },

            3 => {
                let c1_r = 125.0;
                let c1_g = 186.0;
                let c1_b = 182.0;
                
                let c2_r = 46.0;
                let c2_g = 59.0;
                let c2_b = 65.0;
                
                *pixel = image::Rgb([_lerp(c1_r, c2_r, val.sqrt()) as u8,
                                     _lerp(c1_g, c2_g, val.sqrt()) as u8,
                                     _lerp(c1_b, c2_b, val.sqrt()) as u8]);
            },

            _ => {
                panic!("Unknown color function!");
            }
        }
        
    }    

    // Save the image as “fractal.png”
    let fout = &mut File::create(name).unwrap();

    // We must indicate the image's color type
    // and what format to save as
    image::ImageRgb8(imgbuf).save(fout, image::PNG).unwrap();

}

fn write_image_1ch(data: &[f64],
                   img_width: usize, img_height: usize,
                   name: &Path) {

    let mut imgbuf = image::ImageBuffer::new(img_width as u32,
                                             img_height as u32);
    
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let val = data[(y*img_width as u32 + x) as usize];

        *pixel = image::Rgb([(val * 255.0) as u8,
                             (val * 255.0) as u8,
                             (val * 255.0) as u8]);
    }
        
    // Save the image as “fractal.png”
    let fout = &mut File::create(name).unwrap();

    // We must indicate the image's color type
    // and what format to save as
    image::ImageRgb8(imgbuf).save(fout, image::PNG).unwrap();

}

fn make_tracers(d1: &[f64],
                perlin_size: usize,
                img_width: usize,
                img_height: usize,
                phase: f64,
                num_jobs: usize) -> Vec<Tracer> {
    
    let mut tracers = Vec::new();
    for _ in 0..num_jobs {
        tracers.push(Tracer::new(d1,
                                 perlin_size,
                                 img_width, img_height,
                                 phase));
    }
    
    let mut c = 0;
    for x in 0..img_width {
        for y in 0..img_height {
            let part_x = x as f64 / img_width as f64;
            let part_y = y as f64 / img_height as f64;

            tracers[c].add_particle(part_x, part_y);
            c = (c + 1) % tracers.len();
        }
    }

    return tracers;
}

fn make_octaves(img_width: usize, img_height: usize) -> Vec<f64> {
    let data = vec![perlin(2, img_width),
                    perlin(4, img_width),
                    perlin(8, img_width)];

    let mut to_r = Vec::new();

    for idx in 0..(img_width*img_height) {
        let d1 = (data[0][idx] * 2.0) - 1.0;
        let d2 = (data[1][idx] * 2.0) - 1.0;
        let d3 = (data[2][idx] * 2.0) - 1.0;
        
        to_r.push(d1 * 0.10
                  + d2 * 0.20
                  + d3 * 0.40);
    }

    // normalize
    let max_val = to_r.iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let min_val = to_r.iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));

    let range = max_val - min_val;

    return to_r.iter().map(|v| (v - min_val) / range)
        .collect();

}

fn write_tracers(out_name: &Path,
                 img_width: usize,
                 img_height: usize,
                 tracers: &[Tracer],
                 max_flux: usize,
                 color_func: u8) {
    
    let mut init_val: Vec<usize> = Vec::new();
    for _ in 0..img_width {
        for _ in 0..img_height {
            init_val.push(0);
        }
    }
    
    let data = tracers.iter()
        .map(|t| t.get_unnormalized_flux());

    for vec in data {
        for c in 0..vec.len() {
            init_val[c] += vec[c];
        }
    }
    
    let res:Vec<f64> = init_val.iter()
        .map(|&k| k as f64 / max_flux as f64)
        .collect();
    
    write_image(&res,
                img_width, img_height,
                out_name, color_func);
}




fn main() {

    let matches = App::new("Perlin flow fields")
        .version("0.0.1")
        .author("Ryan Marcus <ryan@ryanmarc.us>")
        .about("A program to make (optionally animated) Perlin flow fields")
        .arg(Arg::with_name("output")
             .short("o")
             .help("Name of output file")
             .takes_value(true)
             .default_value("out"))
        .arg(Arg::with_name("width")
             .short("w")
             .help("Width of the output image")
             .takes_value(true)
             .default_value("960"))
        .arg(Arg::with_name("height")
             .short("h")
             .help("height of the output image")
             .takes_value(true)
             .default_value("540"))
        .arg(Arg::with_name("frequency")
             .long("noise-frequency")
             .short("f")
             .help("Frequency of Perlin noise")
             .takes_value(true)
             .default_value("3"))
        .arg(Arg::with_name("animate")
             .short("a")
             .help("Create an animation (instead of a single frame)")
             .takes_value(false))
        .arg(Arg::with_name("color function")
             .short("c")
             .help("The coloring function to use (0 through 3)")
             .takes_value(true)
             .default_value("0"))
        .arg(Arg::with_name("number of jobs")
             .short("j")
             .long("jobs")
             .help("Number of parallel jobs")
             .default_value("1"))
        .arg(Arg::with_name("perlin")
             .short("p")
             .long("perlin")
             .help("Just produce Perlin noise with width size")
             .conflicts_with("color function")
             .conflicts_with("animate")
             .conflicts_with("number of jobs")
             .conflicts_with("height"))
        .arg(Arg::with_name("octaves")
             .long("octaves")
             .help("Produce Perlin noise with multiple pre-defined octaves")
             .conflicts_with("color function")
             .conflicts_with("animate")
             .conflicts_with("number of jobs")
             .conflicts_with("height")
             .conflicts_with("perlin"))
        .get_matches();

    let out_name = matches.value_of("output").unwrap();
    let color_func = matches.value_of("color function").unwrap()
        .parse::<u8>()
        .expect("Color function must be an integer value!");

    let img_width = matches.value_of("width").unwrap().parse::<usize>()
        .expect("Image width must be an integer value");

    let img_height = matches.value_of("height").unwrap().parse::<usize>()
        .expect("Image height must be an integer value");

    let freq = matches.value_of("frequency").unwrap().parse::<u16>()
        .expect("Frequency must be an integer value");

    let do_animation = matches.is_present("animate");
    let just_perlin = matches.is_present("perlin");
    let octaves = matches.is_present("octaves");

    let num_jobs = matches.value_of("number of jobs")
        .unwrap().parse::<usize>()
        .expect("Number of jobs must be an integer value");
    
    let perlin_size: usize = if just_perlin {
        img_width
    } else {
        cmp::min(img_width, img_height)/2
    };

    if octaves {
        let octave_img:Vec<f64> = make_octaves(img_width, img_width);
        write_image_1ch(&octave_img, img_width, img_width,
                        Path::new(&format!("{}.png", out_name)));
        return;
    }
    
    let d1 = perlin(freq, perlin_size);

    if just_perlin {
        write_image_1ch(&d1, img_width, img_width,
                        Path::new(&format!("{}.png", out_name)));
        return;
    }
    
    let btwn = Range::new(0.0,
                          2.0 * std::f64::consts::PI);
    let mut rng = rand::thread_rng();
    let phase = btwn.ind_sample(&mut rng);


    let mut tracers = make_tracers(&d1, perlin_size,
                                   img_width, img_height,
                                   phase, num_jobs);


    // first, progress all the way to the end to get the max flux value
    tracers.par_iter_mut()
        .for_each(|t| t.progress_for(15*60, 0.0025));

    
    // next, compute the max flux...
    let max_flux = {
        
        let mut max = 0;
        let data: Vec<&Vec<usize>> = tracers.iter()
            .map(|t| t.get_unnormalized_flux())
            .collect();

        for idx in 0..data[0].len() {
            let mut sum = 0;
            for d in &data {
                sum += d[idx];
            }

            max = cmp::max(max, sum);
        }

        max
    };

    if !do_animation {
        // write the normalized flux out, and we're done.
        write_tracers(Path::new(&format!("{}.png", out_name)),
                      img_width, img_height,
                      &tracers,
                      max_flux, color_func);
        return;
    }



    // now, reset / remake all the tracers and generate the frames
    
    tracers = make_tracers(&d1, perlin_size,
                           img_width, img_height,
                           phase, num_jobs);

    
    let temp_dir = Temp::new_dir().unwrap();


    for frame in 0..15*60 {
        if frame % 10 == 0 {
            println!("Computing frame {} / {}...", frame, 15*60);
        }
        
        tracers.par_iter_mut()
            .for_each(|t| t.progress(0.0025));

        let pb = temp_dir.as_ref()
            .join(Path::new(&format!("out_{:04}.png",
                                     frame)
            ));
        write_tracers(&pb,
                      img_width, img_height,
                      &tracers,
                      max_flux, color_func);
    }

    // now, run the FFMPEG command
    // /usr/bin/ffmpeg -framerate 60 -i out_%04d.png -pix_fmt yuv420p out.mp4
    let mut child = Command::new("/usr/bin/ffmpeg")
        .args(&["-framerate", "60", "-i",
                &format!("{}/out_%04d.png", temp_dir.as_ref().to_str().unwrap()),
                "-pix_fmt", "yuv420p",
                &format!("{}.mp4", out_name)])
        .spawn()
        .expect("failed to execute process");
    
    child.wait().unwrap();

}
