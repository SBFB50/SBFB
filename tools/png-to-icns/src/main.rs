// SPDX-License-Identifier: AGPL-3.0-or-later
//! Convert a PNG image to a macOS ICNS icon file.
//!
//! Usage: `png-to-icns <input.png> <output.icns>`
//!
//! Generates icon entries at standard macOS sizes (16, 32, 64,
//! 128, 256, 512 pixels). Sizes larger than the source image
//! are skipped.

use std::io::BufWriter;
use std::path::PathBuf;

use icns::{IconFamily, IconType, Image, PixelFormat};
use image::imageops::FilterType;
use image::GenericImageView;

const ICON_SIZES: &[(u32, IconType)] = &[
    (16, IconType::RGBA32_16x16),
    (32, IconType::RGBA32_32x32),
    (64, IconType::RGBA32_64x64),
    (128, IconType::RGBA32_128x128),
    (256, IconType::RGBA32_256x256),
    (512, IconType::RGBA32_512x512),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: png-to-icns <input.png> <output.icns>");
        std::process::exit(1);
    }

    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let source = image::open(&input).unwrap_or_else(|e| {
        eprintln!("Failed to open {}: {e}", input.display());
        std::process::exit(1);
    });

    let (src_w, src_h) = source.dimensions();
    eprintln!("Source: {}x{}", src_w, src_h);

    let mut family = IconFamily::new();
    let mut count = 0u32;

    for &(size, icon_type) in ICON_SIZES {
        if size > src_w || size > src_h {
            eprintln!("Skipping {size}x{size} (source too small)");
            continue;
        }

        let resized = source.resize_exact(size, size, FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let icon_image = Image::from_data(
            PixelFormat::RGBA,
            icon_type.pixel_width(),
            icon_type.pixel_height(),
            rgba.into_raw(),
        )
        .unwrap_or_else(|e| {
            eprintln!("Failed to create {size}x{size} icon: {e}");
            std::process::exit(1);
        });

        family
            .add_icon_with_type(&icon_image, icon_type)
            .unwrap_or_else(|e| {
                eprintln!("Failed to add {size}x{size} icon: {e}");
                std::process::exit(1);
            });

        count += 1;
    }

    if count == 0 {
        eprintln!("No icon sizes could be generated (source too small)");
        std::process::exit(1);
    }

    let file = std::fs::File::create(&output).unwrap_or_else(|e| {
        eprintln!("Failed to create {}: {e}", output.display());
        std::process::exit(1);
    });

    family.write(BufWriter::new(file)).unwrap_or_else(|e| {
        eprintln!("Failed to write ICNS: {e}");
        std::process::exit(1);
    });

    eprintln!("Wrote {} icon sizes to {}", count, output.display());
}
