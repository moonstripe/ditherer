# Ditherer CLI Utility

A command-line utility for applying Bayer matrix dithering to images. The tool supports grayscale and color dithering using various Bayer matrix sizes (2x2, 4x4, 8x8) and offers options to preserve pixel brightness order during dithering.

## Features

- Apply Bayer dithering to grayscale or color images.
- Choose between different Bayer matrix sizes: `2x2`, `4x4`, or `8x8`.
- Optionally preserve the order of light or dark pixels during color dithering.
- Input image can be provided either from a file or piped from stdin.
- Output image can be saved to a file or printed to stdout.

## Installation

### Using Cargo (Rust)

If you have Rust installed, you can build and run the tool with the following commands:

```bash
cargo install ditherer
```

### Precompiled Binaries

Alternatively, you can download precompiled binaries for your platform from the releases section of this repository.

## Usage

```bash
ditherer [OPTIONS]
```

### Options

- `-i, --input <INPUT_IMG>`  
  Path to the input image file (optional). If not provided, the image will be read from stdin.

- `-o, --output <OUTPUT_IMG>`  
  Path to save the output image (optional). If not provided, the output will be written to stdout.

- `-m, --matrix-size <MATRIX_SIZE>`  
  Specify the Bayer matrix size for dithering. Options:

  - `m2`: 2x2 matrix
  - `m4`: 4x4 matrix
  - `m8`: 8x8 matrix

- `-c, --color`  
  Apply dithering on the brightness channel of color images. By default, dithering will be applied to grayscale images.

- `-p, --preserve-order <PRESERVE_ORDER>`  
  When color dithering is enabled, specify whether to preserve the "dark" or "light" pixels' order. Options:

  - `dark`: Preserve dark pixels' order.
  - `light`: Preserve light pixels' order.

- `-h, --help`  
  Show help message.

- `--version`  
  Show the version of the tool.

## Examples

### Grayscale Dithering with 4x4 Bayer Matrix

```bash
ditherer -i input.png -o output.png -m m4
```

### Color Dithering with 8x8 Bayer Matrix and Preserve Dark Pixels' Order

```bash
ditherer -i input.png -o output.png -m m8 -c -p dark
```

### Piping Image Data from stdin (Grayscale)

```bash
cat input.png | ditherer -o output.png -m m2
```

### Output to stdout (Grayscale)

```bash
ditherer -i input.png -m m4
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
