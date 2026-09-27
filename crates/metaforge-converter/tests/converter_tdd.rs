use metaforge_converter::{
    convert_audio_wav_bytes, convert_image_bytes, probe_media_file, AudioConvertOptions,
    ImageConvertOptions, ImageTargetFormat,
};
use std::io::Cursor;
use tempfile::NamedTempFile;

fn create_synthetic_png() -> Vec<u8> {
    let img = image::RgbImage::from_fn(64, 64, |x, y| {
        image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
    });
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("write synthetic png");
    buf
}

fn create_synthetic_wav(channels: u16, sample_rate: u32, num_samples: usize) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Vec::new();
    let mut writer = hound::WavWriter::new(Cursor::new(&mut buf), spec).expect("create wav writer");
    for i in 0..num_samples {
        let sample = ((i as f64 * 0.1).sin() * 10000.0) as i16;
        writer.write_sample(sample).expect("write sample");
        if channels == 2 {
            writer.write_sample(sample / 2).expect("write sample ch2");
        }
    }
    writer.finalize().expect("finalize wav");
    buf
}

#[test]
fn test_png_to_jpeg_conversion() {
    let png = create_synthetic_png();
    let opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Jpeg,
        quality: Some(90),
        resize: None,
        preserve_metadata: true,
        max_dimension: 4096,
    };

    let jpeg = convert_image_bytes(&png, &opts).expect("convert png to jpeg");
    assert!(jpeg.len() > 10);
    assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]); // JPEG SOI
}

#[test]
fn test_png_to_webp_and_bmp_conversion() {
    let png = create_synthetic_png();

    let webp_opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Webp,
        ..Default::default()
    };
    let webp = convert_image_bytes(&png, &webp_opts).expect("convert to webp");
    assert_eq!(&webp[0..4], b"RIFF");
    assert_eq!(&webp[8..12], b"WEBP");

    let bmp_opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Bmp,
        ..Default::default()
    };
    let bmp = convert_image_bytes(&png, &bmp_opts).expect("convert to bmp");
    assert_eq!(&bmp[0..2], b"BM");
}

#[test]
fn test_image_resize_during_conversion() {
    let png = create_synthetic_png();
    let opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Png,
        resize: Some((32, 16)),
        ..Default::default()
    };

    let resized_bytes = convert_image_bytes(&png, &opts).expect("resize png");
    let decoded = image::load_from_memory(&resized_bytes).expect("decode resized png");
    assert_eq!(decoded.width(), 32);
    assert_eq!(decoded.height(), 16);
}

#[test]
fn test_audio_stereo_to_mono_downmix() {
    let stereo_wav = create_synthetic_wav(2, 44100, 1000);
    let opts = AudioConvertOptions {
        target_channels: Some(1),
        target_sample_rate: None,
        target_bits_per_sample: None,
    };

    let mono_wav = convert_audio_wav_bytes(&stereo_wav, &opts).expect("downmix audio");
    let reader = hound::WavReader::new(Cursor::new(mono_wav)).expect("read mono wav");
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.spec().sample_rate, 44100);
}

#[test]
fn test_audio_resampling() {
    let wav = create_synthetic_wav(1, 44100, 22050); // 0.5s at 44.1kHz
    let opts = AudioConvertOptions {
        target_channels: None,
        target_sample_rate: Some(22050),
        target_bits_per_sample: None,
    };

    let resampled_wav = convert_audio_wav_bytes(&wav, &opts).expect("resample audio");
    let reader = hound::WavReader::new(Cursor::new(resampled_wav)).expect("read resampled wav");
    assert_eq!(reader.spec().sample_rate, 22050);
}

#[test]
fn test_media_probing() {
    let png_bytes = create_synthetic_png();
    let mut temp = NamedTempFile::new().expect("create temp");
    std::io::Write::write_all(&mut temp, &png_bytes).expect("write temp png");

    let probe = probe_media_file(temp.path()).expect("probe media file");
    assert_eq!(probe.media_kind, "Image");
    assert_eq!(probe.dimensions, Some((64, 64)));

    let wav_bytes = create_synthetic_wav(2, 48000, 4800);
    let mut wav_temp = NamedTempFile::new().expect("create temp wav");
    std::io::Write::write_all(&mut wav_temp, &wav_bytes).expect("write temp wav");

    let wav_probe = probe_media_file(wav_temp.path()).expect("probe wav file");
    assert_eq!(wav_probe.media_kind, "Audio");
    assert_eq!(wav_probe.sample_rate, Some(48000));
    assert_eq!(wav_probe.channels, Some(2));
}
