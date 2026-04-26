use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::Args;
use env_logger::{Builder, Env, Target};

#[derive(Debug, Clone, Args)]
pub struct LoggingArgs {
    #[arg(long, env = "LOGGER_ROOT_FILE", help = "Path to a file containing the resolved shared log root directory")]
    pub shared_log_root_file: Option<PathBuf>,
}

struct TeeWriter {
    stdout: io::Stdout,
    file: std::fs::File,
}

impl TeeWriter {
    fn new(log_file: &Path) -> io::Result<Self> {
        if let Some(parent) = log_file.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        Ok(Self {
            stdout: io::stdout(),
            file,
        })
    }
}

impl Write for TeeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write_all(buf)?;
        self.file.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()?;
        self.file.flush()?;
        Ok(())
    }
}

pub fn init_stdout_logger(level: &'static str) {
    let mut builder = Builder::from_env(Env::default().default_filter_or(level));
    builder.target(Target::Stdout);
    builder.init();
}

pub fn init_dual_logger(log_file: impl AsRef<Path>, level: &'static str) -> io::Result<()> {
    let writer = TeeWriter::new(log_file.as_ref())?;

    let mut builder = Builder::from_env(Env::default().default_filter_or(level));
    builder.target(Target::Pipe(Box::new(writer)));
    builder.try_init().map_err(io::Error::other)
}

impl LoggingArgs {
    pub fn try_resolve_log_root(&self) -> io::Result<PathBuf> {
        let file = self.shared_log_root_file.as_ref().ok_or(
            io::Error::new(
                io::ErrorKind::NotFound,
                "shared log root file argument is not set",
            )
        )?;

        let content = std::fs::read_to_string(file)?;
        let root = content.trim();
        if root.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("shared log root file {} is empty", file.display()),
            ));
        }

        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
