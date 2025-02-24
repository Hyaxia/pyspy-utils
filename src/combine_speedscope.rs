#[derive(serde::Serialize, Debug, Deserialize, Clone)]
struct Shared {
    frames: Vec<Frame>,
}

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Profile {
    name: String,
    unit: String,
    startValue: f64,
    endValue: f64,
    samples: Vec<Vec<i32>>,
    weights: Vec<f64>,
    r#type: String,
}

use serde::Deserialize;

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
pub struct Speedscope {
    profiles: Vec<Profile>,
    shared: Shared,
    #[serde(rename = "$schema")]
    schema: String,
    // null value
    exporter: String,
    name: String,
}

#[derive(serde::Serialize, Debug, Deserialize, Clone)]
struct Frame {
    name: String,
    file: String,
    line: u32,
    col: Option<u32>,
}

use std::{collections::HashMap, fs::File, hash::Hash, io::BufReader, ptr::null};

pub fn read_speedscope_files(paths: Vec<&str>) -> Vec<Speedscope> {
    // let mut frames = Vec::new();
    let speedscopes = paths
        .iter()
        .map(|path| {
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        })
        .collect();
    return speedscopes;
}

fn hash_frame(frame: &Frame) -> String {
    format!(
        "{}:{}:{}:{}",
        frame.name,
        frame.file,
        frame.line,
        frame.col.unwrap_or(0)
    )
}

// this function returns a hashmap that is global for all speedscope files
fn create_hash_to_new_index_and_frame(
    speedscopes: &Vec<Speedscope>,
) -> HashMap<String, (usize, &Frame)> {
    let mut hash_to_new_index_and_frame = HashMap::new();
    let mut index = 0;
    for speedscope in speedscopes {
        speedscope.shared.frames.iter().for_each(|frame| {
            let hash = hash_frame(&frame);
            if !hash_to_new_index_and_frame.contains_key(&hash) {
                hash_to_new_index_and_frame.insert(hash, (index.clone(), frame));
                index += 1;
            }
        });
    }
    return hash_to_new_index_and_frame;
}

fn combine_profiles_weights(profiles: &[&Profile]) -> Vec<f64> {
    let mut combined_weights = Vec::new();
    profiles.iter().for_each(|profile| {
        let mut new_weights = Vec::new();

        profile.weights.iter().for_each(|weight| {
            new_weights.push(*weight);
        });
        combined_weights.extend(new_weights);
    });
    return combined_weights;
}

fn get_frame_by_index(frames: &Vec<Frame>, index: usize) -> Frame {
    return frames[index].clone();
}

pub fn extract_same_profile_groups(
    speedscopes: &Vec<Speedscope>,
) -> HashMap<String, Vec<&Profile>> {
    return speedscopes
        .iter()
        .fold(HashMap::new(), |mut acc, speedscope| {
            speedscope.profiles.iter().for_each(|profile| {
                let name = profile.name.clone();
                if !acc.contains_key(&name) {
                    acc.insert(name.clone(), vec![profile]);
                } else {
                    let profiles = acc.get_mut(&name).unwrap();
                    profiles.push(profile);
                }
            });
            return acc;
        });
}

fn adjust_samples_from_current_frame_indexes_to_new_indexes(
    profiles: &[&Profile],
    hash_to_new_index_and_frame: HashMap<String, (usize, &Frame)>,
    frames: &Vec<Frame>,
) -> Vec<Vec<i32>> {
    let mut combined_samples = Vec::new();
    profiles.iter().for_each(|profile| {
        profile.samples.iter().for_each(|sample| {
            let mut new_sample = Vec::new();
            sample.iter().for_each(|frame_index| {
                let frame = get_frame_by_index(frames, *frame_index as usize);
                let hash = hash_frame(&frame);
                let new_index = hash_to_new_index_and_frame.get(&hash).unwrap();
                new_sample.push(new_index.0 as i32);
            });
            combined_samples.push(new_sample);
        });
    });
    return combined_samples;
}

fn combine_samples_from_multiple_profiles(profiles: &[&Profile]) -> Vec<Vec<i32>> {
    let mut combined_samples = Vec::new();
    profiles.iter().for_each(|profile| {
        profile.samples.iter().for_each(|sample| {
            combined_samples.push(sample.clone());
        });
    });
    return combined_samples;
}


fn calculate_end_value(profiles: &[&Profile]) -> f64 {
    let mut end_value = 0.0;
    profiles.iter().for_each(|profile| {
        end_value += profile.endValue;
    });
    return end_value;
}

