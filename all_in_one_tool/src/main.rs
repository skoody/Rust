use std::env;
use std::fs::File;
use std::io::BufReader;
use png::chunk::ChunkType;

// Define a mapping from chunk types to their descriptions
const CHUNK_DESCRIPTIONS: &[(ChunkType, &str)] = &[
    (png::chunk::IHDR, "Image Header: Contains basic image information."),
    (png::chunk::PLTE, "Palette: Contains the color palette for indexed-color images."),
    (png::chunk::IDAT, "Image Data: Contains the actual image data."),
    (png::chunk::IEND, "Image End: Marks the end of the PNG datastream."),
    (png::chunk::TRNS, "Transparency: Contains transparency information."),
    (png::chunk::CHRM, "Primary Chromaticities: Specifies the chromaticity of the display primaries."),
    (png::chunk::GAMA, "Gamma: Specifies the gamma of the image."),
    (png::chunk::ICCP, "ICC Profile: Contains the International Color Consortium profile of the image."),
    (png::chunk::SBIT, "Significant Bits: Defines the number of significant bits for each color channel."),
    (png::chunk::SRGB, "sRGB Color Space: Indicates that the standard sRGB color space is used."),
    (png::chunk::TEXT, "Textual Data (tEXt): Contains textual metadata (ISO 8859-1)."),
    (png::chunk::ZTXT, "Compressed Textual Data (zTXt): Contains compressed textual metadata."),
    (png::chunk::ITXT, "International Textual Data (iTXt): Contains textual metadata (UTF-8)."),
    (png::chunk::BKGD, "Background Color: Specifies the default background color."),
    (png::chunk::HIST, "Image Histogram: Stores the histogram of the image."),
    (png::chunk::PHYS, "Physical Pixel Dimensions: Defines the physical size of the pixels."),
    (png::chunk::SPLT, "Suggested Palette: Provides a suggested palette to use if the display device cannot show all colors."),
    (png::chunk::TIME, "Last Modification Time: Contains the time of the last image modification."),
    (png::chunk::OFFS, "Image Offset: Specifies the position of the image on a larger canvas."),
    (png::chunk::PCAL, "Pixel Calibration: Calibrates the scale of pixel values."),
    (png::chunk::SCAL, "Physical Scaling: Defines the physical scaling of the image subject."),
    (png::chunk::GIFG, "GIF Graphic Control Extension: Metadata for GIF compatibility."),
    (png::chunk::GIFT, "GIF Plain Text Extension: Metadata for GIF compatibility."),
    (png::chunk::GIFX, "GIF Application Extension: Metadata for GIF compatibility."),
];

fn get_chunk_description(chunk_type: ChunkType) -> &'static str {
    CHUNK_DESCRIPTIONS.iter()
        .find(|(ct, _)| *ct == chunk_type)
        .map(|(_, desc)| *desc)
        .unwrap_or("Unknown Chunk Type: No standard description available.")
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_png_file>", args[0]);
        return;
    }

    let file_path = &args[1];
    println!("--- Reading PNG File: {} ---\n", file_path);

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path, e);
            return;
        }
    };

    let decoder = png::Decoder::new(file);
    let reader = match decoder.read_info() {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error reading PNG info: {}", e);
            return;
        }
    };

    let info = reader.info();

    println!("[+] --- General Image Information ---");
    println!("      Size: {}x{} pixels", info.width, info.height);
    println!("      Color Type: {:?}", info.color_type);
    println!("      Bit Depth: {:?}", info.bit_depth);
    println!("      Compression: {:?}", info.compression);
    println!("      Filter Method: {:?}", info.filter);
    println!("      Interlaced: {}", if info.interlaced { "Yes" } else { "No" });

    if let Some(gamma) = info.gamma {
        println!("      Gamma: {}", gamma);
    }
    if let Some(srgb) = info.srgb {
        println!("      sRGB Rendering Intent: {:?}", srgb);
    }
    if let Some(palette) = &info.palette {
         println!("      Palette: {} colors", palette.len() / 3);
    }
     if let Some(trns) = &info.trns {
        println!("      Transparency Chunk (tRNS) present. Length: {}", trns.len());
    }
    println!("\n[+] --- Metadata Chunks Found ---");
    if let Some(chunks) = &info.source_chunks {
        if chunks.is_empty() {
            println!("      No metadata chunks found in the file.");
        } else {
            for (chunk_type, chunk_data) in chunks {
                println!("\n  -> Chunk Type: {}", chunk_type);
                println!("     Description: {}", get_chunk_description(*chunk_type));
                println!("     Size: {} bytes", chunk_data.len());
                // Optionally, print the first few bytes of the chunk data for inspection
                // let preview_len = std::cmp::min(16, chunk_data.len());
                // println!("     Data Preview: {:x?}", &chunk_data[..preview_len]);
            }
        }
    } else {
        println!("      Could not read source chunks information.");
    }

    println!("\n--- Finished processing. ---");
}
