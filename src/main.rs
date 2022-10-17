use core::num;
use num_traits::cast::ToPrimitive;
use std::path::{Path, PathBuf};

use rexiv2;

#[derive(Clone, Debug)]
struct Exposure {
    speed: f64,
    iso: f64,
    ev: f64,
    tv: f64,
    aperture: Option<f64>,
}

// https://github.com/alchemy-fr/exiftool/blob/master/lib/Image/ExifTool/Canon.pm
fn to_ev(input: i32) -> f64 {
    input as f64 / 32.0 + 5.0
}

impl Exposure {
    fn from_exif(path: &Path) -> Option<Self> {
        if let Ok(meta) = rexiv2::Metadata::new_from_path(path) {
            // meta.get_exif_tags().unwrap().into_iter().for_each(|tag| {
            //     println!("{} {:?}", tag, meta.get_tag_multiple_strings(&tag));
            // });
            Some(Self {
                speed: meta.get_exposure_time().unwrap().to_f64().unwrap(),
                iso: meta.get_iso_speed().unwrap().into(),
                ev: to_ev(meta.get_tag_numeric(&"Exif.CanonSi.MeasuredEV")),
                tv: meta
                    .get_tag_rational(&"Exif.Photo.ShutterSpeedValue")
                    .unwrap()
                    .to_f64()
                    .unwrap(),
                aperture: meta.get_fnumber().map(|f| f.to_f64().unwrap()),
            })
        } else {
            None
        }
    }

    fn compute_aperture(&self) -> (f64, f64) {
        (
            2f64.powf((self.ev + self.speed.log2() + (self.iso / 100.).log2()) * 0.5),
            2f64.powf(0.5 * (self.ev - self.tv + (self.iso / 100.).log2())),
        )
    }

    fn compute_apex(&self) -> (f64, f64) {
        (self.ev - self.tv, self.tv + self.speed.log2())
    }
}

fn read_exif(path: &Path) {
    let meta = rexiv2::Metadata::new_from_path(path).unwrap();
    meta.get_exif_tags().unwrap().into_iter().for_each(|tag| {
        println!("{} {:?}", tag, meta.get_tag_multiple_strings(&tag));
    });
}

fn main() {
    let read_path = std::env::args()
        .nth(1)
        .unwrap_or("/nfs/photo/cedric/jpeg/public/2022/2022-09-jardin".to_string());
    for fname in std::fs::read_dir(read_path).unwrap() {
        {
            let path = fname.unwrap().path();
            if let Some(exposure) = Exposure::from_exif(&path) {
                println!(
                    "{:?} {:?} so {:?} {:?}",
                    path,
                    exposure,
                    exposure.compute_aperture(),
                    exposure.compute_apex()
                );
            } else {
                println!("Skipping {:?}", path);
            }
        }
    }
}
