extern crate image;

use std::path::{self, Path};

fn main() {
    let N = 5;
    let img = image::open(Path::new("image.jpg")).unwrap()
        .to_rgb();

    // Initialize pixel array
    let mut pixels: Vec<[u8;3]> = Vec::new();
    {
        let img = img.clone().into_raw();
        for i in 0..img.len()/3 {
            pixels.push(clone_array(&img[3*i..3*(i+1)]));
        }
    }
    pixels.sort();
    // pixels.dedup();

    // Initialize centroids
    let mut centroids: Vec<[u8;3]> = Vec::new();
    for i in 0..N {
        let val = i * (255 / N);
        centroids.push([val, val, val]);
    }

    // Begin main loop
    loop {
        // Initialize bins
        let mut bins: Vec<Vec<&[u8;3]>> = Vec::new();
        for _ in 0..N { bins.push(Vec::new()); }

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
            for p in bin {
                r += p[0] as usize;
                g += p[1] as usize;
                b += p[2] as usize;
            }
            let r = (r / bin.len()) as u8;
            let b = (b / bin.len()) as u8;
            let g = (g / bin.len()) as u8;
            palette.push([r, g, b]);
        }

        // TODO: Add iteration limit
        if palette == centroids { break } else {
            centroids = palette;
        }
    }

    for color in centroids {
        println!("{}", rgb_string(color));
    }
}

fn rgb_string(p: [u8;3]) -> String {
    format!("#{:02x}{:02x}{:02x}", p[0], p[1], p[2])
}

fn clone_array<T: Copy>(slice: &[T]) -> [T; 3] {
    [slice[0], slice[1], slice[2]]
}

