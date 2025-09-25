use sic_core::image::codecs::pnm::{PnmSubtype, SampleEncoding};

// NB: Is now unused. Just as example for RUST-14319.
pub fn pnm_subtype(sample_encoding: SampleEncoding) -> PnmSubtype {
    let example = "pam";

    match example {
        "pam" => PnmSubtype::ArbitraryMap,
        _ => PnmSubtype::Pixmap(sample_encoding),
    }
}
