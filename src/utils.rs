use std::{
    env,
    io::{self, stdout, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use eyre::Context;

pub fn get_full_path(path: impl AsRef<Path>) -> crate::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

pub fn resolve_and_get_file_name(path: impl AsRef<Path>) -> crate::Result<String> {
    let file_name = path
        .as_ref()
        .canonicalize()?
        .file_name()
        .ok_or_else(|| eyre::eyre!("Could not read file name"))?
        .to_str()
        .ok_or_else(|| eyre::eyre!("Could not convert file name to string"))?
        .to_string();

    Ok(file_name)
}

/// Prompts the user in the terminal with a yes/no question. Returns `true` when
/// the user responds "y", `false` otherwise.
pub fn prompt_bool(question: &str) -> crate::Result<bool> {
    print!("{question} (y/n): ");
    io::stdout().flush().wrap_err("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .wrap_err("failed to read input")?;

    Ok(input.trim().to_lowercase() == "y")
}

pub struct TimedPrinter {
    last_print_time: Option<Instant>,
    interval: Duration,
}

impl TimedPrinter {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_print_time: None,
            interval,
        }
    }

    pub fn flush_print(&mut self, msg: impl AsRef<str>, force: bool) -> crate::Result<()> {
        match self.last_print_time {
            Some(last_print_time) if last_print_time.elapsed() < self.interval && !force => {}
            _ => {
                self.last_print_time = Some(Instant::now());
                print!("{}", msg.as_ref());
                stdout().flush().wrap_err("Failed to flush output")?;
            }
        }

        Ok(())
    }
}

pub struct RunningAverager {
    num_samples: usize,
    average: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
}

impl RunningAverager {
    pub fn new() -> Self {
        Self {
            num_samples: 0,
            average: None,
            min: None,
            max: None,
        }
    }

    pub fn add_sample(&mut self, sample: f64) {
        self.num_samples += 1;

        self.average = self
            .average
            .map(|average| average + (sample - average) / self.num_samples as f64)
            .or(Some(sample));

        self.min = self.min.map(|min| min.min(sample)).or(Some(sample));
        self.max = self.max.map(|max| max.max(sample)).or(Some(sample));
    }

    pub fn average(&self) -> Option<f64> {
        self.average
    }

    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }
}
