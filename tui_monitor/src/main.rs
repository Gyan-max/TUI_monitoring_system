use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    io::stdout,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::{sync::mpsc, task::JoinHandle, time};

/// App state
struct App {
    system: System,
    last_update: Instant,
    task_stats: Arc<Mutex<TaskStats>>,
}

/// Statistics for the running tasks
struct TaskStats {
    task_a_count: usize,
    task_b_count: usize,
    last_task_a: Instant,
    last_task_b: Instant,
}

impl TaskStats {
    fn new() -> Self {
        Self {
            task_a_count: 0,
            task_b_count: 0,
            last_task_a: Instant::now(),
            last_task_b: Instant::now(),
        }
    }
}

impl App {
    fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        Self {
            system,
            last_update: Instant::now(),
            task_stats: Arc::new(Mutex::new(TaskStats::new())),
        }
    }

    fn update(&mut self) {
        self.system.refresh_all();
        self.last_update = Instant::now();
    }
}

/// Spawn a task that simulates work and updates task stats
fn spawn_task_a(stats: Arc<Mutex<TaskStats>>, mut shutdown: mpsc::Receiver<()>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown.recv() => break,
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    // Simulate some work with random delay
                    let work_time = Duration::from_millis(50 + (rand() % 200));
                    tokio::time::sleep(work_time).await;
                    
                    // Update stats
                    let mut stats = stats.lock().unwrap();
                    stats.task_a_count += 1;
                    stats.last_task_a = Instant::now();
                }
            }
        }
    })
}

/// Spawn a task that simulates IO work and updates task stats
fn spawn_task_b(stats: Arc<Mutex<TaskStats>>, mut shutdown: mpsc::Receiver<()>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown.recv() => break,
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Simulate IO work
                    tokio::time::sleep(Duration::from_millis(150 + (rand() % 350))).await;
                    
                    // Update stats
                    let mut stats = stats.lock().unwrap();
                    stats.task_b_count += 1;
                    stats.last_task_b = Instant::now();
                }
            }
        }
    })
}

// Simple pseudo-random number generator
fn rand() -> u64 {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    (now % 1000) as u64
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tokio console instrumentation
    console_subscriber::init();

    // Initialize terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("Failed to enter alternate screen")?;
    
    // Create terminal and app state
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    let mut app = App::new();

    // Spawn our sample async tasks
    let (task_a_shutdown_tx, task_a_shutdown_rx) = mpsc::channel(1);
    let (task_b_shutdown_tx, task_b_shutdown_rx) = mpsc::channel(1);
    let task_a = spawn_task_a(app.task_stats.clone(), task_a_shutdown_rx);
    let task_b = spawn_task_b(app.task_stats.clone(), task_b_shutdown_rx);

    // Main loop
    let mut interval = time::interval(Duration::from_millis(1000));
    
    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &app))?;

        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Update system metrics on interval
        tokio::select! {
            _ = interval.tick() => {
                app.update();
            }
        }
    }

    // Clean shutdown
    let _ = task_a_shutdown_tx.send(()).await;
    let _ = task_b_shutdown_tx.send(()).await;
    let _ = task_a.await;
    let _ = task_b.await;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    
    Ok(())
}

/// Renders the user interface
fn ui(f: &mut ratatui::Frame, app: &App) {
    // Split the screen into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title bar
    let title = Paragraph::new("TUI System Monitor (Press 'q' to quit)")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // System metrics area
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // CPU metrics
    let cpu_usage = app.system.global_cpu_info().cpu_usage();
    let cpu_text = format!("CPU Usage: {:.2}%", cpu_usage);
    let cpu_info = Paragraph::new(cpu_text)
        .block(Block::default().title("CPU Info").borders(Borders::ALL));
    f.render_widget(cpu_info, metrics_chunks[0]);

    // Memory metrics
    let total_memory = app.system.total_memory() / 1024 / 1024;
    let used_memory = app.system.used_memory() / 1024 / 1024;
    let memory_text = format!(
        "Memory Usage: {}/{} MB ({:.2}%)",
        used_memory,
        total_memory,
        (used_memory as f64 / total_memory as f64) * 100.0
    );
    let memory_info = Paragraph::new(memory_text)
        .block(Block::default().title("Memory Info").borders(Borders::ALL));
    f.render_widget(memory_info, metrics_chunks[1]);

    // Task metrics area
    let task_stats = app.task_stats.lock().unwrap();
    let task_text = format!(
        "Task A: {} executions (last: {:?} ago)\nTask B: {} executions (last: {:?} ago)",
        task_stats.task_a_count,
        task_stats.last_task_a.elapsed(),
        task_stats.task_b_count,
        task_stats.last_task_b.elapsed(),
    );
    let task_info = Paragraph::new(task_text)
        .block(Block::default().title("Async Tasks").borders(Borders::ALL));
    f.render_widget(task_info, chunks[2]);

    // Status bar
    let status = format!(
        "Last update: {:?} ago | Run tokio-console in a separate terminal to view detailed async metrics",
        app.last_update.elapsed()
    );
    let status_info = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_info, chunks[3]);
}
