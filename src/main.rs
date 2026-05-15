use anyhow::{ Context, Result };
use clap::{ Parser, Subcommand };
use dirs::home_dir;
use std::{ fs, io::Write, path::PathBuf, process::{ Command, Stdio } };

#[derive(Parser)]
#[command(name = "ssshu", version, about = "Segfault's Secure SHell Utils")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start reverse SSH tunnel
    Tunnel {
        /// Example: byte@ssh.vapma.wtf
        user_host: String,

        /// Remote port on VPS
        #[arg(short, long)]
        remote: u16,

        /// Local target, example: 127.0.0.1:25565
        #[arg(short, long)]
        local: String,

        /// SSH port
        #[arg(short = 'p', long, default_value_t = 22)]
        ssh_port: u16,
    },

    /// Test SSH connection
    Ping {
        /// Example: byte@ssh.vapma.wtf
        user_host: String,

        /// SSH port
        #[arg(short = 'p', long, default_value_t = 22)]
        ssh_port: u16,
    },

    /// Install SSH public key like ssh-copy-id
    PushKey {
        /// Example: byte@ssh.vapma.wtf
        user_host: String,

        /// Path to public key
        #[arg(short, long)]
        key: Option<PathBuf>,

        /// SSH port
        #[arg(short = 'p', long, default_value_t = 22)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tunnel { user_host, remote, local, ssh_port } => {
            start_reverse_tunnel(&user_host, remote, &local, ssh_port)?;
        }

        Commands::Ping { user_host, ssh_port } => {
            ssh_ping(&user_host, ssh_port)?;
        }

        Commands::PushKey { user_host, key, port } => {
            push_key(&user_host, key, port)?;
        }
    }

    Ok(())
}

fn start_reverse_tunnel(
    user_host: &str,
    remote_port: u16,
    local: &str,
    ssh_port: u16
) -> Result<()> {
    println!("Starting reverse tunnel...");
    println!("0.0.0.0:{remote_port} -> {local}");

    let status = Command::new("ssh")
        .arg("-N")
        .arg("-T")
        .arg("-p")
        .arg(ssh_port.to_string())
        .arg("-o")
        .arg("ServerAliveInterval=60")
        .arg("-o")
        .arg("ServerAliveCountMax=3")
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-R")
        .arg(format!("0.0.0.0:{remote_port}:{local}"))
        .arg(user_host)
        .stdin(Stdio::null())
        .status()
        .context("failed to start ssh")?;

    if !status.success() {
        anyhow::bail!("ssh exited with status {status}");
    }

    Ok(())
}

fn ssh_ping(user_host: &str, ssh_port: u16) -> Result<()> {
    println!("Testing SSH connection...");

    let status = Command::new("ssh")
        .arg("-p")
        .arg(ssh_port.to_string())
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("ConnectTimeout=5")
        .arg(user_host)
        .arg("echo ok")
        .status()
        .context("failed to start ssh")?;

    if status.success() {
        println!("SSH OK");
    } else {
        println!("SSH FAILED");
    }

    Ok(())
}

fn push_key(user_host: &str, key_path: Option<PathBuf>, port: u16) -> Result<()> {
    let default_key = home_dir()
        .context("failed to get home directory")?
        .join(".ssh")
        .join("id_ed25519.pub");

    let key_path = key_path.unwrap_or(default_key);

    if !key_path.exists() {
        anyhow::bail!("public key does not exist: {}", key_path.display());
    }

    let public_key = fs::read_to_string(&key_path).context("failed to read public key")?;

    println!("Using key: {}", key_path.display());

    let remote_command = concat!(
        "mkdir -p ~/.ssh && ",
        "chmod 700 ~/.ssh && ",
        "touch ~/.ssh/authorized_keys && ",
        "chmod 600 ~/.ssh/authorized_keys && ",
        "cat >> ~/.ssh/authorized_keys"
    );

    let mut child = Command::new("ssh")
        .arg("-p")
        .arg(port.to_string())
        .arg(user_host)
        .arg(remote_command)
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to start ssh")?;

    let stdin = child.stdin.as_mut().context("failed to open stdin")?;

    stdin.write_all(public_key.as_bytes())?;

    let status = child.wait()?;

    if !status.success() {
        anyhow::bail!("ssh exited with status {status}");
    }

    println!("Key installed successfully");

    Ok(())
}