fn adjust_speedscopes_to_new_index(
    speedscopes: &[Speedscope],
    hash_to_new_index_and_frame: &HashMap<String, (usize, &Frame)>,
) -> Vec<Speedscope> {
    let mut new_speedscopes = speedscopes.to_vec();
    for speedscope in new_speedscopes.iter_mut() {
        for profile in speedscope.profiles.iter_mut() {
            let adjusted_samples = adjust_samples_from_current_frame_indexes_to_new_indexes(
                &[profile],
                hash_to_new_index_and_frame.clone(),
                &speedscope.shared.frames,
            );
            profile.samples = adjusted_samples;
        }
    }
    new_speedscopes
}

pub fn entry_point(all_profiles_path: &str, combined_speedscope_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let collected_files_str = std::fs::read_to_string(all_profiles_path)?;
    let collected_files_strings = collected_files_str.lines().collect();
    let speedscopes = read_speedscope_files(collected_files_strings);
    let combined_speedscope = combine_speedscope_files(speedscopes);
    // write the combined speedscope to a file
    std::fs::write(
        combined_speedscope_path,
        serde_json::to_string(&combined_speedscope)?,
    )?;
    Ok(())
}

pub fn combine_speedscope_files(speedscopes: Vec<Speedscope>) -> Speedscope {
    let hash_to_new_index_and_frame = create_hash_to_new_index_and_frame(&speedscopes);
    let adjusted_speedscopes =
        adjust_speedscopes_to_new_index(&speedscopes, &hash_to_new_index_and_frame);
    let profile_groups = extract_same_profile_groups(&adjusted_speedscopes);
    let mut new_profiles = Vec::new();
    let mut new_frames = Vec::new();
    for (name, profiles) in profile_groups {
        let combined_weights = combine_profiles_weights(profiles.as_slice());
        let combined_samples = combine_samples_from_multiple_profiles(profiles.as_slice());
        let start_value = profiles[0].startValue;
        let end_value = calculate_end_value(profiles.as_slice());
        new_profiles.push(Profile {
            name: name.clone(),
            unit: profiles[0].unit.clone(),
            startValue: start_value,
            endValue: end_value,
            samples: combined_samples,
            weights: combined_weights,
            r#type: "sampled".to_string(),
        });
        hash_to_new_index_and_frame
            .iter()
            .for_each(|(_, (index, frame))| {
                new_frames.push((*frame).clone());
            });
    }
    let new_speedscope = Speedscope {
        profiles: new_profiles,
        shared: Shared { frames: new_frames },
        schema: "https://www.speedscope.app/file-format-schema.json".to_string(),
        exporter: "py-spy@0.4.0".to_string(),
        name: "py-spy profile".to_string(),
    };
    return new_speedscope;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_speedscope() -> Speedscope {
        let frame1 = Frame {
            name: "function_a".to_string(),
            file: "file1.rs".to_string(),
            line: 10,
            col: Some(5),
        };
        let frame2 = Frame {
            name: "function_b".to_string(),
            file: "file2.rs".to_string(),
            line: 20,
            col: Some(3),
        };

        let profile1 = Profile {
            name: "main".to_string(),
            unit: "nanoseconds".to_string(),
            startValue: 0.0,
            endValue: 10.0,
            samples: vec![vec![0, 1], vec![1, 0]],
            weights: vec![1.0, 2.0],
            r#type: "sampled".to_string(),
        };

        Speedscope {
            profiles: vec![profile1],
            shared: Shared {
                frames: vec![frame1, frame2],
            },
            schema: "https://www.speedscope.app/file-format-schema.json".to_string(),
            exporter: "test-exporter".to_string(),
            name: "test-profile".to_string(),
        }
    }

    #[test]
    fn test_combine_speedscope_files_single_input() {
        let speedscope = create_sample_speedscope();
        let result = combine_speedscope_files(vec![speedscope]);
        assert_eq!(result.profiles.len(), 1);
        assert_eq!(result.shared.frames.len(), 2);
        assert_eq!(result.profiles[0].samples.len(), 2);
    }

    #[test]
    fn test_combine_speedscope_files_multiple_inputs() {
        let speedscope1 = create_sample_speedscope();
        let mut speedscope2 = create_sample_speedscope();
        speedscope2.profiles[0].samples = vec![vec![1, 1]];
        speedscope2.profiles[0].weights = vec![3.0];

        let result = combine_speedscope_files(vec![speedscope1, speedscope2]);

        assert_eq!(result.profiles.len(), 1);
        assert_eq!(result.profiles[0].weights.len(), 3);
        assert_eq!(result.profiles[0].samples.len(), 3);
    }

    #[test]
    fn test_combine_speedscope_files_empty_input() {
        let result = combine_speedscope_files(vec![]);
        assert!(result.profiles.is_empty());
        assert!(result.shared.frames.is_empty());
    }
}
