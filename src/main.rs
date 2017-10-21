#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate image;

#[macro_use]
extern crate serde_derive;
extern crate docopt;

use std::path::Path;

docopt!(Args derive Debug, "
Usage: parrot [options] FILE
       parrot (-h|--help)

Options:
-B, --bins N        Number of colors to generate. [default: 8]
-a, --approx        Approximate average with closest color in the image.
--unweighted        Use only unique colors to generate palette.
-T, --true-color    Print colors in true color.
", flag_bins: usize);

fn main() {
    // Parse arguments
    let args: Args = Args::docopt().deserialize().unwrap_or_else(|e| e.exit());
    let img = match image::open(Path::new(&args.arg_FILE)) {
        Ok(f) => f,
        Err(_) => { 
            println!("Error: File `{}` does not exist or could not be opened!", args.arg_FILE);
            return
        },
    }.to_rgb();

    // Initialize pixel array
    let mut pixels: Vec<[u8;3]> = Vec::new();
    {
        let img = img.clone().into_raw();
        for i in 0..img.len()/3 {
            pixels.push(clone_array(&img[3*i..3*(i+1)]));
        }
    }
    pixels.sort();
    if args.flag_unweighted { pixels.dedup(); }

    // Initialize centroids
    let mut centroids: Vec<[u8;3]> = vec![[0, 0, 0]; args.flag_bins];
    {
        let mut pixels_trunc = pixels.clone();
        pixels_trunc.dedup();

        // Generate current sum of distances for all centroids
        let mut max_d_vec = Vec::new();
        for i in 0..args.flag_bins {
            let mut d = 0;
            for j in 0..args.flag_bins {
                if i == j { continue }
                d += color_dist(&centroids[i], &centroids[j]);
            }
            max_d_vec.push(d);
        }
        
        for p in pixels_trunc {
            let mut max_i = 0;
            let mut max_d = 0;

            // Calculate distance of color from every other point
            // for each candidate point
            for i in 0..args.flag_bins {
                let mut d = 0;
                for j in 0..args.flag_bins {
                    if i == j { continue }
                    d += color_dist(&p, &centroids[j])
                }
                if d >= max_d && d > max_d_vec[i] {
                    max_i = i;
                    max_d = d;
                }
            }
            if max_d > max_d_vec[max_i] {
                max_d_vec[max_i] = max_d;
                centroids[max_i] = p;
            }
        }
    }

    // Begin main loop
    loop {
        // Initialize bins
        let mut bins: Vec<Vec<&[u8;3]>> = Vec::new();
        for _ in 0..args.flag_bins { bins.push(Vec::new()); }

        // Binning loop vars
        let mut first_cycle = true;
        let mut prv_bin_index = 0;
        let mut prv_p = &[0, 0, 0];

        // Start binning process
        for p in &pixels {
            if first_cycle || prv_p != p {
                if first_cycle { first_cycle = false; }

                let mut min_d = 765; // max distance (3 * 255)
                let mut bin_index = 0;
                let mut index = 0;
                for &c in &centroids {
                    let d = (p[0] as i32 - c[0] as i32).abs() +
                        (p[1] as i32 - c[1] as i32).abs() +
                        (p[2] as i32 - c[2] as i32).abs();
                    if d <= min_d {
                        min_d = d;
                        bin_index = index;
                    }
                    index += 1;
                }
                bins[bin_index].push(p);
                prv_bin_index = bin_index;
                prv_p = p;
            } else if prv_p == p {
                bins[prv_bin_index].push(p);
            }
        }

        // Calculate palette
        let mut palette: Vec<[u8;3]> = Vec::new();
        for bin in &bins {
            let (mut r, mut g, mut b) = (0, 0, 0);
            let len = if bin.len() != 0 { bin.len() } else { 1 };
            for p in bin {
                r += p[0] as usize;
                g += p[1] as usize;
                b += p[2] as usize;
            }
            let r = (r / len) as u8;
            let b = (b / len) as u8;
            let g = (g / len) as u8;
            palette.push([r, g, b]);
        }

        // TODO: Add iteration limit
        if palette == centroids {
            if args.flag_approx {
                for i in 0..bins.len() {
                    let mut closest_color = [0, 0, 0];
                    let mut min_d = 765;
                    for &color in &bins[i] {
                        let d = color_dist(color, &centroids[i]);
                        if d < min_d {
                            closest_color = *color;
                            min_d = d;
                        };
                    }
                    centroids[i] = closest_color;
                }
            }
            break;
        } else {
            centroids = palette;
        }
    }

    centroids.sort();
    for color in centroids {
        println!("{}", rgb_string(&color, args.flag_true_color));
    }
}

fn rgb_string(p: &[u8;3], true_color: bool) -> String {
    if true_color {
        format!("#{r:02x}{g:02x}{b:02x} \x1b[48;2;{r};{g};{b}m       \x1b[0m", r=p[0], g=p[1], b=p[2])
    } else {
        format!("#{:02x}{:02x}{:02x}", p[0], p[1], p[2])
    }
}

fn clone_array<T: Copy>(slice: &[T]) -> [T; 3] {
    [slice[0], slice[1], slice[2]]
}

fn color_dist(a: &[u8;3], b: &[u8;3]) -> u64 {
    ((a[0] as i32 - b[0] as i32).abs() +
     (a[1] as i32 - b[1] as i32).abs() +
     (a[2] as i32 - b[2] as i32).abs()) as u64
}
