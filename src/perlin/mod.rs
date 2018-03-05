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
use rand;

struct PerlinNoise {
    dim: u16,
    grid: Vec<u8>
}

impl PerlinNoise {
    fn new(dim: u16) -> PerlinNoise {
        let mut to_r = PerlinNoise {
            dim: dim,
            grid: Vec::new()
        };

        for _ in 0..dim+1 {
            for _ in 0..dim+1 {
                to_r.grid.push(rand::random::<(u8)>()  % 8);
            }
        }

        return to_r;
    }

    /*
    Vectors:

    0:  -1, -1
    1:  -1,  0
    2:  -1,  1
    3:   0, -1
    4:   0,  1 
    5:   1, -1
    6:   1,  0
    7:   1,  1
     */
    fn _to_vec(v: u8) -> (f64, f64) {
        return match v {
            0 => (-1.0, -1.0),
            1 => (-1.0,  0.0),
            2 => (-1.0,  1.0),
            3 => (0.0,  -1.0),
            4 => (0.0,   1.0),
            5 => (1.0,  -1.0),
            6 => (1.0,   0.0),
            7 => (1.0,   1.0),
            _ => panic!("u8 out of range!")
        };
    }

    fn _dot(x1: (f64, f64), x2: (f64, f64)) -> f64 {
        let (a, b) = x1;
        let (c, d) = x2;
        let res = a*c + b*d;

        return res;
    }

    fn _fade(v: f64) -> f64 {
        assert!(v <= 1.0);
        assert!(v >= 0.0);
        return 6.0*v.powf(5.0) - 15.0*v.powf(4.0) + 10.0*v.powf(3.0);
    }

    fn _lerp(a: f64, b: f64, w: f64) -> f64 {
        assert!(w >= 0.0);
        assert!(w <= 1.0);

        return w*a + (1.0 - w)*b;
    }


    fn sample(&self, ox: f64, oy: f64) -> f64 {

        assert!(ox <= 1.0);
        assert!(oy <= 1.0);
        
        let x: f64 = ox * f64::from(self.dim);
        let y: f64 = oy * f64::from(self.dim);
        let sx: usize = x as usize;
        let sy: usize = y as usize;


        // c1 c4
        // c2 c3

        let c1 = PerlinNoise::_to_vec(self.grid[sy
                                        * (self.dim as usize) + sx]);
        let c2 = PerlinNoise::_to_vec(self.grid[(sy + 1)
                                        * (self.dim as usize) + sx]);
        let c3 = PerlinNoise::_to_vec(self.grid[(sy + 1)
                                        * (self.dim as usize) + (sx + 1)]);
        let c4 = PerlinNoise::_to_vec(self.grid[sy
                                        * (self.dim as usize) + (sx + 1)]);

        let dc1 = (x - sx as f64, y - sy as f64);
        let dc2 = (x - sx as f64, y - (sy+1) as f64);
        let dc3 = (x - (sx+1) as f64, y - (sy+1) as f64);
        let dc4 = (x - (sx+1) as f64, y - sy as f64);

        let top_val = PerlinNoise::_lerp(
            PerlinNoise::_dot(dc1, c1),
            PerlinNoise::_dot(dc4, c4),
            PerlinNoise::_fade(1.0 - (x - sx as f64))
        );


        let bot_val = PerlinNoise::_lerp(
            PerlinNoise::_dot(dc2, c2),
            PerlinNoise::_dot(dc3, c3),
            PerlinNoise::_fade(1.0 - (x - sx as f64))
        );
        
        let val = PerlinNoise::_lerp(
            top_val,
            bot_val,
            PerlinNoise::_fade(1.0 - (y - sy as f64))
        );

        assert!(val <= 1.0);
        assert!(val >= -1.0);
        return (1.0 + val) / 2.0;
    }
}

pub fn perlin(freq: u16, size: usize) -> Vec<f64> {
    let pn = PerlinNoise::new(freq);
    let img_size = size;

    let mut out = Vec::new();

    for x in 0..size {
        for y in 0..size {
            let val = pn.sample(x as f64 / img_size as f64,
                                y as f64 / img_size as f64);
            out.push(val);
        }
    }

    return out;
}
