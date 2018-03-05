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
use std;
use rand;
use rand::distributions::{IndependentSample, Range};

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    in_bounds: bool
}

pub struct Tracer {
    vec_field: Vec<f64>,
    flux: Vec<usize>,
    vec_size: usize,
    img_width: usize,
    img_height: usize,
    particles: Vec<Particle>
        
}

impl Tracer {
    pub fn new(vec_field: &[f64],
               vec_size: usize,
               img_width: usize, img_height: usize,
               phase: f64) -> Tracer {
        
        let mut to_r =  Tracer {
            vec_field: Vec::new(),
            vec_size: vec_size,
            flux: Vec::new(),
            particles: Vec::new(),
            img_width: img_width,
            img_height: img_height
        };

        for _ in 0..img_width {
            for _ in 0..img_height {
                to_r.flux.push(0);
            }
        }

        for el in vec_field {
            to_r.vec_field.push(el * 2.0 * std::f64::consts::PI + phase);
        }

        return to_r;
    }

    pub fn add_particle(&mut self, x: f64, y: f64) {
        assert!(x <= 1.0 && x >= 0.0);
        assert!(y <= 1.0 && y >= 0.0);
        
        self.particles.push(Particle {
            x: x, y: y,
            vx: 0.0, vy: 0.0,
            in_bounds: true
        });
    }

    pub fn add_random_particle(&mut self) {
        let btwn = Range::new(0.0, 1.0);
        let mut rng = rand::thread_rng();
        let x = btwn.ind_sample(&mut rng);
        let y = btwn.ind_sample(&mut rng);

        self.add_particle(x, y);
        
    }

    fn _get_accel(&self, x: f64, y: f64) -> f64 {
        let vec_x = (x * self.vec_size as f64) as usize;
        let vec_y = (y * self.vec_size as f64) as usize;
        return self.vec_field[vec_y * self.vec_size + vec_x];
    }

    fn _inc_flux(&mut self, x: f64, y: f64) {
        let flx_x = (x * self.img_width as f64) as usize;
        let flx_y = (y * self.img_height as f64) as usize;

        let idx = flx_y * self.img_width + flx_x;
        self.flux[idx] += 1;
    }

    pub fn progress_for(&mut self, steps: usize, dt: f64) {
        for _ in 0..steps {
            self.progress(dt);
        }
    }

    pub fn progress(&mut self, dt: f64) {
        // move the particles locally
        let mut parts = std::mem::replace(&mut self.particles,
                                          vec![]);
        
        for mut p in &mut parts {
            // first, figure out what velocity we are
            // currently in (which cell)
            let angle = self._get_accel(p.x, p.y);
            let fx = angle.cos();
            let fy = angle.sin();

            
            p.vx += fx * dt;
            p.vy += fy * dt;

            assert!(p.x >= 0.0);
            assert!(p.y >= 0.0);
            assert!(p.x <= 1.0);
            assert!(p.y <= 1.0);
            
            p.x += p.vx * dt;
            p.y += p.vy * dt;

            if p.x < 0.0 || p.x > 1.0
                || p.y < 0.0 || p.y > 1.0 {
                    p.in_bounds = false;
                } else {
                    self._inc_flux(p.x, p.y);

                    let fric = dt * 0.5;

                    p.vx *= 1.0 - fric;
                    p.vy *= 1.0 - fric;
                }

        }

        parts.retain(|p| p.in_bounds);

        std::mem::replace(&mut self.particles, parts);
    }

    pub fn get_normalized_flux(&self) -> Vec<f64> {
        let mut to_r = Vec::new();

        // first, find the largest element in flux
        let max_val = (*self.flux.iter().max().unwrap()) as f64;

        for val in &self.flux {
            to_r.push(*val as f64 / max_val);
        }

        return to_r;
    }

    pub fn get_unnormalized_flux(&self) -> &Vec<usize> {
        return &self.flux;
    }

}
