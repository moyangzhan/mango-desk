use crate::errors::AppError;
use crate::global::SUPPORTED_AUDIO_EXTS;
use crate::utils::base64_util::file_to_data_uri;
use anyhow::{Result, anyhow};
use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::get_codecs;
use symphonia::default::get_probe;

pub fn is_supported_audio_ext(ext: &str) -> bool {
    SUPPORTED_AUDIO_EXTS.contains(&ext)
}

/// Determine if a file is an audio file
/// Returns Ok(true) for an audio, Ok(false) for non-audio, and Err for file operation failure
pub fn is_supported_audio_file(path: &str) -> Result<bool, AppError> {
    if let Some(kind) = infer::get_from_path(path)? {
        let ext = kind.extension();
        if SUPPORTED_AUDIO_EXTS.contains(&ext) {
            return Ok(true);
        }
    }
    // No audio format matched
    Ok(false)
}

/// Convert audio file to Data URI
pub fn audio_to_data_uri(path: &str) -> Result<String, AppError> {
    let supported = is_supported_audio_file(path)?;
    if !supported {
        return Err(AppError::UnsupportedAudioFormat(path.to_string()));
    }
    let data_uri = file_to_data_uri(path)?;
    Ok(data_uri)
}

/// Load audio file and convert to f32 samples at 16kHz for whisper.cpp
pub fn load_audio_to_f32(path: &str) -> Result<Vec<f32>> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = get_probe().format(
        &Default::default(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;

    let track = format.default_track().ok_or(anyhow!("no audio track"))?;
    let source_rate = track
        .codec_params
        .sample_rate
        .ok_or(anyhow!("Unknown rate"))?;
    let target_rate = 16000;
    let resample_ratio = source_rate as f32 / target_rate as f32;
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut audio_samples: Vec<f32> = Vec::new();
    let mut next_sample_pos: f32 = 0.0;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };
        let decoded = decoder.decode(&packet)?;
        let mut current_frame = Vec::new();

        match decoded {
            AudioBufferRef::F32(buf) => {
                for i in 0..buf.frames() {
                    let sum: f32 = (0..buf.spec().channels.count())
                        .map(|ch| buf.chan(ch)[i])
                        .sum();
                    current_frame.push(sum / buf.spec().channels.count() as f32);
                }
            }
            _ => {
                let spec = decoded.spec();
                let channels = spec.channels.count();
                let mut sbuf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                sbuf.copy_interleaved_ref(decoded);
                let samples = sbuf.samples();
                for i in 0..(samples.len() / channels) {
                    let sum: f32 = (0..channels).map(|ch| samples[i * channels + ch]).sum();
                    current_frame.push(sum / channels as f32);
                }
            }
        }

        // Resample to 16kHz
        let frame_data = current_frame;
        let frame_len = frame_data.len() as f32;
        let mut i = next_sample_pos;

        while i < frame_len {
            let idx = i as usize;
            let next_idx = (idx + 1).min((frame_len - 1.0) as usize);
            let frac = i - idx as f32;
            let interpolated_sample = frame_data[idx] * (1.0 - frac) + frame_data[next_idx] * frac;
            audio_samples.push(interpolated_sample);
            i += resample_ratio;
        }
        next_sample_pos = i - frame_len;
    }

    Ok(audio_samples)
}
