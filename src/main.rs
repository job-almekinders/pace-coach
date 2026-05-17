mod classifier;
mod collector;
mod nudge;
mod settings;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "pace-coach",
    about = "Typing pace monitor — nudges you when your pace signals rushing"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Internal flag: run as background daemon (do not use directly)
    #[arg(long, hide = true)]
    daemon: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Start,
    /// Stop the background daemon
    Stop,
    /// Show current typing state
    Status {
        #[arg(long)]
        verbose: bool,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Print current configuration
    Show,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MetricsSnapshot {
    state: String,
    emoji: String,
    kpm60: f64,
    kpm_10: f64,
    var_dt: f64,
    correction_rate: f64,
}

fn pace_coach_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".pace-coach")
}

fn main() {
    let cli = Cli::parse();

    if cli.daemon {
        run_daemon();
        return;
    }

    match cli.command {
        Some(Commands::Start) => start_daemon(),
        Some(Commands::Stop) => stop_daemon(),
        Some(Commands::Status { verbose }) => status(verbose),
        Some(Commands::Config {
            command: ConfigCommands::Show,
        }) => config_show(),
        None => {
            eprintln!("No command given. Try `pace-coach --help`");
            std::process::exit(1);
        }
    }
}

fn spawn_menubar(dir: &std::path::Path) {
    let exe_dir = match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        Some(d) => d,
        None => {
            eprintln!("warning: cannot determine exe directory, menu bar icon unavailable");
            return;
        }
    };

    let menubar_exe = exe_dir.join("pace-coach-menubar");
    if !menubar_exe.exists() {
        eprintln!("warning: pace-coach-menubar not found — menu bar icon unavailable");
        return;
    }

    #[allow(clippy::zombie_processes)]
    let child = match std::process::Command::new(&menubar_exe)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: failed to spawn pace-coach-menubar: {e}");
            return;
        }
    };

    let _ = std::fs::write(dir.join("pace-coach-menubar.pid"), child.id().to_string());
}

