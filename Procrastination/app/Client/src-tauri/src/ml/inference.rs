use std::path::PathBuf;
use ndarray::Array2;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::{TensorRef};
use crate::models::table_structs::FeatureVectors;

pub fn load_model(resource_path: PathBuf) -> Result<Session, ort::Error> {
    let model = Session::builder()?.with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(4)?.commit_from_file(resource_path)?;

    Ok(model)
}


pub fn run_inference(session: &mut Session, features: &FeatureVectors) -> Result<(String, f64), ort::Error> {

    let feature_vec = vec![
        features.typing_speed as f32,
        features.repetitive_key_ratio as f32,
        features.mouse_velocity as f32,
        features.idle_ratio as f32,
        features.window_switch_frequency as f32,
        features.scroll_velocity as f32,
    ];
    debug_assert_eq!(feature_vec.len(), 6, "Feature vector length mismatch");
    let input_data = Array2::from_shape_vec((1, 6), feature_vec)
        .expect("Feature vector length must be exactly 6");

    let outputs = session.run(ort::inputs!["float_input" => TensorRef::from_array_view(input_data.view())?])?;

    // extractintg predicted label integer
    let (_shape, label_data) = outputs["label"].try_extract_tensor::<i64>()?;
    let predicted_int = label_data[0];

    // map integer to label string
    let label = match predicted_int {
        0 => "Focused",
        1 => "At Risk",
        2 => "Procrastinating",
        3 => "Idle",
        _ => "Unknown"
    }.to_string();
    
    //extract confidence from probability output
    let (_, prob_data) = outputs["probabilities"].try_extract_tensor::<f32>()?;

    // find  probability og predicted class
    let confidence = prob_data[predicted_int as usize] as f64;

    Ok((label, confidence))
}