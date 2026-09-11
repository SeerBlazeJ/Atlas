use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig,
    OfflineTtsPocketModelConfig, Wave,
};
use std::sync::mpsc;
use std::thread;

pub struct VoiceModule {
    tx: mpsc::Sender<String>,
    buffer: String,
}

impl VoiceModule {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            // Initialize Rodio for continuous background playback
            let (_stream, stream_handle) =
                OutputStream::try_default().expect("Failed to get audio output device");
            let sink = Sink::try_new(&stream_handle).expect("Failed to create rodio sink");

            // Point this directly to the extracted pocket-tts folder
            let config = OfflineTtsConfig {
                model: OfflineTtsModelConfig {
                    num_threads: 2,
                    pocket: OfflineTtsPocketModelConfig {
                        encoder: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/encoder.onnx".into(),
                        ),
                        decoder: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/decoder.int8.onnx".into(),
                        ),
                        lm_flow: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/lm_flow.int8.onnx".into(),
                        ),
                        lm_main: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/lm_main.int8.onnx".into(),
                        ),
                        text_conditioner: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/text_conditioner.onnx".into(),
                        ),

                        // Fix the field names here:
                        vocab_json: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/vocab.json".into(),
                        ),
                        token_scores_json: Some(
                            "./sherpa-onnx-pocket-tts-int8-2026-01-26/token_scores.json".into(),
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            // Option<OfflineTts> is returned, so .expect() properly unwraps it
            let tts = OfflineTts::create(&config).expect("Failed to create Sherpa TTS instance");
            let wave = Wave::read("./sherpa-onnx-pocket-tts-int8-2026-01-26/test_wavs/bria.wav")
                .expect("Failed to read reference audio wave file");
            // Sherpa requires this struct for generation parameters
            let gen_config = GenerationConfig {
                sid: 0,
                speed: 1.0,
                reference_audio: Some(wave.samples().to_owned()),
                reference_sample_rate: wave.sample_rate(),
                ..Default::default()
            };

            // Receive sentences, synthesize, and queue them seamlessly
            while let Ok(text) = rx.recv() {
                if let Some(generated_audio) =
                    tts.generate_with_config(&text, &gen_config, None::<fn(&[f32], f32) -> bool>)
                {
                    let sample_rate = generated_audio.sample_rate() as u32;
                    let audio_data = generated_audio.samples().to_vec();

                    let buffer = SamplesBuffer::new(1, sample_rate, audio_data);
                    sink.append(buffer);
                }
            }

            sink.sleep_until_end();
        });

        Ok(Self {
            tx,
            buffer: String::new(),
        })
    }

    pub fn push_text(&mut self, text: &str) {
        self.buffer.push_str(text);

        if let Some(pos) = self
            .buffer
            .rfind(|c: char| c == '.' || c == '?' || c == '!' || c == '\n')
        {
            let sentence = self.buffer[..=pos].to_string();
            self.buffer = self.buffer[pos + 1..].to_string();

            let clean_sentence = sentence.trim();
            if !clean_sentence.is_empty() {
                let _ = self.tx.send(clean_sentence.to_string());
            }
        }
    }

    pub fn flush(&mut self) {
        let clean = self.buffer.trim();
        if !clean.is_empty() {
            let _ = self.tx.send(clean.to_string());
        }
        self.buffer.clear();
    }
}
