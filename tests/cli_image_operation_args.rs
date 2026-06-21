#![deny(clippy::all)]

use std::process::Child;

#[macro_use]
pub mod common;

// The following integration test modules currently only check whether the processes they start exit
// successfully.

#[cfg(test)]
fn command(input: &str, output: &str, args: &str) -> Child {
    common::SicTestCommandBuilder::new()
        .input_from_resources(input)
        .output_in_target(output)
        .with_args(args.split(' '))
        .spawn_child()
}

#[cfg(test)]
mod blur {
    use crate::common::*;

    //    FIXME(image-rs/image#983): blur panics on option unwrap within image::imageops::sample::vertical_sample.
    //    #[test]
    //    fn blur_0() {
    //        let mut process = command(DEFAULT_IN, "img_op_arg.png", "--blur 0");
    //        let result = process.wait();
    //        assert!(result.is_ok());
    //        assert!(result.unwrap().success());
    //    }

    #[test]
    fn blur_1() {
        let mut process = SicTestCommandBuilder::new()
            .input_from_resources(DEFAULT_IN)
            .output_in_target("img_op_arg.png")
            .with_args(["--blur", "1"])
            .spawn_child();

        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn blur_1_dot_1() {
        let mut process = SicTestCommandBuilder::new()
            .input_from_resources(DEFAULT_IN)
            .output_in_target("img_op_arg.png")
            .with_args(["--blur", "1.1"])
            .spawn_child();

        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn blur_neg_1_dot_1() {
        let mut process = SicTestCommandBuilder::new()
            .input_from_resources(DEFAULT_IN)
            .output_in_target("img_op_arg.png")
            .with_args(["--blur", "-1.1"])
            .spawn_child();

        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod crop {
    use super::*;
    use crate::common::*;

    #[test]
    fn crop_simple() {
        let mut process = SicTestCommandBuilder::new()
            .input_from_resources(DEFAULT_IN)
            .output_in_target("img_op_arg.png")
            .with_args(["--crop", "0", "0", "1", "1"])
            .spawn_child();

        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn dont_allow_separated_values() {
        let mut process = command(DEFAULT_IN, "cio_crop2.png", "--crop 0 0 --crop 1 1");
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }

    #[test]
    fn incorrect_amount_of_values() {
        let mut process = command(DEFAULT_IN, "cio_crop3.png", "--crop 0 0 1");
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }

    #[test]
    fn crop_multiple_ok() {
        let mut process = command(DEFAULT_IN, "cio_crop4.png", "--crop 2 2 3 3 --crop 0 0 1 1");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn crop_multiple_one_empty() {
        let mut process = command(
            DEFAULT_IN,
            "cio_crop5.png",
            "--crop --crop 2 2 3 3 --crop 0 0 1 1",
        );
        let result = process.wait();
        assert!(result.is_ok());

        // Here we would like to assert_not! instead
        assert_not!(result.unwrap().success());
    }

    // This one however will fail, as we tell Clap we require 4 values for this cli argument.
    #[test]
    fn crop_empty() {
        let mut process = command(DEFAULT_IN, "cio_crop6.png", "--crop");
        let result = process.wait();
        assert!(result.is_ok());

        assert_not!(result.unwrap().success());
    }

    #[test]
    fn crop_too_few() {
        let mut process = command(DEFAULT_IN, "cio_crop7.png", "--crop 0 0 2");
        let result = process.wait();
        assert!(result.is_ok());

        assert_not!(result.unwrap().success());
    }

    #[test]
    fn crop_too_many() {
        let mut process = command(DEFAULT_IN, "cio_crop8.png", "--crop 0 0 2 2 2");
        let result = process.wait();
        assert!(result.is_ok());

        assert_not!(result.unwrap().success());
    }

    #[test]
    fn crop_invalid_selection() {
        let mut process = command(DEFAULT_IN, "cio_crop9.png", "--crop 1 1 1 1");
        let result = process.wait();
        assert!(result.is_ok());

        assert_not!(result.unwrap().success());
    }

    #[test]
    fn crop_out_of_bounds_selection() {
        let mut process = command(DEFAULT_IN, "cio_crop9.png", "--crop 1 1 10 10");
        let result = process.wait();
        assert!(result.is_ok());

        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod diff {
    use super::*;
    use crate::common::*;

    #[test]
    fn diff() {
        let mut process = command(
            "2x3_wrabaa.png",
            "cio_diff_1.png",
            &[
                "--diff",
                setup_input_path("3x2_wbaaba.png").to_str().unwrap(),
            ]
            .join(" "),
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn diff_invalid_path() {
        let mut process = command(
            "2x3_wrabaa.png",
            "cio_diff_2.png",
            &["--diff", setup_input_path("0:1.png").to_str().unwrap()].join(" "),
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod filter3x3 {
    use super::*;
    use crate::common::*;

    #[test]
    fn filter3x3() {
        let mut process = command(
            DEFAULT_IN,
            "cio_f3x3_1.png",
            "--filter3x3 1 2 3 4 5 -6 -7 -8 -9.555",
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn filter3x3_emboss() {
        let mut process = command(
            DEFAULT_IN,
            "cio_f3x3_emboss.png",
            "--filter3x3 -1 -1 0 -1 1 1 0 1 1",
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn filter3x3_not() {
        let mut process = command(
            DEFAULT_IN,
            "cio_f3x3_2.png",
            "--filter3x3 1 2 3 4 p -6 -7 -8 -9.555",
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod fliph {
    use super::*;
    use crate::common::*;

    #[test]
    fn fliph() {
        let mut process = command(DEFAULT_IN, "cio_fliph.png", "--flip-horizontal");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod flipv {
    use super::*;
    use crate::common::*;

    #[test]
    fn flipv() {
        let mut process = command(DEFAULT_IN, "cio_flipv.png", "--flip-vertical");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod grayscale {
    use super::*;
    use crate::common::*;

    #[test]
    fn grayscale() {
        let mut process = command(DEFAULT_IN, "cio_gs.png", "--grayscale");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod hue_rotate {
    use super::*;
    use crate::common::*;

    #[test]
    fn hue_rotate() {
        let mut process = command(DEFAULT_IN, "cio_hr.png", "--hue-rotate -90");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn hue_rotate_not() {
        let mut process = command(DEFAULT_IN, "cio_hr2.png", "--hue-rotate -p");
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod invert {
    use super::*;
    use crate::common::*;

    #[test]
    fn invert() {
        let mut process = command(DEFAULT_IN, "cio_invert.png", "--invert");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod overlay {
    use super::*;
    use crate::common::*;

    #[test]
    fn overlay() {
        let mut process = command(
            DEFAULT_IN,
            "cio_overlay.png",
            &[
                "--overlay",
                setup_input_path("3x2_wbaaba.png").to_str().unwrap(),
                "0",
                "0",
            ]
            .join(" "),
        );
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod resize {
    use super::*;
    use crate::common::*;

    #[test]
    fn resize() {
        let mut process = command(DEFAULT_IN, "cio_resize1.png", "--resize 10 10");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn resize_not() {
        let mut process = command(DEFAULT_IN, "cio_resize2.png", "--resize 10 p");
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod rotate90 {
    use super::*;
    use crate::common::*;

    #[test]
    fn rotate90() {
        let mut process = command(DEFAULT_IN, "cio_rot90.png", "--rotate90");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod rotate180 {
    use super::*;
    use crate::common::*;

    #[test]
    fn rotate180() {
        let mut process = command(DEFAULT_IN, "cio_rot180.png", "--rotate180");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod rotate270 {
    use super::*;
    use crate::common::*;

    #[test]
    fn rotate270() {
        let mut process = command(DEFAULT_IN, "cio_rot270.png", "--rotate270");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}

#[cfg(test)]
mod unsharpen {
    use super::*;
    use crate::common::*;

    #[test]
    fn unsharpen() {
        let mut process = command(DEFAULT_IN, "cio_resize1.png", "--unsharpen 1.5 1");
        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn unsharpen_not() {
        let mut process = command(DEFAULT_IN, "cio_resize2.png", "--unsharpen 1.5 1.0");
        let result = process.wait();
        assert!(result.is_ok());
        assert_not!(result.unwrap().success());
    }
}

#[cfg(test)]
mod mixed {
    use super::*;
    use crate::common::*;

    #[test]
    fn all_regular_operations() {
        let mut process = command(
            DEFAULT_IN,
            "img_op_arg_mixed_all.png",
            "--blur 1 \
             --brighten 2 \
             --contrast 3 \
             --crop 0 0 2 2 \
             --filter3x3 0 1 2 3 4 5 6 7 8 \
             --flip-horizontal \
             --flip-vertical \
             --grayscale \
             --hue-rotate -90 \
             --invert \
             --resize 10 10 \
             --rotate90 \
             --rotate180 \
             --rotate270 \
             --unsharpen 1.5 1",
        );

        let result = process.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }
}
