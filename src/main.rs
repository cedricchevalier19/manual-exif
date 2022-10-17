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
}

// https://github.com/alchemy-fr/exiftool/blob/master/lib/Image/ExifTool/Canon.pm
fn to_ev(input: i32) -> f64 {
    input as f64 / 32.0 + 5.0
}

impl Exposure {
    fn from_exif(path: &Path) -> Self {
        let meta = rexiv2::Metadata::new_from_path(path).unwrap();
        // meta.get_exif_tags().unwrap().into_iter().for_each(|tag| {
        //     println!("{} {:?}", tag, meta.get_tag_multiple_strings(&tag));
        // });
        Self {
            speed: meta.get_exposure_time().unwrap().to_f64().unwrap(),
            iso: meta.get_iso_speed().unwrap().into(),
            ev: to_ev(meta.get_tag_numeric(&"Exif.CanonSi.MeasuredEV")),
            tv: meta
                .get_tag_rational(&"Exif.Photo.ShutterSpeedValue")
                .unwrap()
                .to_f64()
                .unwrap(),
        }
    }

    fn compute_aperture(&self) -> (f64, f64) {
        (
            2f64.powf((self.ev + self.speed.log2() - (self.iso / 100.).log2()) * 0.5),
            2f64.powf(0.5 * (self.ev - self.tv) + (self.iso / 100.).log2()),
        )
    }
}

fn read_exif(path: &Path) {
    let meta = rexiv2::Metadata::new_from_path(path).unwrap();
    meta.get_exif_tags().unwrap().into_iter().for_each(|tag| {
        println!("{} {:?}", tag, meta.get_tag_multiple_strings(&tag));
    });
}

fn main() {
    for fname in ["img.jpg", "IMG_5556_DxO.jpg"] {
        let mut path = PathBuf::new();
        path.push("tests");
        path.push(fname);
        //read_exif(&path);
        {
            let exposure = Exposure::from_exif(&path);
            println!("{:?} so {:?}", exposure, exposure.compute_aperture());
        }
    }
}
