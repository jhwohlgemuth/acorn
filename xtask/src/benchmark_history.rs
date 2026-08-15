use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;
const DEFAULT_CRITERION_DIR: &str = "target/criterion";
const DEFAULT_OUTPUT_DIR: &str = "target/benchmark-results";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct History {
    schema_version: u32,
    runs: Vec<BenchmarkRun>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct BenchmarkRun {
    run_id: String,
    timestamp: String,
    #[serde(default)]
    project_version: String,
    commit_sha: String,
    ref_name: String,
    pipeline_id: String,
    pipeline_url: String,
    job_id: String,
    job_url: String,
    environment: Environment,
    benchmarks: Vec<Measurement>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Environment {
    fingerprint: String,
    runner_id: String,
    runner_description: String,
    operating_system: String,
    architecture: String,
    cpu_model: String,
    logical_cores: usize,
    rustc: String,
    cargo: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Measurement {
    id: String,
    group_id: String,
    function_id: Option<String>,
    value_str: Option<String>,
    throughput: Option<Value>,
    estimate_ns: f64,
    mean_ns: f64,
    median_ns: f64,
    slope_ns: Option<f64>,
    std_dev_ns: f64,
    confidence_lower_ns: f64,
    confidence_upper_ns: f64,
}

#[derive(Debug, Deserialize)]
struct CriterionBenchmark {
    group_id: String,
    function_id: Option<String>,
    value_str: Option<String>,
    throughput: Option<Value>,
    full_id: String,
}

#[derive(Debug, Deserialize)]
struct CriterionEstimates {
    mean: Statistic,
    median: Statistic,
    slope: Option<Statistic>,
    std_dev: Statistic,
}

#[derive(Debug, Deserialize)]
struct Statistic {
    confidence_interval: ConfidenceInterval,
    point_estimate: f64,
}

#[derive(Debug, Deserialize)]
struct ConfidenceInterval {
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Debug, Deserialize)]
struct GitLabPackage {
    version: String,
    status: Option<String>,
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        | Some("build") => build_command(&args[1..]),
        | Some("latest-package") => latest_package_command(&args[1..]),
        | _ => Err(
            "usage: cargo xtask benchmark-history build [criterion-dir] [output-dir] [previous-history]\n       cargo xtask benchmark-history latest-package <package-list-json>"
                .to_string(),
        ),
    }
}

fn build_command(args: &[String]) -> Result<(), String> {
    let criterion_dir = args.first().map_or_else(|| PathBuf::from(DEFAULT_CRITERION_DIR), PathBuf::from);
    let output_dir = args.get(1).map_or_else(|| PathBuf::from(DEFAULT_OUTPUT_DIR), PathBuf::from);
    let previous = args.get(2).map(PathBuf::from);
    build_history(&criterion_dir, &output_dir, previous.as_deref()).map(|history| {
        println!("Wrote {} benchmark run(s) to {}", history.runs.len(), output_dir.display());
    })
}

fn latest_package_command(args: &[String]) -> Result<(), String> {
    args.first()
        .ok_or_else(|| "latest-package requires a package-list JSON path".to_string())
        .and_then(|path| fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}")))
        .and_then(|content| latest_package_version(&content))
        .map(|version| println!("{}", version.unwrap_or_default()))
}

fn build_history(criterion_dir: &Path, output_dir: &Path, previous: Option<&Path>) -> Result<History, String> {
    load_measurements(criterion_dir)
        .and_then(|benchmarks| previous_history(previous).map(|history| (benchmarks, history)))
        .and_then(|(benchmarks, previous)| {
            let run = current_run(benchmarks);
            let history = History {
                schema_version: SCHEMA_VERSION,
                runs: previous.runs.into_iter().chain(core::iter::once(run.clone())).collect(),
            };
            write_outputs(output_dir, &run, &history).map(|()| history)
        })
}

fn previous_history(path: Option<&Path>) -> Result<History, String> {
    path.map_or_else(
        || {
            Ok(History {
                schema_version: SCHEMA_VERSION,
                runs: Vec::new(),
            })
        },
        |path| {
            fs::read_to_string(path)
                .map_err(|error| format!("failed to read previous history {}: {error}", path.display()))
                .and_then(|content| {
                    serde_json::from_str::<History>(&content).map_err(|error| format!("failed to parse previous history {}: {error}", path.display()))
                })
                .and_then(|history| {
                    (history.schema_version == SCHEMA_VERSION)
                        .then_some(history)
                        .ok_or_else(|| format!("unsupported benchmark history schema; expected version {SCHEMA_VERSION}"))
                })
        },
    )
}

fn load_measurements(root: &Path) -> Result<Vec<Measurement>, String> {
    estimate_paths(root).and_then(|paths| {
        let measurements = paths
            .iter()
            .map(|estimates| measurement_from_files(&estimates.with_file_name("benchmark.json"), estimates))
            .collect::<Result<Vec<_>, _>>();
        measurements.and_then(|mut measurements| {
            measurements.sort_by(|left, right| left.id.cmp(&right.id));
            (!measurements.is_empty())
                .then_some(measurements)
                .ok_or_else(|| format!("no Criterion estimates found under {}", root.display()))
        })
    })
}

fn estimate_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    fs::read_dir(root)
        .map_err(|error| format!("failed to read Criterion directory {}: {error}", root.display()))
        .and_then(|entries| {
            entries
                .map(|entry| entry.map_err(|error| format!("failed to read Criterion entry: {error}")))
                .collect::<Result<Vec<_>, _>>()
        })
        .and_then(|entries| {
            entries
                .into_iter()
                .map(|entry| {
                    let path = entry.path();
                    if path.is_dir() {
                        estimate_paths(&path)
                    } else if path.file_name().is_some_and(|name| name == "estimates.json")
                        && path.parent().and_then(Path::file_name).is_some_and(|name| name == "new")
                    {
                        Ok(vec![path])
                    } else {
                        Ok(Vec::new())
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|paths| paths.into_iter().flatten().collect())
        })
}

fn measurement_from_files(benchmark_path: &Path, estimates_path: &Path) -> Result<Measurement, String> {
    fs::read_to_string(benchmark_path)
        .map_err(|error| format!("failed to read {}: {error}", benchmark_path.display()))
        .and_then(|benchmark| {
            fs::read_to_string(estimates_path)
                .map_err(|error| format!("failed to read {}: {error}", estimates_path.display()))
                .map(|estimates| (benchmark, estimates))
        })
        .and_then(|(benchmark, estimates)| measurement_from_json(&benchmark, &estimates))
}

fn measurement_from_json(benchmark: &str, estimates: &str) -> Result<Measurement, String> {
    serde_json::from_str::<CriterionBenchmark>(benchmark)
        .map_err(|error| format!("failed to parse Criterion benchmark metadata: {error}"))
        .and_then(|benchmark| {
            serde_json::from_str::<CriterionEstimates>(estimates)
                .map_err(|error| format!("failed to parse Criterion estimates: {error}"))
                .map(|estimates| (benchmark, estimates))
        })
        .map(|(benchmark, estimates)| {
            let selected = estimates.slope.as_ref().unwrap_or(&estimates.mean);
            Measurement {
                id: benchmark.full_id,
                group_id: benchmark.group_id,
                function_id: benchmark.function_id,
                value_str: benchmark.value_str,
                throughput: benchmark.throughput,
                estimate_ns: selected.point_estimate,
                mean_ns: estimates.mean.point_estimate,
                median_ns: estimates.median.point_estimate,
                slope_ns: estimates.slope.as_ref().map(|slope| slope.point_estimate),
                std_dev_ns: estimates.std_dev.point_estimate,
                confidence_lower_ns: selected.confidence_interval.lower_bound,
                confidence_upper_ns: selected.confidence_interval.upper_bound,
            }
        })
}

fn current_run(benchmarks: Vec<Measurement>) -> BenchmarkRun {
    let pipeline_id = environment_value("CI_PIPELINE_ID", "local");
    let job_id = environment_value("CI_JOB_ID", "local");
    BenchmarkRun {
        run_id: format!("{pipeline_id}-{job_id}"),
        timestamp: environment_value("CI_PIPELINE_CREATED_AT", &unix_timestamp()),
        project_version: project_version(),
        commit_sha: environment_value("CI_COMMIT_SHA", &command_output("git", &["rev-parse", "HEAD"], "unknown")),
        ref_name: environment_value("CI_COMMIT_REF_NAME", "local"),
        pipeline_id,
        pipeline_url: environment_value("CI_PIPELINE_URL", ""),
        job_id,
        job_url: environment_value("CI_JOB_URL", ""),
        environment: current_environment(),
        benchmarks,
    }
}

fn current_environment() -> Environment {
    let runner_id = environment_value("CI_RUNNER_ID", "local");
    let architecture = environment_value("CI_RUNNER_EXECUTABLE_ARCH", env::consts::ARCH);
    let cpu_model = environment_value("BENCHMARK_CPU_MODEL", &cpu_model());
    let logical_cores = std::thread::available_parallelism().map_or(1, usize::from);
    let fingerprint = format!("{runner_id}|{architecture}|{cpu_model}|{logical_cores}");
    Environment {
        fingerprint,
        runner_id,
        runner_description: environment_value("CI_RUNNER_DESCRIPTION", "local"),
        operating_system: env::consts::OS.to_string(),
        architecture,
        cpu_model,
        logical_cores,
        rustc: command_output("rustc", &["--version"], "unknown"),
        cargo: command_output("cargo", &["--version"], "unknown"),
    }
}

fn environment_value(name: &str, fallback: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn project_version() -> String {
    normalized_project_version(env::var("CI_COMMIT_TAG").ok().as_deref(), &workspace_version())
}

fn normalized_project_version(tag: Option<&str>, fallback: &str) -> String {
    tag.and_then(|tag| tag.strip_prefix('v'))
        .filter(|version| !version.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn workspace_version() -> String {
    fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|content| content.parse::<toml_edit::DocumentMut>().ok())
        .and_then(|document| document["workspace"]["package"]["version"].as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn command_output(command: &str, args: &[&str], fallback: &str) -> String {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn cpu_model() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|value| value.split_once(':').map(|(_, model)| model.trim().to_string()))
            })
        })
        .unwrap_or_else(|| command_output("sysctl", &["-n", "machdep.cpu.brand_string"], "unknown"))
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn write_outputs(output_dir: &Path, run: &BenchmarkRun, history: &History) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))
        .and_then(|()| write_json(&output_dir.join("run.json"), run))
        .and_then(|()| write_json(&output_dir.join("history.json"), history))
        .and_then(|()| {
            fs::write(output_dir.join("history.csv"), history_csv(history)).map_err(|error| format!("failed to write history.csv: {error}"))
        })
        .and_then(|()| {
            fs::write(output_dir.join("report.html"), history_report(history)).map_err(|error| format!("failed to write report.html: {error}"))
        })
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))
        .and_then(|content| fs::write(path, format!("{content}\n")).map_err(|error| format!("failed to write {}: {error}", path.display())))
}

