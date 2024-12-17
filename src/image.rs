use eyre::{eyre, Error, Result};
use image::ImageEncoder;
use image::{
    codecs::png::{CompressionType, FilterType},
    guess_format, load_from_memory,
};
use serde::Deserialize;
use std::{fmt::Display, io::Cursor};
use utoipa::{IntoParams, ToSchema};

/// `convert` converts the `Format` of a given image from its underliying format
/// to the given one.
fn convert(file: &[u8], to: Format, quality: Option<u8>) -> Result<Vec<u8>, Error> {
    // let img = load_from_memory(file)?;
    // let mut converted_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    // img.write_to(&mut converted_img, to.into())?;
    // // Ok(converted_img.get_ref().clone())
    let img = load_from_memory(file)?;
    let mut converted_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    match to {
        Format::Jpeg => {
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut converted_img,
                quality.unwrap_or(80),
            );
            encoder.write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                img.color().into(),
            )?;
        }
        Format::Png => {
            // Decide compression type based on quality
            let compression_type = if quality.unwrap_or(80) < 10 {
                CompressionType::Fast
            } else if quality.unwrap_or(80) < 20 {
                CompressionType::Default
            } else {
                CompressionType::Best
            };

            let encoder = image::codecs::png::PngEncoder::new_with_quality(
                &mut converted_img,
                compression_type,
                FilterType::NoFilter,
            );
            encoder.write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                img.color().into(),
            )?;
        }
        _ => {
            // For lossless formats, ignore quality
            img.write_to(&mut converted_img, to.into())?;
        }
    }

    Ok(converted_img.get_ref().clone())
}

/// `resize_image` scales _*down*_ the given image.
fn resize(file: &[u8], d: &Dimension) -> Result<Vec<u8>, Error> {
    let img = load_from_memory(file)?;
    let (width, height) = match d {
        Dimension(Some(Width(w)), Some(Height(h))) => (*w, *h),
        Dimension(Some(Width(w)), None) => (*w, img.height()),
        Dimension(None, Some(Height(h))) => (img.width(), *h),
        Dimension(None, None) => return Err(eyre!("No dimensions specified")),
    };
    let img = img.thumbnail(width, height);
    let mut resized_img: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    img.write_to(&mut resized_img, guess_format(file)?)?;
    Ok(resized_img.get_ref().clone())
}

pub enum Operation {
    Convert(Format),
    Resize(Dimension),
    Quality(u8),
}

/// `transform` performs all needed operations on an image (byte slice)
/// # Errors
/// - When trying to guess the image format
pub fn transform(file: &[u8], op: &[Operation]) -> Result<Vec<u8>, Error> {
    op.iter().try_fold(Vec::with_capacity(0), |acc, o| {
        // don't use accumulator on our first pass
        // and check that the passed value is a valid image
        // this results in a small overhead
        // (compared to just checking for a non-empty file)
        // in this case its probably an overkill.
        guess_format(file)?;
        let acc = if acc.is_empty() { file } else { &acc };

        match o {
            Operation::Convert(f) => convert(acc, *f, None),
            Operation::Resize(s) => resize(acc, s),
            Operation::Quality(q) => {
                // For now, only works with JPEG and WebP
                let format = guess_format(acc)?;
                match format {
                    image::ImageFormat::Jpeg => convert(acc, Format::Jpeg, Some(*q)),
                    image::ImageFormat::WebP => convert(acc, Format::Webp, Some(*q)),
                    _ => Ok(acc.to_vec()), // No-op for other formats
                }
            }
        }
    })
}

/// Format of the image
#[derive(Clone, Copy, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Avif,
    Png,
    Jpeg,
    Webp,
}

impl Format {
    #[must_use]
    pub const fn content_type(&self) -> &str {
        match self {
            Self::Avif => "image/avif",
            Self::Webp => "image/webp",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Avif => "avif",
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Webp => "webp",
        };
        write!(f, "{name}")
    }
}

impl From<Format> for image::ImageFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::Avif => Self::Avif,
            Format::Png => Self::Png,
            Format::Webp => Self::WebP,
            Format::Jpeg => Self::Jpeg,
        }
    }
}

/// Width of an image
#[derive(Clone, Copy, Debug, Deserialize, ToSchema, IntoParams)]
#[into_params(names("width"))]
pub struct Width(pub u32);
/// Height of an image
#[derive(Clone, Copy, Debug, Deserialize, ToSchema, IntoParams)]
#[into_params(names("height"))]
pub struct Height(pub u32);
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct Dimension(pub Option<Width>, pub Option<Height>);

impl Display for Width {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match (self.0, self.1) {
            (Some(w), Some(h)) => &format!("{w}_{h}"),
            (Some(w), None) => &format!("{w}_original"),
            (None, Some(h)) => &format!("original_{h}"),
            (None, None) => "original_original",
        };
        write!(f, "{output}")
    }
}
