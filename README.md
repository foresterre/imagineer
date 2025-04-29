<img alt="The imagineer project logo" align="right" src="resources/logo/imagineer_100.jpg">

# imagineer image cli


[![ci](https://github.com/foresterre/imagineer/workflows/github_actions_ci/badge.svg)](https://github.com/foresterre/imagineer/actions?query=workflow%3Agithub_actions_ci)
[![Crates.io version shield](https://img.shields.io/crates/v/imagineer.svg)](https://crates.io/crates/imagineer)
[![Docs](https://docs.rs/imagineer/badge.svg)](https://docs.rs/crate/imagineer)
[![Crates.io license shield](https://img.shields.io/crates/l/imagineer.svg)](https://crates.io/crates/imagineer)

_Convert images and perform image operations from the command-line._

`imagineer` (formerly 'sic' image cli) is a front-end for the [image crate](https://github.com/image-rs/image).
Aside from image operations supplied by the image crate, a few additional helpful operations such
as diff, and some provided via [imageproc](https://github.com/image-rs/imageproc) are included. 
Imagineer has support for static and animated images.

**The new name**

- Why a rename? 'sic' originally stood for "simple image cli", but if you have two custom parsers, support batch image processing, static and animated images, is it still "simple"?
- A few years ago 'sic' became 'sic image cli', but mostly just for me
- Recently, I decided I like a new name, something fun, something which is closer to the word 'image', and also the name of the crate for which this project is a CLI frontend, some crate names were already taken,
- But, I found something
- Thus, now we have 'imagineer', a pun on the meaning of the words "image", "engineering" and "the imagination"

### Installation

#### Install with cargo:

* run `cargo install imagineer`

#### Pre build binaries

* download from [releases](https://github.com/foresterre/imagineer/releases).


#### Build from source

- Setup rust and cargo (for example using [rustup](https://rustup.rs/))
- Clone this repo: `git clone https://github.com/foresterre/imagineer.git && cd imagineer`
- Build a release: `cargo build --release`

`imagineer` is usually build against the latest stable Rust version, but may also work with older versions.

#### Using a package manager

<details><summary>Homebrew</summary>
<p>

🍺 Homebrew on MacOS:

```shell
brew tap tgotwig/sic
brew install tgotwig/sic/sic
```

🍺 Homebrew on Linux:

```shell
brew tap tgotwig/linux-sic
brew install tgotwig/linux-sic/sic
```

</p>
</details>

<details><summary>emerge</summary>

**gentoo** linux via GURU overlay:

```sh
emerge -av media-gfx/sic
```
</details>

### Usage

##### Convert images

Convert an image from one format to another, for example from PNG to JPG.

* Command: `imagineer --input <input> --output <output>`
* Shorthand: `imagineer -i <input> -o <output>`
* Example: `imagineer -i input.png -o output.jpg` <br>

If you want to explicitly set the image output format, you may do so by providing the `--output-format <format>` argument.
Otherwise, imagineer will attempt to infer the format from the output file extension.

`--help` can be used to view a complete list of supported image output formats. Included are:
* `AVIF`, `BMP`, `OpenExr`, `Farbfeld`, `GIF`, `ICO`, `JPEG`, `PNG`, `PNM` (`PAM`, `PBM`, `PGM` and `PPM`), `QOI` `TGA`, `TIFF` and `WebP`.



The JPEG quality can optionally be set with `--jpeg-encoding-quality <value>`. The value should be in the range 1-100 (with default 80).
Files which are formatted with a PNM format (with one subtype of PBM, PGM and PPM) use binary encoding (PNM P4, P5 and P6 respectively) by default.
To use ascii encoding, you can provide the following flag: `--pnm-encoding-ascii`.

##### Convert or apply operations on a set of images

For the use case where you have a directory containing several (hundreds of) images which you like to convert to different
format, or on which you perhaps want to apply certain image operations, `imagineer` provides built-in glob pattern matching.
This mode can be used by providing the `--glob-input` and `--glob-output` options instead of `--input` and `--output` respectively.

Examples:
* To convert a directory of images from PNG to JPG, you can run imagineer with the following arguments: <br>
    * `imagineer --glob-input "*.png" --glob-output output_dir --output-format jpg"`
* To convert all images with the `jpg`, `jpeg` and `png` extensions to BMP:
    * `imagineer --glob-input "*.{jpg, jpeg, png}" --glob-output output_dir --output-format bmp`
* To emboss all images in a folder (assuming it contains only supported image files and no folders):
    * `imagineer --glob-input "*" --glob-output embossed_output --filter3x3 -1 -1 0 -1 1 1 0 1 1`

A few things worth noticing: 1) We use quotation marks (`"`) around the input argument, so our shell won't expand the
glob pattern to a list of files. 2) When using glob mode, our output (`--glob-output`) should be a folder instead of a file. 3) We
need to explicitly state the output format with `--output-format`, unless we work with a known extension we want to keep.

Output images are placed in the output folder using the directory structure mirrored from the first common directory of
all input files. If output directories do not exist, they will be created. 


<br>

##### Image operations

With `imagineer`, you can manipulate images using image operations. Image operations can be used directly from the CLI, or through
imagineer's _image script_. 

NB: Operations are applied in a left-to-right order and are generally not commutative. This may be especially surprising when
applying image operation via CLI options and flags.

###### 📜 image script

Use this method by using the `--apply-operations "<operations>"` (shorthand: `-x`) cli argument and providing
statements which tell `imagineer` what operations should be applied on the image, for example: <br>
`imagineer -i input.jpg -o output.jpg --apply-operations "flip-horizontal; blur 10; resize 250 250"` <br>
When more than one image operation is provided, the separator `;` should be used to separate each operation statement. <br><br>

###### ✏️ CLI ops

Use this method by providing cli image operation arguments, such as `--blur` and `--crop`, directly.  
If we use the _cli operations_ method the previously shown example becomes: <br>
`imagineer -i input.png -o output.jpg --flip-horizontal --blur 10 --resize 250 250` <br>
<br><br>

##### Available image operations

|operations|syntax^1|description|
|---|---|---|
|blur               | `blur <fp>`                               | Performs a Gaussian blur on the image ([more info](https://docs.rs/image/0.19.0/image/imageops/fn.blur.html)). An argument below `0.0`, will use `1.0` instead. |
|brighten           | `brighten <int>`                          | Create a brightened version of the image. |
|contrast           | `contrast <fp>`                           | Adjust the contrast of the image. |
|crop               | `crop <uint> <uint> <uint> <uint>`        | Syntax: `crop <lx> <ly> <rx> <ry>`, where `lx` is top left corner x pixel coordinate starting at 0, `ly` is the top left corner y pixel coordinate starting at 0, `rx` is the  bottom right corner x pixel coordinate and `ry` is the bottom right corner y pixel coordinate. `rx` and `ry` should be larger than `lx` and `ly` respectively. |
|diff               | `diff <path>`                             | Diff the input image against the argument image to show which pixels are the same (white), different (red) or not part of either image (transparent). |
|draw-text ^2       | `draw-text <string> <nv:coord> <nv:rgba> <nv:size> <nv:font>` | Draw text on top of an image (note: alpha-blending is not yet supported).  |
|filter3x3          | `filter3x3 <fp9x> `                       | Apply a 3 by 3 convolution filter. |
|flip horizontal    | `flip-horizontal`                         | Flips the image on the horizontal axis. |
|flip vertical      | `flip-vertical`                           | Flips the image on the vertical axis. |
|gray scale         | `grayscale`                               | Transform each pixel to only hold an intensity of light value. Reduces the color space to contain only gray monochromatic values.|
|horizontal gradient| `horizontal-gradient <nv:rgba> <nv:rgba>` | Fill and blend the image with a horizontal gradient from left to right.  |
|hue rotate         | `hue-rotate <int>`                        | Rotates the hue, argument is in degrees. Rotates `<int>%360` degrees. |
|invert             | `invert`                                  | Invert the colours of an image. |
|overlay            | `overlay <path> <uint> <uint>`            | Overlay an image loaded from the provided argument path over the input image (at a certain position). |
|resize             | `resize <uint> <uint>`                    | Resize the image to x by y pixels. Can both up- and downscale. Uses a `lanczos3` sampling filter unless overridden. Prior to imagineer (back then still called 'sic'), v0.11, the default sampling filter was `gaussian`. |
| >                 | `set preserve-aspect-ratio <bool>`        | Enables preservation of the aspect ratio when resizing. |
| >                 | `set sampling-filter <value>`             | When resizing use the `<value>` sampling filter. Choices are `catmullrom`, `gaussian`,`lanczos3`,`nearest`,`triangle`. |
|rotate90           | `rotate90`                                | Rotate an image 90 degrees. |
|rotate180          | `rotate180`                               | Rotate an image 180 degrees. |
|rotate270          | `rotate270`                               | Rotate an image 270 degrees. |
|threshold          | `threshold`                               | Apply automatic thresholding on the image. |
|unsharpen          | `unsharpen <fp> <int>`                    | Applies an unsharpen mask to the image. The first parameter defines how much the image should be blurred and the second parameter defines a threshold. If the difference between the original and blurred image is at least the threshold, they will be subtracted from each other. Can be used to sharpen an image. |
|vertical gradient  | `vertical-gradient <nv:rgba> <nv:rgba>`   | Fill and blend the image with a vertical gradient from top to bottom.  |


^1 _The syntax in the table applies to image script, but can also be used as a reference when using image operations via CLI arguments_<br>
^2 _draw-text is only available when compiled with `imageproc-ops` feature_


##### Image operation modifiers

For some operations, their behaviour can be adapted by setting an operation modifier. These modifiers can be overwritten and they can also be reset (to their default behaviour).

|environment operation|syntax|description|
|---|---|---|
|set environment option   | `set <option> [<args 0..n>]` | Enables the use of a modifier for an operation. Any operation which uses the value of the modifier will use the set modifier value instead of the default value. Can be overwritten by calling `set` again for the same operation and modifier specifier. |
|unset environment option | `del <option>`               | Resets the modifier value. Any operation which looks at the value of this modifier will use the default value instead.|

_legend_:

`<byte>`: an 8 bit unsigned integer (positive number in range 0-255<br>
`<uint>`: a 32 bit unsigned integer (positive number)<br>
`<int>`: a 32 bit signed integer (positive or negative number)<br>
`<fp>`: a 32 bit floating-point number (real number)<br>
`<fp9x>`: 9 succeeding 32 bit floating-point numbers<br>
`<path>`: a qualified path to an image reachable from your current platform (the path should be surrounded by quotation marks, i.e. " or ')<br>
`<string>`: a valid unicode string<br>

`<nv:coord>`: a named value representing a coordinate (top left is (0, 0)), with syntax `coord(<uint>, <uint>)`<br>
`<nv:rgba>`: a named value representing an RGBA color, with syntax: `rgba(<byte>, <byte>, <byte>, <byte>)`<br>
`<nv:size>`: a named value representing a font size, with syntax: `size(<fp>)`<br>
`<nv:font>`: a named value representing a (TrueType) font file location, with syntax: `font(<path>)`<br>


##### Examples

**blur** example: <br>
`imagineer -i in.png -o out.png --apply-operations "blur 1.3;"` <br>
or <br>
`imagineer -i in.png -o out.png --blur 1.3`

**brighten** example: <br>
`imagineer -i in.png -o out.png --apply-operations "brighten 2;"` <br>
or <br>
`imagineer -i in.png -o out.png --brighten 2`

**contrast** example: <br>
`imagineer -i in.png -o out.png --apply-operations "contrast 0.7;"` <br>
or <br>
`imagineer -i in.png -o out.png --contrast 0.7`

**crop** example: <br>
`imagineer -i in.png -o out.png --apply-operations "crop 0 0 10 10;"` <br>
or <br>
`imagineer -i in.png -o out.png --crop 0 0 10 10`

**diff** example: <br>
`imagineer -i a.png -o diff_between_a_and_b.png --apply-operations "diff 'b.png'"` <br>
or <br>
`imagineer -i a.png -o diff_between_a_and_b.png --diff b.png`

| a                                      | b                                      | output                                                         |
| -------------------------------------- |--------------------------------------- | -------------------------------------------------------------- |
| ![a](resources/help-images/diff/a.png) | ![b](resources/help-images/diff/b.png) | ![output](resources/help-images/diff/diff_between_a_and_b.png) |

With an animated image:

| a                                      | b                                      | output                                                         |
| -------------------------------------- |--------------------------------------- | -------------------------------------------------------------- |
| ![a](resources/loop.gif) | ![b](resources/loop-diff.gif) | ![output](resources/help-images/diff/loop-diffed.gif) |

**draw-text** example (requires build feature `imageproc-ops`): <br>
`imagineer -i in.png -o out.png --apply-operations "draw-text '<3' coord(10, 2) rgba(255, 0, 0, 255) size(14) font('./Lato-Regular.ttf')"` <br>
or <br>
`imagineer -i in.png -o out.png --draw-text "<3" "coord(10, 2)" "rgba(255, 0, 0, 255)" "size(14)" "font('Lato-Regular.ttf')"`

| input                                         | output                                                         |
| --------------------------------------------- | -------------------------------------------------------------- |
| ![in](resources/help-images/draw-text/in.png) | ![out](resources/help-images/draw-text/out.png)                |


**filter3x3** example: <br>
`imagineer -i in.png -o out.png --apply-operations "filter3x3 -1 -1 0 -1 0 1 0 1 1"` <br>
or <br>
`imagineer -i in.png -o out.png --filter3x3 -1 -1 0 -1 0 1 0 1 1`

**flip horizontal** example: <br>
`imagineer -i in.png -o out.png --apply-operations "flip-horizontal"` <br>
or <br>
`imagineer -i in.png -o out.png --flip-horizontal`

**flip vertical** example: <br>
`imagineer -i in.png -o out.png --apply-operations "flip-vertical"` <br>
or <br>
`imagineer -i in.png -o out.png --flip-vertical`

**gray scale** example: <br>
`imagineer -i in.png -o out.png --apply-operations "grayscale"` <br>
or <br>
`imagineer -i in.png -o out.png --grayscale`

**horizontal gradient** example: <br>
`imagineer -i in.png -o out.png --apply-operations "horizontal-gradient rgba(255, 0, 0, 255) rgba(0, 0, 255, 255)"` <br>
or <br>
`imagineer -i in.png -o out.png --horizontal-gradient "rgba(255, 0, 0, 255)" "rgba(0, 0, 255, 255)"`

**hue rotate** example: <br>
`imagineer -i in.png -o out.png --apply-operations "hue-rotate -90"` <br>
or <br>
`imagineer -i in.png -o out.png --hue-rotate -90`

**invert** example: <br>
`imagineer -i in.png -o out.png --apply-operations "invert"` <br>
or <br>
`imagineer -i in.png -o out.png --invert`

**overlay** example: <br>
`imagineer -i in.png -o out.png --apply-operations "overlay 'image.png' 10 10"` <br>
or <br>
`imagineer -i in.png -o out.png --overlay "image.png" 10 10`

**resize** example: <br>
`imagineer -i in.png -o out.png --apply-operations "resize 100 100"` <br>
or <br>
`imagineer -i in.png -o out.png --resize 100 100`

**resize** with **preserve aspect ratio** example: <br>
`imagineer -i in.png -o out.png --apply-operations "set preserve-aspect-ratio true; resize 100 100"` <br>
or <br>
`imagineer -i in.png -o out.png --preserve-aspect-ratio true --resize 100 100`

**resize** with **custom sampling filter** (default is 'lanczos3') example: <br>
`imagineer -i in.png -o out.png --apply-operations "set sampling-filter triangle; resize 100 100"` <br>
or <br>
`imagineer -i in.png -o out.png --sampling-filter triangle --resize 100 100`

**rotate 90 degree** example: <br>
`imagineer -i in.png -o out.png --apply-operations "rotate90"` <br>
or <br>
`imagineer -i in.png -o out.png --rotate90`

**rotate 180 degree** example: <br>
`imagineer -i in.png -o out.png --apply-operations "rotate180"` <br>
or <br>
`imagineer -i in.png -o out.png --rotate180`

**rotate 270 degree** example: <br>
`imagineer -i in.png -o out.png --apply-operations "rotate270"` <br>
or <br>
`imagineer -i in.png -o out.png --rotate270`

**threshold** example: <br>
`imagineer -i in.png -o out.png --apply-operations "threshold"` <br>
or <br>
`imagineer -i in.png -o out.png --threshold`

**unsharpen** example: <br>
`imagineer -i in.png -o out.png --apply-operations "unsharpen -0.7 1"` <br>
or <br>
`imagineer -i in.png -o out.png --unsharpen -0.7 1`

**vertical gradient** example: <br>
`imagineer -i in.png -o out.png --apply-operations "vertical-gradient rgba(255, 0, 0, 255) rgba(0, 0, 255, 255)"` <br>
or <br>
`imagineer -i in.png -o out.png --vertical-gradient "rgba(255, 0, 0, 255)" "rgba(0, 0, 255, 255)"`


example with *multiple* image operations which are applied from left-to-right: <br>
`imagineer -i in.png -o out.png --apply-operations "rotate180; flip-horizontal; set sampling-filter nearest; resize 75 80; hue-rotate 75"` <br>
or <br>
`imagineer -i in.png -o out.png --rotate180 --flip-horizontal --sampling-filter nearest --resize 75 80 --hue-rotate 75`


<br>

**Other resources on image operations**

For additional information on available options and flags, run `imagineer --help`.

### License
 
Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

### Suggestions, Questions, Bugs

Feel free to open an issue :mailbox_with_mail: if you have a suggestion, a question or found a bug =).

🎸 🎺 🎻 🎷
