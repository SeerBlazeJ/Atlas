use sherpa_onnx::{
    OnlineRecognizer, OnlineRecognizerConfig, OnlineStream, OnlineTransducerModelConfig,
};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

// 1. Shared State Definition
pub struct AppState {
    pub recognizer: OnlineRecognizer,
    pub stream: Mutex<Option<OnlineStream>>,
}

// Initialize recognizer during app startup
pub fn init_state() -> AppState {
    let model_dir = "./sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8";
    let mut config = OnlineRecognizerConfig::default();

    config.model_config.transducer = OnlineTransducerModelConfig {
        encoder: Some(format!("{}/encoder.int8.onnx", model_dir)),
        decoder: Some(format!("{}/decoder.int8.onnx", model_dir)),
        joiner: Some(format!("{}/joiner.int8.onnx", model_dir)),
        ..Default::default()
    };
    config.model_config.tokens = Some(format!("{}/tokens.txt", model_dir));
    config.decoding_method = Some("greedy_search".into());
    config.enable_endpoint = true;

    let recognizer = OnlineRecognizer::create(&config).expect("failed to create recognizer");
    AppState {
        recognizer,
        stream: Mutex::new(None),
    }
}

// 2. Tauri Commands

#[tauri::command]
pub fn start_stream(state: State<'_, AppState>) -> Result<(), String> {
    let mut stream_guard = state.stream.lock().map_err(|e| e.to_string())?;
    let stream = state.recognizer.create_stream();
    *stream_guard = Some(stream);
    Ok(())
}

#[tauri::command]
pub fn send_audio_chunk(
    app: AppHandle,
    state: State<'_, AppState>,
    sample_rate: u32,
    samples: Vec<f32>,
) -> Result<(), String> {
    let mut stream_guard = state.stream.lock().map_err(|e| e.to_string())?;

    if let Some(stream) = stream_guard.as_mut() {
        // Accept audio coming from frontend microphone
        stream.accept_waveform(sample_rate as i32, &samples);

        // Decode available frames
        while state.recognizer.is_ready(stream) {
            state.recognizer.decode(stream);
        }

        // Send intermediate transcriptions back to frontend asynchronously
        let result = state.recognizer.get_result(stream).unwrap();
        if !result.text.is_empty() {
            app.emit("stt-partial-result", result.text)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>) -> Result<String, String> {
    // 1. Scope the lock so it drops before the .await
    let final_text = {
        let mut stream_guard = state.stream.lock().map_err(|e| e.to_string())?;

        if let Some(stream) = stream_guard.take() {
            stream.input_finished();
            while state.recognizer.is_ready(&stream) {
                state.recognizer.decode(&stream);
            }
            state.recognizer.get_result(&stream).unwrap().text
        } else {
            String::new()
        }
    }; // <-- stream_guard is automatically dropped here!

    // 2. NOW we can safely await downstream processes
    if !final_text.is_empty() {
        process_transcript(&final_text).await?;
    }
    dbg!(&final_text);
    Ok(final_text)
}

// 3. Downstream Function Call
async fn process_transcript(text: &str) -> Result<(), String> {
    println!("Processing finished transcript: {}", text);
    // Call LLM, database, or next pipeline logic here
    Ok(())
}
