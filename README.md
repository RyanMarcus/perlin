# Perlin

A Perlin noise generator written in Rust.


```
Ryan Marcus <ryan@ryanmarc.us>
A program to make (optionally animated) Perlin flow fields

USAGE:
    perlin [FLAGS] [OPTIONS]

FLAGS:
    -a               Create an animation (instead of a single frame)
        --help       Prints help information
        --octaves    Produce Perlin noise with multiple pre-defined octaves
    -p, --perlin     Just produce Perlin noise with width size
    -V, --version    Prints version information

OPTIONS:
    -c <color function>                  The coloring function to use (0 through 3) [default: 0]
    -f, --noise-frequency <frequency>    Frequency of Perlin noise [default: 3]
    -h <height>                          height of the output image [default: 540]
    -j, --jobs <number of jobs>          Number of parallel jobs [default: 1]
    -o <output>                          Name of output file [default: out]
    -w <width>                           Width of the output image [default: 960]
```