fn start_daemon() {
    let dir = pace_coach_dir();
    std::fs::create_dir_all(&dir).expect("cannot create ~/.pace-coach");
    let pid_path = dir.join("pace-coach.pid");

    // Check if already running
    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let alive = std::process::Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if alive {
                println!("already running (pid {pid})");
                return;
            }
        }
    }

    let exe = std::env::current_exe().expect("cannot get current exe path");
    let log_path = dir.join("pace-coach.log");
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("cannot open log file");

    #[allow(clippy::zombie_processes)]
    let child = std::process::Command::new(&exe)
        .arg("--daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::from(log_file))
        .spawn()
        .expect("cannot spawn daemon");

    let pid = child.id();
    std::fs::write(&pid_path, pid.to_string()).expect("cannot write pid file");
    spawn_menubar(&dir);
    println!("started (pid {pid})");
}
fn stop_daemon() {
    let dir = pace_coach_dir();
    let pid_path = dir.join("pace-coach.pid");
    let sock_path = dir.join("pace-coach.sock");

    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("not running (no pid file)");
            std::process::exit(1);
        }
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("invalid pid file");
            std::process::exit(1);
        }
    };

    let _ = std::process::Command::new("kill")
        .arg(pid.to_string())
        .status();
    let _ = std::fs::remove_file(&pid_path);
    let _ = std::fs::remove_file(&sock_path);

    // Kill menu bar process
    let menubar_pid_path = dir.join("pace-coach-menubar.pid");
    if let Ok(s) = std::fs::read_to_string(&menubar_pid_path) {
        if let Ok(menubar_pid) = s.trim().parse::<u32>() {
            let _ = std::process::Command::new("kill")
                .arg(menubar_pid.to_string())
                .status();
        }
    }
    let _ = std::fs::remove_file(&menubar_pid_path);

    println!("stopped");
}
fn status(verbose: bool) {
    use std::io::Read;
    use std::os::unix::net::UnixStream;

    let sock_path = pace_coach_dir().join("pace-coach.sock");

    let mut stream = match UnixStream::connect(&sock_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("not running");
            std::process::exit(1);
        }
    };

    let mut buf = String::new();
    if stream.read_to_string(&mut buf).is_err() {
        eprintln!("failed to read from daemon");
        std::process::exit(1);
    }

    let snap: MetricsSnapshot = match serde_json::from_str(&buf) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("invalid response from daemon");
            std::process::exit(1);
        }
    };

    if verbose {
        println!("state:           {} {}", snap.state, snap.emoji);
        println!("kpm (60s):       {:.1}", snap.kpm60);
        println!("kpm (10s):       {:.1}", snap.kpm_10);
        println!("correction rate: {:.3}", snap.correction_rate);
        println!("rhythm variance: {:.2}", snap.var_dt);
    } else {
        println!("{} {}", snap.state, snap.emoji);
    }
}
fn config_show() {
    let path = pace_coach_dir().join("config.json");
    match std::fs::read_to_string(&path) {
        Ok(s) => print!("{s}"),
        Err(_) => {
            let defaults = settings::Settings::default();
            print!("{}", serde_json::to_string_pretty(&defaults).unwrap());
        }
    }
}
fn run_daemon() {
    let dir = pace_coach_dir();
    std::fs::create_dir_all(&dir).expect("cannot create ~/.pace-coach");

    let raw_metrics: Arc<Mutex<Option<collector::RawMetrics>>> = Arc::new(Mutex::new(None));
    let snapshot: Arc<Mutex<MetricsSnapshot>> = Arc::new(Mutex::new(MetricsSnapshot::default()));

    // Collector thread
    let raw_for_collector = raw_metrics.clone();
    std::thread::spawn(move || {
        if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            collector::start(raw_for_collector)
        })) {
            eprintln!("[pace-coach] collector panicked: {e:?}");
            eprintln!("[pace-coach] metrics unavailable — check Input Monitoring permission");
        }
    });

    // Unix socket listener thread — serves MetricsSnapshot JSON on each connection
    let snap_for_socket = snapshot.clone();
    let sock_path = dir.join("pace-coach.sock");
    let _ = std::fs::remove_file(&sock_path); // remove stale socket from previous run
    std::thread::spawn(move || {
        use std::io::Write;
        use std::os::unix::net::UnixListener;

        let listener = match UnixListener::bind(&sock_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[pace-coach] socket bind failed: {e}");
                return;
            }
        };
        for mut s in listener.incoming().flatten() {
            let snap = snap_for_socket.lock().unwrap().clone();
            let json = serde_json::to_string(&snap).unwrap_or_default();
            let _ = s.write_all(json.as_bytes());
        }
    });

    let s = settings::load();
    let mut nudge = nudge::NudgeState::new();

    eprintln!("[pace-coach] daemon started");

    loop {
        std::thread::sleep(Duration::from_secs(2));

        let new_snap = match raw_metrics.lock().unwrap().clone() {
            Some(row) => {
                let state = classifier::classify(
                    row.is_active,
                    row.kpm_60,
                    row.var_dt,
                    row.correction_rate,
                    s.correction_rate_threshold,
                );
                MetricsSnapshot {
                    state: state.label().into(),
                    emoji: state.emoji().into(),
                    kpm60: row.kpm_60,
                    kpm_10: (row.kpm_10 * 10.0).round() / 10.0,
                    var_dt: (row.var_dt * 100.0).round() / 100.0,
                    correction_rate: (row.correction_rate * 1000.0).round() / 1000.0,
                }
            }
            None => MetricsSnapshot::default(),
        };

        *snapshot.lock().unwrap() = new_snap.clone();
        if nudge.update(
            &new_snap.state,
            s.stress_duration_secs,
            s.nudge_cooldown_secs,
        ) {
            nudge::notify("Slow down, take a breath, move away from your laptop.");
        }
    }
}
