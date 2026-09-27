use metaforge_converter::{
    convert_audio_wav_bytes, convert_image_bytes, AudioConvertOptions, ImageConvertOptions,
    ImageTargetFormat,
};
use metaforge_core::error::MetaForgeError;
use std::io::Cursor;

fn create_valid_png() -> Vec<u8> {
    let img = image::RgbImage::from_fn(16, 16, |_, _| image::Rgb([100, 150, 200]));
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("write synthetic png");
    buf
}

#[test]
fn test_adversarial_dimension_allocation_bomb() {
    let png = create_valid_png();

    // Adversarial resize attempt requesting 100,000 x 100,000 (10 gigapixels DoS attempt)
    let attack_opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Jpeg,
        resize: Some((100_000, 100_000)),
        max_dimension: 16384,
        ..Default::default()
    };

    let result = convert_image_bytes(&png, &attack_opts);
    assert!(result.is_err(), "Must reject allocation bomb dimensions");
    match result.err().unwrap() {
        MetaForgeError::DimensionAllocationBomb { width, height } => {
            assert_eq!(width, 100_000);
            assert_eq!(height, 100_000);
        }
        other => panic!("Expected DimensionAllocationBomb, got {:?}", other),
    }
}

#[test]
fn test_adversarial_corrupted_truncated_image_stream() {
    // Random binary garbage simulating truncated or weaponized image payload
    let garbage = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xDE, 0xAD, 0xBE, 0xEF];
    let opts = ImageConvertOptions::default();

    let result = convert_image_bytes(&garbage, &opts);
    assert!(result.is_err(), "Must reject corrupted image stream gracefully");
    assert!(matches!(result.err().unwrap(), MetaForgeError::ConversionError { .. }));
}

#[test]
fn test_adversarial_empty_buffer_rejection() {
    let empty: Vec<u8> = Vec::new();
    let img_res = convert_image_bytes(&empty, &ImageConvertOptions::default());
    assert!(matches!(img_res.err().unwrap(), MetaForgeError::EmptyFile(_)));

    let audio_res = convert_audio_wav_bytes(&empty, &AudioConvertOptions::default());
    assert!(matches!(audio_res.err().unwrap(), MetaForgeError::EmptyFile(_)));
}

#[test]
fn test_adversarial_audio_invalid_channel_counts() {
    // Valid minimal WAV
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Vec::new();
    let mut writer = hound::WavWriter::new(Cursor::new(&mut buf), spec).unwrap();
    writer.write_sample(0i16).unwrap();
    writer.finalize().unwrap();

    // Adversarial channel request: 0 channels
    let opts_zero = AudioConvertOptions {
        target_channels: Some(0),
        target_sample_rate: None,
        target_bits_per_sample: None,
    };
    assert!(convert_audio_wav_bytes(&buf, &opts_zero).is_err());

    // Adversarial channel request: 99 channels (exhaustion attack)
    let opts_huge = AudioConvertOptions {
        target_channels: Some(99),
        target_sample_rate: None,
        target_bits_per_sample: None,
    };
    assert!(convert_audio_wav_bytes(&buf, &opts_huge).is_err());
}
