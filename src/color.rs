use na::{Point4, Vector4};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color(pub Point4<u8>);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{},{})", self.r(), self.g(), self.b(), self.a())
    }
}

impl Color {
    pub fn r(&self) -> u8 {
        return self.0[0];
    }

    pub fn g(&self) -> u8 {
        return self.0[1];
    }

    pub fn b(&self) -> u8 {
        return self.0[2];
    }

    pub fn a(&self) -> u8 {
        return self.0[3];
    }

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color([r, g, b, a].into())
    }

    pub fn find_most_common(counts: &HashMap<Color, u32>) -> Self {
        *counts.iter().max_by_key(|(_, v)| *v).unwrap().0
    }

    pub fn find_average(counts: &HashMap<Color, u32>) -> Self {
        assert!(!counts.is_empty());
        let total_pixels = counts.iter().map(|(_, v)| *v as u64).sum::<u64>();
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        let mut a = 0u64;
        for (c, cnt) in counts {
            r += c.r() as u64 * *cnt as u64;
            g += c.g() as u64 * *cnt as u64;
            b += c.b() as u64 * *cnt as u64;
            a += c.a() as u64 * *cnt as u64;
        }
        Color::new(
            (r / total_pixels) as u8,
            (g / total_pixels) as u8,
            (b / total_pixels) as u8,
            (a / total_pixels) as u8,
        )
    }
}

fn find_centroid(colors: &Vec<Color>) -> Point4<f32> {
    let total_pixels = colors.len();
    let mut r = 0usize;
    let mut g = 0usize;
    let mut b = 0usize;
    let mut a = 0usize;
    for c in colors.iter() {
        r += c.r() as usize;
        g += c.g() as usize;
        b += c.b() as usize;
        a += c.a() as usize;
    }

    [
        (r / total_pixels) as f32,
        (g / total_pixels) as f32,
        (b / total_pixels) as f32,
        (a / total_pixels) as f32,
    ]
    .into()
}

impl From<Point4<f32>> for Color {
    fn from(data: Point4<f32>) -> Self {
        Color(na::convert_unchecked::<Point4<f32>, Point4<u8>>(data))
    }
}

/// taken from https://github.com/liborty/rstats
fn gmedian(colors: &Vec<Color>, eps: f32) -> Color {
    let fcolors: Vec<Point4<f32>> = colors.iter().map(|v| na::convert(v.0)).collect();

    let mut g = find_centroid(colors);
    let mut recsum = 0f32;
    loop {
        let mut nextg = Vector4::<f32>::zeros();
        let mut nextrecsum = 0_f32;
        for v in fcolors.iter() {
            let mag = na::distance(&v, &g);
            if mag > eps {
                let rec = 1.0_f32 / mag.sqrt();
                // vsum increment by components
                nextg += v.coords * rec;
                nextrecsum += rec // add separately the reciprocals for final scaling
            } // else simply ignore this point should its distance from g be zero
        }
        nextg /= nextrecsum;
        if nextrecsum - recsum < eps {
            return Point4::from(nextg).into();
        }
        g = Point4::from(nextg);
        recsum = nextrecsum;
    }
}

/// taken from https://github.com/liborty/rstats
fn pmedian(colors: &Vec<Color>, eps: f32) -> Color {
    let fcolors: Vec<Point4<f32>> = colors.iter().map(|v| na::convert(v.0)).collect();
    let mut g = find_centroid(colors);
    // running global sum of reciprocals
    let mut recsum = 0f32;

    // running global sum of unit vectors
    let mut vsum = Vector4::<f32>::zeros();
    // previous reciprocals for each point
    let mut precs: Vec<f32> = Vec::with_capacity(colors.len());
    // termination flag triggered by any one point
    let mut terminate = true;

    // initial vsum,recsum and precs
    for p in &fcolors {
        // square magnitude of p - g
        let magsq = (p - g).norm_squared();
        if magsq < eps {
            precs.push(0.);
            continue;
        }; // skip this point, it is too close
        let rec = 1.0 / (magsq.sqrt());
        // vsum incremented by components of unit vector
        vsum += rec * p.coords;
        precs.push(rec); // store rec for this p
        recsum += rec;
    }

    // first iteration done, update g
    g = Point4::from(vsum / recsum);
    loop {
        // vector iteration till accuracy eps is exceeded
        for (p, rec) in fcolors.iter().zip(&mut precs) {
            let magsq = (p - g).norm_squared();
            if magsq < eps {
                *rec = 0.0;
                continue;
            }; // skip this point, it is too close
            let recip = 1.0 / (magsq.sqrt());
            let recdelta = recip - *rec; // change in reciprocal for p
            *rec = recip; // update rec for this p for next time
                          // vsum updated by components
            vsum += recdelta * p.coords;
            // update recsum
            recsum += recdelta;
            // update g immediately for each point p
            g = Point4::from(vsum / recsum);
            // termination condition detected but do the rest of the points anyway
            if terminate && recdelta.abs() > eps {
                terminate = false
            };
        }
        if terminate {
            return g.into();
        }; // termination reached
        terminate = true
    }
}
