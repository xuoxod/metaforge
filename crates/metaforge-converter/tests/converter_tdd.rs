use metaforge_converter::{
    convert_audio_wav_bytes, convert_image_bytes, convert_stream, execute_batch_convert,
    probe_media_file, AudioConvertOptions, AudioPipeline, BatchConvertOptions, ImageConvertOptions,
    ImagePipeline, ImageTargetFormat,
};
use std::fs;
use std::io::Cursor;
use tempfile::{tempdir, NamedTempFile};

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
    assert!(webp.starts_with(b"RIFF"));

    let bmp_opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Bmp,
        ..Default::default()
    };
    let bmp = convert_image_bytes(&png, &bmp_opts).expect("convert to bmp");
    assert!(bmp.starts_with(b"BM"));
}

#[test]
fn test_image_resize_during_conversion() {
    let png = create_synthetic_png();
    let opts = ImageConvertOptions {
        target_format: ImageTargetFormat::Png,
        resize: Some((32, 16)),
        ..Default::default()
    };

    let resized = convert_image_bytes(&png, &opts).expect("resize png");
    let decoded = image::load_from_memory(&resized).expect("load resized png");
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
        ..Default::default()
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
        ..Default::default()
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

#[test]
fn test_chainable_audio_pipeline() {
    let wav_bytes = create_synthetic_wav(2, 48000, 2400);

    let pipeline = AudioPipeline::new()
        .sample_rate(16000)
        .channels(1)
        .gain(1.5)
        .normalize(true);

    let out_bytes = pipeline.process_bytes(&wav_bytes, Some("wav")).expect("pipeline transcode");
    let reader = hound::WavReader::new(Cursor::new(out_bytes)).expect("read wav");
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.spec().sample_rate, 16000);
}

#[test]
fn test_chainable_image_pipeline() {
    let png_bytes = create_synthetic_png();

    let pipeline = ImagePipeline::new()
        .target(ImageTargetFormat::Webp)
        .quality(95)
        .resize(32, 32);

    let out_bytes = pipeline.process_bytes(&png_bytes).expect("pipeline transcode");
    assert!(out_bytes.starts_with(b"RIFF"));

    let decoded = image::load_from_memory(&out_bytes).expect("decode webp");
    assert_eq!(decoded.width(), 32);
    assert_eq!(decoded.height(), 32);
}

#[test]
fn test_batch_conversion_recursive_tree_and_dry_run() {
    let in_dir = tempdir().expect("create in dir");
    let sub_dir = in_dir.path().join("subfolder");
    fs::create_dir_all(&sub_dir).expect("create sub dir");

    // Populate test files
    let png = create_synthetic_png();
    let wav = create_synthetic_wav(1, 44100, 1000);

    fs::write(in_dir.path().join("photo1.png"), &png).expect("write photo1");
    fs::write(sub_dir.join("photo2.png"), &png).expect("write photo2");
    fs::write(sub_dir.join("audio.wav"), &wav).expect("write audio");

    let out_dir = tempdir().expect("create out dir");

    // 1. Test Dry Run
    let dry_options = BatchConvertOptions {
        input_dir: in_dir.path().to_path_buf(),
        output_dir: out_dir.path().to_path_buf(),
        extensions: vec!["png".to_string()],
        target_format: "webp".to_string(),
        max_workers: 4,
        dry_run: true,
        flatten: false,
        ..Default::default()
    };

    let dry_report = execute_batch_convert(&dry_options).expect("execute dry run");
    assert!(dry_report.dry_run);
    assert_eq!(dry_report.total_scanned, 2); // 2 PNG files
    assert_eq!(dry_report.items.len(), 2);
    // Out directory must NOT have been populated
    assert!(!out_dir.path().join("photo1.webp").exists());

    // 2. Test Real Batch Conversion (with tree preservation)
    let real_options = BatchConvertOptions {
        input_dir: in_dir.path().to_path_buf(),
        output_dir: out_dir.path().to_path_buf(),
        extensions: vec!["png".to_string()],
        target_format: "webp".to_string(),
        max_workers: 4,
        dry_run: false,
        flatten: false,
        ..Default::default()
    };

    let real_report = execute_batch_convert(&real_options).expect("execute real batch");
    assert!(!real_report.dry_run);
    assert_eq!(real_report.converted, 2);
    assert_eq!(real_report.failed, 0);

    assert!(out_dir.path().join("photo1.webp").exists());
    assert!(out_dir.path().join("subfolder/photo2.webp").exists());
}

#[test]
fn test_convert_stream_pipes() {
    let png = create_synthetic_png();
    let mut output = Vec::new();

    let report = convert_stream(
        Cursor::new(png),
        &mut output,
        Some("png"),
        "jpeg",
        &ImageConvertOptions::default(),
        &AudioConvertOptions::default(),
    )
    .expect("stream conversion");

    assert_eq!(report.input_path, "<stdin>");
    assert_eq!(report.output_path, "<stdout>");
    assert!(output.starts_with(&[0xFF, 0xD8])); // JPEG
}
