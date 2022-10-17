use core::num;
use num_traits::cast::ToPrimitive;
use std::path::{Path, PathBuf};

use rexiv2;

#[derive(Clone, Debug)]
struct Exposure {
    speed: f64,
    iso: f64,
    ev: Option<f64>,
    tv: Option<f64>,
    aperture: Option<f64>,
    flash: bool,
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
            let canon_measured_ev = "Exif.CanonSi.MeasuredEV";
            let ev = if meta.has_tag(&canon_measured_ev) {
                Some(to_ev(meta.get_tag_numeric(&canon_measured_ev)))
            } else {
                None
            };

            Some(Self {
                speed: meta.get_exposure_time().unwrap().to_f64().unwrap(),
                iso: meta.get_iso_speed().unwrap().into(),
                ev,
                tv: meta
                    .get_tag_rational(&"Exif.Photo.ShutterSpeedValue")
                    .map(|x| x.to_f64().unwrap()),
                aperture: meta.get_fnumber().map(|f| f.to_f64().unwrap()),
                flash: meta.get_tag_numeric("Exif.Photo.Flash") != 0i32,
            })
        } else {
            None
        }
    }

    fn compute_aperture(&self) -> (Option<f64>, Option<f64>) {
        if self.flash {
            (None, None)
        } else {
            (
                if self.ev.is_some() {
                    Some(2f64.powf(
                        (self.ev.unwrap() + self.speed.log2() + (self.iso / 100.).log2()) * 0.5,
                    ))
                } else {
                    None
                },
                if self.ev.is_some() && self.tv.is_some() {
                    Some(2f64.powf(
                        0.5 * (self.ev.unwrap() - self.tv.unwrap() + (self.iso / 100.).log2()),
                    ))
                } else {
                    None
                },
            )
        }
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
            if "jpg" != path.extension().unwrap().to_ascii_lowercase() {
                continue;
            }
            if let Some(exposure) = Exposure::from_exif(&path) {
                println!(
                    "{:?} {:?} so {:?}",
                    path,
                    exposure,
                    exposure.compute_aperture(),
                );
            } else {
                println!("Skipping {:?}", path);
            }
        }
    }
}
