use clap::{Parser, Subcommand};
use colored::*;
mod combine_speedscope;
mod run_continuos_pyspy;
mod speedscope_format;

#[derive(Subcommand)]
enum Commands {
    RunContinuosPyspy {
        /// Name of the pod
        #[arg(short = 'p', long)]
        pod_name: String,

        /// Namespace of the pod
        #[arg(short = 'n', long)]
        namespace: String,

        /// Duration of each sample
        #[arg(short = 'd', long)]
        duration_seconds: u16,

        /// Num of samples to take
        #[arg(short = 's', long)]
        num_of_samples: u16,
    },
    CombineSpeedscopeFiles {
        /// The file that contains paths to all of the relevant speedscope files
        #[arg(short, long)]
        all_profiles_file_path: String,
    },
}

#[derive(Parser)]
#[command(
    name = "pyspy-helper",
    version = "1.0",
    about = "useful utils for pyspy"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RunContinuosPyspy {
            pod_name,
            namespace,
            duration_seconds,
            num_of_samples,
        } => {
            println!(
                "{}",
                format!(
                    "====> Running continuos pyspy profiling on pod {} in namespace {} for {} seconds",
                    pod_name, namespace, duration_seconds
                )
                .green()
            );
            match run_continuos_pyspy::run_continuos_pyspy(
                pod_name,
                namespace,
                duration_seconds,
                num_of_samples,
            ) {
                Ok(_) => println!(
                    "{}",
                    format!("====> Successfuly finished running pyspy profiling").green()
                ),
                Err(e) => eprintln!(
                    "{}",
                    format!("====> Error running continuos pyspy profiling: {}", e).red()
                ),
            }
        }
        Commands::CombineSpeedscopeFiles {
            all_profiles_file_path,
        } => {
            println!(
                "{}",
                format!(
                    "====> Combining speedscope files from {}",
                    all_profiles_file_path
                )
                .green()
            );
            let combined_speedscope_file_path = "./profiling_results/combined_speedscope.json";
            match combine_speedscope::entry_point(
                &all_profiles_file_path,
                &combined_speedscope_file_path,
            ) {
                Ok(_) => println!(
                    "{}",
                    format!(
                        "====> Successfuly combined speedscope files to {}",
                        combined_speedscope_file_path
                    )
                    .green()
                ),

                Err(e) => eprintln!(
                    "{}",
                    format!("====> Error combining speedscope files: {}", e).red()
                ),
            }
        }
    }
}
