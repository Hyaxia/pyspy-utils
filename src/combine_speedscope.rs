use crate::speedscope_format::{self, Frame, Profile, Shared, Speedscope};
use std::{collections::HashMap, fs::File, hash::Hash, io::BufReader};

fn combine_samples_from_multiple_profiles(profiles: &[&Profile]) -> Vec<Vec<i32>> {
    let mut combined_samples = Vec::new();
    profiles.iter().for_each(|profile| {
        profile.samples.iter().for_each(|sample| {
            combined_samples.push(sample.clone());
        });
    });
    return combined_samples;
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

fn create_hash_to_new_index_and_frame(
    speedscopes: &[speedscope_format::Speedscope],
) -> HashMap<String, (i32, speedscope_format::Frame)> {
    let mut hash_to_new_index_and_frame = std::collections::HashMap::new();
    let mut index = 0;
    for speedscope in speedscopes {
        speedscope.shared.frames.iter().for_each(|frame| {
            let hash = frame.hash();
            if !hash_to_new_index_and_frame.contains_key(&hash) {
                hash_to_new_index_and_frame.insert(hash, (index.clone(), frame.clone()));
                index += 1;
            }
        });
    }
    return hash_to_new_index_and_frame;
}

fn get_sorted_frames(hash_to_new_index_and_frame: &HashMap<String, (i32, Frame)>) -> Vec<Frame> {
    // Convert HashMap into a vector of (i32, Frame) tuples
    let mut sorted_vec: Vec<(i32, Frame)> = hash_to_new_index_and_frame
        .values()
        .cloned() // Clone the Frame objects to avoid borrowing issues
        .collect();

    // Sort by the i32 value
    sorted_vec.sort_by_key(|(index, _frame)| *index);

    // Extract and collect the Frame objects into a Vec<Frame>
    sorted_vec.into_iter().map(|(_, frame)| frame).collect()
}

fn adjust_speedscope_to_new_indexes_and_frames(
    speedscope: &speedscope_format::Speedscope,
    hash_to_new_index_and_frame: &HashMap<String, (i32, speedscope_format::Frame)>,
) -> speedscope_format::Speedscope {
    let mut new_profiles = vec![];
    let new_shared_frames = get_sorted_frames(hash_to_new_index_and_frame);

    for profile in &speedscope.profiles {
        let mut new_samples = vec![];
        // for each sample, create a list with the relevant new indexes
        for sample in &profile.samples {
            let mut new_sample = vec![];
            for old_index in sample {
                let (new_index, _) = hash_to_new_index_and_frame
                    .get(&speedscope.shared.frames[*old_index as usize].hash())
                    .unwrap();
                new_sample.push(new_index.clone());
            }
            new_samples.push(new_sample);
        }

        new_profiles.push(speedscope_format::Profile {
            name: profile.name.clone(),
            unit: profile.unit.clone(),
            start_value: profile.start_value,
            end_value: profile.end_value,
            samples: new_samples,
            weights: profile.weights.clone(),
            r#type: profile.r#type.clone(),
        });
    }
    let new_shared = speedscope_format::Shared {
        frames: new_shared_frames,
    };
    return speedscope_format::Speedscope {
        profiles: new_profiles,
        shared: new_shared,
        schema: speedscope.schema.clone(),
        exporter: speedscope.exporter.clone(),
        name: speedscope.name.clone(),
    };
}

fn calculate_end_value(profiles: &[&Profile]) -> f64 {
    let mut end_value = 0.0;
    profiles.iter().for_each(|profile| {
        end_value += profile.end_value;
    });
    return end_value;
}

pub fn read_speedscope_files(paths: Vec<&str>) -> Vec<speedscope_format::Speedscope> {
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

pub fn combine_speedscope_files(
    adjusted_speedscopes: Vec<speedscope_format::Speedscope>,
    hash_to_new_index_and_frame: HashMap<String, (i32, speedscope_format::Frame)>,
) -> speedscope_format::Speedscope {
    let profile_groups = extract_same_profile_groups(&adjusted_speedscopes);
    let mut new_profiles = Vec::new();
    let new_shared_frames = get_sorted_frames(&hash_to_new_index_and_frame);
    for (name, profiles) in profile_groups {
        let combined_weights = combine_profiles_weights(profiles.as_slice());
        let combined_samples = combine_samples_from_multiple_profiles(profiles.as_slice());
        let start_value = profiles[0].start_value;
        let end_value = calculate_end_value(profiles.as_slice());
        new_profiles.push(Profile {
            name: name.clone(),
            unit: profiles[0].unit.clone(),
            start_value: start_value,
            end_value: end_value,
            samples: combined_samples,
            weights: combined_weights,
            r#type: "sampled".to_string(),
        });
    }
    let new_speedscope = Speedscope {
        profiles: new_profiles,
        shared: Shared {
            frames: new_shared_frames,
        },
        schema: "https://www.speedscope.app/file-format-schema.json".to_string(),
        exporter: "py-spy@0.4.0".to_string(),
        name: "py-spy profile".to_string(),
    };
    return new_speedscope;
}

/// Combines multiple speedscope files into a single speedscope file
///
/// # Arguments
///
/// * `all_profiles_path` - The path to the file containing the list of all profiles to combine
/// * `combined_speedscope_path` - The path to the file to write the combined speedscope file to
///
/// # Example
///
/// ```rust
/// use combine_speedscope::entry_point;
///
/// entry_point("all_profiles.txt", "combined_speedscope.json");
/// ```
///
/// This will combine all the profiles listed in the `all_profiles.txt` file into a single speedscope file
/// and write it to `combined_speedscope.json`.
pub fn entry_point(
    all_profiles_path: &str,
    combined_speedscope_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let collected_files_str = std::fs::read_to_string(all_profiles_path)?;
    let collected_files_strings = collected_files_str.lines().collect();
    let speedscopes = read_speedscope_files(collected_files_strings);
    let hash_to_new_index_and_frame = create_hash_to_new_index_and_frame(&speedscopes);
    let adjusted_speedscopes: Vec<speedscope_format::Speedscope> = speedscopes
        .iter()
        .map(|speedscope| {
            adjust_speedscope_to_new_indexes_and_frames(speedscope, &hash_to_new_index_and_frame)
        })
        .collect();
    let combined_speedscope =
        combine_speedscope_files(adjusted_speedscopes, hash_to_new_index_and_frame);
    std::fs::write(
        combined_speedscope_path,
        serde_json::to_string(&combined_speedscope)?,
    )?;
    Ok(())
}