fn history_csv(history: &History) -> String {
    let header = "schema_version,run_id,timestamp,project_version,commit_sha,ref_name,pipeline_id,pipeline_url,job_id,job_url,environment_fingerprint,runner_id,runner_description,operating_system,cpu_model,architecture,logical_cores,rustc,cargo,benchmark_id,group_id,function_id,value_str,throughput,estimate_ns,mean_ns,median_ns,slope_ns,std_dev_ns,confidence_lower_ns,confidence_upper_ns";
    core::iter::once(header.to_string())
        .chain(history.runs.iter().flat_map(|run| {
            run.benchmarks.iter().map(|benchmark| {
                [
                    SCHEMA_VERSION.to_string(),
                    csv_field(&run.run_id),
                    csv_field(&run.timestamp),
                    csv_field(&run.project_version),
                    csv_field(&run.commit_sha),
                    csv_field(&run.ref_name),
                    csv_field(&run.pipeline_id),
                    csv_field(&run.pipeline_url),
                    csv_field(&run.job_id),
                    csv_field(&run.job_url),
                    csv_field(&run.environment.fingerprint),
                    csv_field(&run.environment.runner_id),
                    csv_field(&run.environment.runner_description),
                    csv_field(&run.environment.operating_system),
                    csv_field(&run.environment.cpu_model),
                    csv_field(&run.environment.architecture),
                    run.environment.logical_cores.to_string(),
                    csv_field(&run.environment.rustc),
                    csv_field(&run.environment.cargo),
                    csv_field(&benchmark.id),
                    csv_field(&benchmark.group_id),
                    csv_field(benchmark.function_id.as_deref().unwrap_or_default()),
                    csv_field(benchmark.value_str.as_deref().unwrap_or_default()),
                    csv_field(&benchmark.throughput.as_ref().map(Value::to_string).unwrap_or_default()),
                    benchmark.estimate_ns.to_string(),
                    benchmark.mean_ns.to_string(),
                    benchmark.median_ns.to_string(),
                    benchmark.slope_ns.map_or_else(String::new, |value| value.to_string()),
                    benchmark.std_dev_ns.to_string(),
                    benchmark.confidence_lower_ns.to_string(),
                    benchmark.confidence_upper_ns.to_string(),
                ]
                .join(",")
            })
        }))
        .chain(core::iter::once(String::new()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn csv_field(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

fn history_report(history: &History) -> String {
    let groups = history.runs.iter().fold(
        BTreeMap::<(String, String), Vec<(&BenchmarkRun, &Measurement)>>::new(),
        |mut groups, run| {
            run.benchmarks.iter().for_each(|measurement| {
                groups
                    .entry((measurement.id.clone(), run.environment.fingerprint.clone()))
                    .or_default()
                    .push((run, measurement));
            });
            groups
        },
    );
    let sections = groups
        .iter()
        .map(|((id, fingerprint), points)| report_section(id, fingerprint, points))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ACORN benchmark history</title><style>{}</style></head><body><main><h1>ACORN benchmark history</h1><p>{} runs, grouped by benchmark and environment fingerprint. Deltas never cross environments.</p>{sections}</main></body></html>\n",
        report_css(),
        history.runs.len()
    )
}

fn report_css() -> &'static str {
    "body{font:14px system-ui,sans-serif;margin:0;background:#f7f8fa;color:#20242a}main{max-width:1100px;margin:auto;padding:2rem}section{background:white;border:1px solid #dfe3e8;border-radius:8px;margin:1rem 0;padding:1rem;overflow:auto}h1,h2{margin-top:0}.meta{color:#59636e}.metric{font-size:1.2rem;font-weight:700}.faster{color:#16733c}.slower{color:#a13028}svg{width:100%;height:180px;background:#fafbfc;border:1px solid #e7e9ec}polyline{fill:none;stroke:#3b6fd8;stroke-width:2}circle{fill:#3b6fd8}table{border-collapse:collapse;width:100%;margin-top:1rem}th,td{text-align:left;border-bottom:1px solid #e7e9ec;padding:.4rem;white-space:nowrap}code{font-size:.85em}"
}

fn report_section(id: &str, fingerprint: &str, points: &[(&BenchmarkRun, &Measurement)]) -> String {
    let latest = points.last().map(|(_, measurement)| measurement.estimate_ns).unwrap_or_default();
    let delta = points
        .iter()
        .rev()
        .take(2)
        .map(|(_, measurement)| measurement.estimate_ns)
        .collect::<Vec<_>>();
    let delta_label = delta
        .get(1)
        .filter(|previous| **previous != 0.0)
        .map(|previous| (latest / previous - 1.0) * 100.0)
        .map(|percent| {
            let class = if percent <= 0.0 { "faster" } else { "slower" };
            format!(" <span class=\"{class}\">({percent:+.2}%)</span>")
        })
        .unwrap_or_default();
    let rows = points
        .iter()
        .map(|(run, measurement)| {
            format!(
                "<tr><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{:.3}</td><td>{:.3}–{:.3}</td><td>{}</td></tr>",
                html_escape(&run.timestamp),
                html_escape(&run.project_version),
                html_escape(&short_sha(&run.commit_sha)),
                html_escape(&run.pipeline_id),
                measurement.estimate_ns,
                measurement.confidence_lower_ns,
                measurement.confidence_upper_ns,
                html_escape(&run.environment.rustc)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<section><h2>{}</h2><p class=\"meta\"><code>{}</code></p><p class=\"metric\">Latest: {:.3} ns{delta_label}</p>{}<table><thead><tr><th>Timestamp</th><th>Version</th><th>Commit</th><th>Pipeline</th><th>Estimate (ns)</th><th>95% CI (ns)</th><th>Toolchain</th></tr></thead><tbody>{rows}</tbody></table></section>",
        html_escape(id),
        html_escape(fingerprint),
        latest,
        trend_svg(points)
    )
}

fn trend_svg(points: &[(&BenchmarkRun, &Measurement)]) -> String {
    let values = points.iter().map(|(_, measurement)| measurement.estimate_ns).collect::<Vec<_>>();
    let min = values.iter().copied().reduce(f64::min).unwrap_or_default();
    let max = values.iter().copied().reduce(f64::max).unwrap_or_default();
    let span = (max - min).max(1.0);
    let denominator = points.len().saturating_sub(1).max(1) as f64;
    let coordinates = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = 20.0 + (index as f64 / denominator) * 760.0;
            let y = 160.0 - ((*value - min) / span) * 140.0;
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>();
    let circles = coordinates
        .iter()
        .map(|coordinate| {
            coordinate
                .split_once(',')
                .map(|(x, y)| format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"3\"/>"))
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg viewBox=\"0 0 800 180\" role=\"img\" aria-label=\"benchmark trend\"><polyline points=\"{}\"/>{circles}</svg>",
        coordinates.join(" ")
    )
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(8).collect()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn latest_package_version(content: &str) -> Result<Option<String>, String> {
    serde_json::from_str::<Vec<GitLabPackage>>(content)
        .map_err(|error| format!("failed to parse GitLab package list: {error}"))
        .map(|packages| {
            packages
                .into_iter()
                .find(|package| package.status.as_deref().is_none_or(|status| status == "default"))
                .map(|package| package.version)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BENCHMARK: &str =
        r#"{"group_id":"document.resolve","function_id":"exact","value_str":null,"throughput":null,"full_id":"document.resolve/exact"}"#;
    const ESTIMATES: &str = r#"{"mean":{"confidence_interval":{"lower_bound":9.0,"upper_bound":11.0},"point_estimate":10.0},"median":{"confidence_interval":{"lower_bound":8.0,"upper_bound":12.0},"point_estimate":10.5},"slope":{"confidence_interval":{"lower_bound":19.0,"upper_bound":21.0},"point_estimate":20.0},"std_dev":{"confidence_interval":{"lower_bound":0.5,"upper_bound":1.5},"point_estimate":1.0}}"#;

    fn measurement(id: &str, estimate: f64) -> Measurement {
        Measurement {
            id: id.to_string(),
            group_id: id.to_string(),
            function_id: None,
            value_str: None,
            throughput: None,
            estimate_ns: estimate,
            mean_ns: estimate,
            median_ns: estimate,
            slope_ns: Some(estimate),
            std_dev_ns: 1.0,
            confidence_lower_ns: estimate - 1.0,
            confidence_upper_ns: estimate + 1.0,
        }
    }

    fn run(id: &str, fingerprint: &str, estimate: f64) -> BenchmarkRun {
        BenchmarkRun {
            run_id: id.to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            project_version: "0.1.70".to_string(),
            commit_sha: "1234567890abcdef".to_string(),
            ref_name: "main".to_string(),
            pipeline_id: id.to_string(),
            pipeline_url: String::new(),
            job_id: id.to_string(),
            job_url: String::new(),
            environment: Environment {
                fingerprint: fingerprint.to_string(),
                runner_id: "1".to_string(),
                runner_description: "runner".to_string(),
                operating_system: "linux".to_string(),
                architecture: "x86_64".to_string(),
                cpu_model: "cpu".to_string(),
                logical_cores: 4,
                rustc: "rustc test".to_string(),
                cargo: "cargo test".to_string(),
            },
            benchmarks: vec![measurement("bench", estimate)],
        }
    }

    #[test]
    fn parses_criterion_and_prefers_slope() {
        let result = measurement_from_json(BENCHMARK, ESTIMATES).expect("Criterion JSON should parse");
        assert_eq!(result.id, "document.resolve/exact");
        assert_eq!(result.estimate_ns, 20.0);
        assert_eq!(result.confidence_lower_ns, 19.0);
    }

    #[test]
    fn selects_latest_complete_package() {
        let packages = r#"[{"version":"broken","status":"error"},{"version":"42","status":"default"}]"#;
        assert_eq!(
            latest_package_version(packages).expect("package list should parse"),
            Some("42".to_string())
        );
    }

    #[test]
    fn normalizes_semantic_version_tag() {
        assert_eq!(normalized_project_version(Some("v1.2.3"), "0.0.0"), "1.2.3");
        assert_eq!(normalized_project_version(Some("nightly"), "0.0.0"), "0.0.0");
    }

    #[test]
    fn prior_runs_without_project_version_remain_compatible() {
        let mut value = serde_json::to_value(run("1", "runner-a", 10.0)).expect("run should serialize");
        value.as_object_mut().expect("run should be an object").remove("project_version");
        let parsed = serde_json::from_value::<BenchmarkRun>(value).expect("legacy run should deserialize");
        assert!(parsed.project_version.is_empty());
    }

    #[test]
    fn rejects_unknown_history_schema() {
        let path = PathBuf::from("target/test_artifacts/benchmark-history-schema.json");
        fs::create_dir_all(path.parent().expect("test path should have parent")).expect("test directory should be created");
        fs::write(&path, r#"{"schema_version":2,"runs":[]}"#).expect("history fixture should be written");
        let result = previous_history(Some(&path));
        fs::remove_file(path).expect("history fixture should be removed");
        assert!(result.is_err(), "unknown history schema should fail");
    }

    #[test]
    fn csv_escapes_metadata() {
        assert_eq!(csv_field("runner, \"one\""), "\"runner, \"\"one\"\"\"");
    }

    #[test]
    fn html_escapes_untrusted_metadata() {
        assert_eq!(html_escape("<runner & 'host'>"), "&lt;runner &amp; &#39;host&#39;&gt;");
    }

    #[test]
    fn report_separates_environment_fingerprints() {
        let history = History {
            schema_version: SCHEMA_VERSION,
            runs: vec![run("1", "runner-a", 10.0), run("2", "runner-b", 20.0)],
        };
        let report = history_report(&history);
        assert_eq!(report.matches("<section>").count(), 2);
        assert!(
            !report.contains("<span class=\"faster\">("),
            "different environments should not produce a faster delta"
        );
        assert!(
            !report.contains("<span class=\"slower\">("),
            "different environments should not produce a slower delta"
        );
    }

    #[test]
    fn report_compares_matching_environment_fingerprints() {
        let history = History {
            schema_version: SCHEMA_VERSION,
            runs: vec![run("1", "runner-a", 10.0), run("2", "runner-a", 12.0)],
        };
        assert!(history_report(&history).contains("+20.00%"));
    }
}
