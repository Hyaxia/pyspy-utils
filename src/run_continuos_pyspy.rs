use chrono::Utc;
use std::fs;
use std::process::Command;

/// Configuration for lifetime profiling
struct ProfilerConfig {
    pub pod_name: String,
    pub namespace: String,
    pub duration_seconds: u16,
    pub num_of_samples: u16,
    pub local_output_dir: String,
}

/// Runs py-spy continuously in chunks, copying the results back to the local machine.
/// This is useful for profiling long-running processes.
/// If py-spy is not installed in the container, it will be installed automatically.
///
/// # Arguments
///
/// * `pod_name` - The name of the pod to profile
/// * `namespace` - The namespace of the pod
/// * `duration_seconds` - The duration of each py-spy run in seconds
///
/// # Example
///
/// ```rust
/// use run_continuos_pyspy::run_continuos_pyspy;
///
/// run_continuos_pyspy("my-pod", "default", 60);
/// ```
///
/// This will run py-spy for 60 seconds in 4 chunks, copying the results back to the local machine.
/// The results will be saved in the `profiling_results` directory.
pub fn run_continuos_pyspy(
    pod_name: String,
    namespace: String,
    duration_seconds: u16,
    num_of_samples: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ProfilerConfig {
        pod_name: pod_name.to_string(),
        namespace: namespace.to_string(),
        duration_seconds: duration_seconds,
        num_of_samples: num_of_samples,
        local_output_dir: "./profiling_results".to_string(),
    };

    fs::create_dir_all(&config.local_output_dir).unwrap();

    ensure_py_spy_installed(&config)?;

    let mut collected_files = Vec::new();
    for i in 0..config.num_of_samples {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let remote_file = format!("pyspy_output_{}.json", timestamp);

        println!(
            "====> Starting py-spy chunk #{} for {} minutes",
            i + 1,
            config.duration_seconds
        );
        run_py_spy(&config, &remote_file)?;

        // Copy results to local machine
        let local_path = format!("{}/{}", config.local_output_dir, remote_file);
        copy_results(&config, &remote_file, &local_path)?;
        collected_files.push(local_path);
    }
    Ok(())
}

fn ensure_py_spy_installed(config: &ProfilerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "====> Checking if py-spy is installed in pod {}",
        config.pod_name
    );

    let py_spy_installed = is_py_spy_installed(config)?;
    if py_spy_installed {
        println!("====> py-spy is installed in the container.");
        return Ok(());
    }

    println!("====> py-spy not found. Installing in container...");
    install_py_spy(config)
}

fn install_py_spy(config: &ProfilerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let install_status = Command::new("kubectl")
        .args(&[
            "exec",
            &config.pod_name,
            "-n",
            &config.namespace,
            "--",
            "pip",
            "install",
            "py-spy",
        ])
        .status()?;

    if !install_status.success() {
        return Err("Failed to install py-spy".into());
    }
    Ok(())
}

fn is_py_spy_installed(config: &ProfilerConfig) -> Result<bool, std::io::Error> {
    let check_output = Command::new("kubectl")
        .args(&[
            "exec",
            &config.pod_name,
            "-n",
            &config.namespace,
            "--",
            "py-spy",
            "--version",
        ])
        .output()?;
    if check_output.status.code().unwrap_or(1) != 0 {
        return Ok(false);
    }
    return Ok(true);
}

fn run_py_spy(
    config: &ProfilerConfig,
    remote_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // we assume that 1 is the target process
    let full_cmd = format!(
        "py-spy record --pid 1 --duration {} --output /tmp/{} --format=speedscope",
        &config.duration_seconds, remote_filename
    );

    println!("====> Running py-spy in container: {}", full_cmd);

    let status = Command::new("kubectl")
        .args(&[
            "exec",
            &config.pod_name,
            "-n",
            &config.namespace,
            "--",
            "bash",
            "-c",
            &full_cmd,
        ])
        .status()?;

    if !status.success() {
        return Err("py-spy record command failed".into());
    }

    Ok(())
}

fn copy_results(
    config: &ProfilerConfig,
    remote_filename: &str,
    local_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let remote_path = format!("/tmp/{}", remote_filename);
    let pod_resource = format!("{}/{}:{}", config.namespace, config.pod_name, remote_path);

    println!("====> Copying results from container: {}", pod_resource);

    let status = Command::new("kubectl")
        .args(&["cp", &pod_resource, local_path])
        .status()?;

    if !status.success() {
        return Err("Failed to copy results from container".into());
    }

    println!("Successfully copied to: {}", local_path);
    Ok(())
}
