//! Command-line interface for local Secret Sorbet password generation.
//!
//! Passphrases are accepted only through a hidden terminal prompt or explicit
//! standard input. Diagnostic events never include the passphrase, generated
//! password, service, username, or counter.

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

use arboard::Clipboard;
use clap::{ArgAction, Parser, Subcommand};
use secret_sorbet_pw::{ALGORITHM_VERSION, LoginInput, generate_login_password};
use thiserror::Error;
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use zeroize::Zeroizing;

/// Maximum UTF-8 byte length accepted for a passphrase by the CLI.
pub const MAX_PASSPHRASE_BYTES: usize = 1024;

/// Number of seconds a generated password remains on the clipboard.
pub const CLIPBOARD_TTL_SECONDS: u64 = 120;

/// Secret Sorbet command-line arguments.
#[derive(Parser)]
#[command(
    name = "seso",
    version,
    about = "Generate deterministic passwords without storing them"
)]
pub struct Cli {
    /// Increase diagnostic verbosity; repeat for debug diagnostics.
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    /// Return the requested diagnostic verbosity.
    pub const fn verbosity(&self) -> u8 {
        self.verbose
    }

    /// Return whether command execution consumes the process standard input.
    pub const fn uses_standard_input(&self) -> bool {
        matches!(
            &self.command,
            Command::Generate(args) if args.passphrase_stdin
        )
    }
}

#[derive(Subcommand)]
enum Command {
    /// Generate a deterministic login password.
    Generate(GenerateArgs),
}

#[derive(clap::Args)]
struct GenerateArgs {
    /// Service domain, such as example.com.
    #[arg(
        long,
        value_name = "DOMAIN",
        required_unless_present = "interactive",
        conflicts_with = "interactive"
    )]
    service: Option<String>,

    /// Account username or email address.
    #[arg(
        long,
        value_name = "USERNAME",
        required_unless_present = "interactive",
        conflicts_with = "interactive"
    )]
    username: Option<String>,

    /// Password revision counter.
    #[arg(long, default_value = "1", value_name = "COUNTER")]
    counter: String,

    /// Read the passphrase from standard input instead of a hidden terminal prompt.
    #[arg(long)]
    passphrase_stdin: bool,

    /// Prompt for service, username, and passphrase.
    #[arg(short = 'i', long, conflicts_with = "passphrase_stdin")]
    interactive: bool,

    /// Print the generated password to standard output instead of using the clipboard.
    #[arg(long)]
    print: bool,
}

/// Failures produced by CLI setup or command execution.
#[derive(Debug, Error)]
pub enum Error {
    /// A passphrase could not be read from the terminal.
    #[error(
        "could not read the passphrase from the terminal; use --passphrase-stdin for non-interactive input"
    )]
    TerminalInput(#[source] io::Error),

    /// A visible value could not be read from the terminal.
    #[error("could not read interactive input from the terminal")]
    InteractiveInput(#[source] io::Error),

    /// Clipboard confirmation could not be read from the terminal.
    #[error(
        "could not ask for clipboard confirmation; use --print in a non-interactive environment"
    )]
    ClipboardConfirmation(#[source] io::Error),

    /// Clap's argument guarantees were not upheld by a programmatic caller.
    #[error("required command-line input is missing")]
    MissingArgument,

    /// A passphrase could not be read from standard input.
    #[error("could not read the passphrase from standard input")]
    StandardInput(#[source] io::Error),

    /// A passphrase exceeded the CLI resource bound.
    #[error("the passphrase exceeds the {MAX_PASSPHRASE_BYTES}-byte limit")]
    PassphraseTooLong,

    /// Standard input contained more than one passphrase line.
    #[error("the passphrase must not contain a line break")]
    MultilinePassphrase,

    /// Password generation rejected the supplied inputs.
    #[error("password generation failed: {0}")]
    Generation(#[from] secret_sorbet_pw::Error),

    /// The generated password could not be written to standard output.
    #[error("could not write the generated password to standard output")]
    StandardOutput(#[source] io::Error),

    /// Terminal status output could not be written.
    #[error("could not write clipboard status to the terminal")]
    TerminalOutput(#[source] io::Error),

    /// The platform clipboard could not be opened.
    #[error("could not access the system clipboard")]
    ClipboardInitialize(#[source] arboard::Error),

    /// The generated password could not be copied.
    #[error("could not copy the generated password to the system clipboard")]
    ClipboardWrite(#[source] arboard::Error),

    /// The generated password could not be removed from the clipboard.
    #[error("could not clear the generated password from the system clipboard")]
    ClipboardClear(#[source] arboard::Error),

    /// Graceful shutdown handling could not be installed.
    #[error("could not install the clipboard cleanup signal handler")]
    SignalHandler(#[source] ctrlc::Error),

    /// The process-wide diagnostic subscriber could not be initialized.
    #[error("could not initialize diagnostic logging")]
    Diagnostics,
}

impl Error {
    /// Return a stable, non-sensitive identifier suitable for diagnostics.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::TerminalInput(_) => "terminal_input",
            Self::InteractiveInput(_) => "interactive_input",
            Self::ClipboardConfirmation(_) => "clipboard_confirmation",
            Self::MissingArgument => "missing_argument",
            Self::StandardInput(_) => "standard_input",
            Self::PassphraseTooLong => "passphrase_too_long",
            Self::MultilinePassphrase => "multiline_passphrase",
            Self::Generation(_) => "generation",
            Self::StandardOutput(_) => "standard_output",
            Self::TerminalOutput(_) => "terminal_output",
            Self::ClipboardInitialize(_) => "clipboard_initialize",
            Self::ClipboardWrite(_) => "clipboard_write",
            Self::ClipboardClear(_) => "clipboard_clear",
            Self::SignalHandler(_) => "signal_handler",
            Self::Diagnostics => "diagnostics",
        }
    }
}

/// Initialize process-wide, stderr-only structured diagnostics.
///
/// Applications must call this at most once. No user-provided fields are
/// attached to emitted events.
pub fn init_diagnostics(verbosity: u8) -> Result<(), Error> {
    let level = match verbosity {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        _ => LevelFilter::DEBUG,
    };

    let filter = tracing_subscriber::filter::filter_fn(move |metadata| {
        is_application_diagnostic_target(metadata.target()) && level >= *metadata.level()
    });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .with_writer(io::stderr)
                .with_filter(filter),
        )
        .try_init()
        .map_err(|_| Error::Diagnostics)
}

fn is_application_diagnostic_target(target: &str) -> bool {
    target == "secret_sorbet_cli"
        || target.starts_with("secret_sorbet_cli::")
        || target == "seso"
        || target.starts_with("seso::")
}

/// Execute a parsed CLI command.
///
/// By default, successful generation offers to copy the password to the system
/// clipboard for two minutes. With `--print`, exactly one password and a trailing
/// newline are written to `output`. When `--passphrase-stdin` is selected, one
/// trailing line ending is removed from `input`; all other bytes are preserved.
pub fn run(cli: Cli, input: &mut dyn BufRead, output: &mut dyn Write) -> Result<(), Error> {
    let mut prompt = TerminalPrompt;
    run_with_prompt(cli, input, output, &mut prompt)
}

fn run_with_prompt(
    cli: Cli,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    prompt: &mut dyn Prompt,
) -> Result<(), Error> {
    match cli.command {
        Command::Generate(args) => generate(args, input, output, prompt),
    }
}

fn generate(
    args: GenerateArgs,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    prompt: &mut dyn Prompt,
) -> Result<(), Error> {
    info!(
        event = "generation_started",
        algorithm = ALGORITHM_VERSION,
        "password generation started"
    );

    let (service, username, mut passphrase) = if args.interactive {
        let service = prompt
            .visible("Service: ")
            .map_err(Error::InteractiveInput)?;
        let username = prompt
            .visible("Username: ")
            .map_err(Error::InteractiveInput)?;
        let passphrase = prompt
            .hidden("Passphrase: ")
            .map_err(Error::TerminalInput)
            .and_then(|value| validate_passphrase(Zeroizing::new(value)))?;
        (service, username, passphrase)
    } else {
        let service = args.service.ok_or(Error::MissingArgument)?;
        let username = args.username.ok_or(Error::MissingArgument)?;
        let passphrase = read_passphrase(args.passphrase_stdin, input, prompt)?;
        (service, username, passphrase)
    };

    let mut login = LoginInput::new(std::mem::take(&mut *passphrase), service, username);
    login.counter = args.counter;
    let password = Zeroizing::new(generate_login_password(&login)?);

    if args.print {
        writeln!(output, "{}", password.as_str()).map_err(Error::StandardOutput)?;
        output.flush().map_err(Error::StandardOutput)?;
    } else if confirm_clipboard(prompt)? {
        copy_with_expiry(&password, prompt)?;
    } else {
        prompt
            .status("Password was not copied.\n")
            .map_err(Error::TerminalOutput)?;
    }

    info!(
        event = "generation_completed",
        algorithm = ALGORITHM_VERSION,
        output_length = password.len(),
        "password generation completed"
    );
    Ok(())
}

fn read_passphrase(
    from_stdin: bool,
    input: &mut dyn BufRead,
    prompt: &mut dyn Prompt,
) -> Result<Zeroizing<String>, Error> {
    if !from_stdin {
        let passphrase = prompt
            .hidden("Passphrase: ")
            .map_err(Error::TerminalInput)?;
        return validate_passphrase(Zeroizing::new(passphrase));
    }

    let mut passphrase = Zeroizing::new(String::new());
    (&mut *input)
        .take((MAX_PASSPHRASE_BYTES + 3) as u64)
        .read_to_string(&mut passphrase)
        .map_err(Error::StandardInput)?;
    strip_one_line_ending(&mut passphrase);
    validate_passphrase(passphrase)
}

trait Prompt {
    fn visible(&mut self, label: &str) -> io::Result<String>;
    fn hidden(&mut self, label: &str) -> io::Result<String>;
    fn status(&mut self, message: &str) -> io::Result<()>;
}

struct TerminalPrompt;

impl Prompt for TerminalPrompt {
    fn visible(&mut self, label: &str) -> io::Result<String> {
        let mut output = terminal_output()?;
        output.write_all(label.as_bytes())?;
        output.flush()?;

        let mut value = String::new();
        BufReader::new(terminal_input()?).read_line(&mut value)?;
        strip_one_line_ending(&mut value);
        Ok(value)
    }

    fn hidden(&mut self, label: &str) -> io::Result<String> {
        rpassword::prompt_password(label)
    }

    fn status(&mut self, message: &str) -> io::Result<()> {
        let mut output = terminal_output()?;
        output.write_all(message.as_bytes())?;
        output.flush()
    }
}

fn confirm_clipboard(prompt: &mut dyn Prompt) -> Result<bool, Error> {
    loop {
        let response = prompt
            .visible("Copy to clipboard for 2 minutes? [Y/n] ")
            .map_err(Error::ClipboardConfirmation)?;
        match response.trim().to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => prompt
                .status("Please answer yes or no.\n")
                .map_err(Error::TerminalOutput)?,
        }
    }
}

trait ClipboardAccess {
    fn set_sensitive_text(&mut self, value: &str) -> Result<(), arboard::Error>;
    fn get_text(&mut self) -> Result<String, arboard::Error>;
    fn clear(&mut self) -> Result<(), arboard::Error>;
}

impl ClipboardAccess for Clipboard {
    fn set_sensitive_text(&mut self, value: &str) -> Result<(), arboard::Error> {
        #[cfg(target_os = "linux")]
        {
            use arboard::SetExtLinux;
            self.set().exclude_from_history().text(value)
        }

        #[cfg(windows)]
        {
            use arboard::SetExtWindows;
            self.set().exclude_from_monitoring().text(value)
        }

        #[cfg(target_os = "macos")]
        {
            use arboard::SetExtApple;
            self.set().exclude_from_history().text(value)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            self.set_text(value)
        }
    }

    fn get_text(&mut self) -> Result<String, arboard::Error> {
        Clipboard::get_text(self)
    }

    fn clear(&mut self) -> Result<(), arboard::Error> {
        Clipboard::clear(self)
    }
}

fn copy_with_expiry(password: &str, prompt: &mut dyn Prompt) -> Result<(), Error> {
    let mut clipboard = Clipboard::new().map_err(Error::ClipboardInitialize)?;
    let (shutdown_sender, shutdown_receiver) = mpsc::channel();
    ctrlc::set_handler(move || {
        let _ = shutdown_sender.send(());
    })
    .map_err(Error::SignalHandler)?;

    clipboard
        .set_sensitive_text(password)
        .map_err(Error::ClipboardWrite)?;
    info!(
        event = "clipboard_copy_started",
        ttl_seconds = CLIPBOARD_TTL_SECONDS,
        "generated password copied to clipboard"
    );

    let wait_result = wait_with_countdown(&shutdown_receiver, CLIPBOARD_TTL_SECONDS, prompt);
    let clear_result = clear_if_unchanged(&mut clipboard, password, prompt);
    let outcome = clear_result.and(wait_result)?;
    info!(
        event = "clipboard_cleanup_completed",
        trigger = outcome.as_str(),
        "clipboard cleanup completed"
    );
    Ok(())
}

#[derive(Clone, Copy)]
enum WaitOutcome {
    Expired,
    Shutdown,
}

impl WaitOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::Shutdown => "shutdown",
        }
    }
}

fn wait_with_countdown(
    shutdown: &Receiver<()>,
    seconds: u64,
    prompt: &mut dyn Prompt,
) -> Result<WaitOutcome, Error> {
    for remaining in (1..=seconds).rev() {
        prompt
            .status(&format!(
                "\rPassword copied. Clearing in {:02}:{:02} (Ctrl+C clears now).",
                remaining / 60,
                remaining % 60
            ))
            .map_err(Error::TerminalOutput)?;

        match shutdown.recv_timeout(Duration::from_secs(1)) {
            Ok(()) => return Ok(WaitOutcome::Shutdown),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(WaitOutcome::Shutdown),
        }
    }
    Ok(WaitOutcome::Expired)
}

fn clear_if_unchanged(
    clipboard: &mut dyn ClipboardAccess,
    password: &str,
    prompt: &mut dyn Prompt,
) -> Result<(), Error> {
    match clipboard.get_text() {
        Ok(current) => {
            let current = Zeroizing::new(current);
            if current.as_str() == password {
                clipboard.clear().map_err(Error::ClipboardClear)?;
                prompt
                    .status(
                        "\rPassword removed from active clipboard; history exclusion was requested.\n",
                    )
                    .map_err(Error::TerminalOutput)?;
            } else {
                prompt
                    .status("\rClipboard changed; generated password is no longer active.\n")
                    .map_err(Error::TerminalOutput)?;
            }
        }
        Err(_) => {
            warn!(
                event = "clipboard_verification_failed",
                "clipboard could not be verified; clearing it for safety"
            );
            clipboard.clear().map_err(Error::ClipboardClear)?;
            prompt
                .status("\rClipboard could not be verified and was cleared for safety.\n")
                .map_err(Error::TerminalOutput)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn terminal_input() -> io::Result<File> {
    OpenOptions::new().read(true).open("/dev/tty")
}

#[cfg(unix)]
fn terminal_output() -> io::Result<File> {
    OpenOptions::new().write(true).open("/dev/tty")
}

#[cfg(windows)]
fn terminal_input() -> io::Result<File> {
    OpenOptions::new().read(true).open("CONIN$")
}

#[cfg(windows)]
fn terminal_output() -> io::Result<File> {
    OpenOptions::new().write(true).open("CONOUT$")
}

#[cfg(not(any(unix, windows)))]
fn terminal_input() -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "terminal input is unsupported on this platform",
    ))
}

#[cfg(not(any(unix, windows)))]
fn terminal_output() -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "terminal output is unsupported on this platform",
    ))
}

fn validate_passphrase(passphrase: Zeroizing<String>) -> Result<Zeroizing<String>, Error> {
    if passphrase.len() > MAX_PASSPHRASE_BYTES {
        return Err(Error::PassphraseTooLong);
    }
    if passphrase.contains(['\r', '\n']) {
        return Err(Error::MultilinePassphrase);
    }
    Ok(passphrase)
}

fn strip_one_line_ending(value: &mut String) {
    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockPrompt {
        responses: Vec<String>,
        labels: Vec<String>,
        status: String,
    }

    impl MockPrompt {
        fn with_responses(responses: &[&str]) -> Self {
            Self {
                responses: responses
                    .iter()
                    .rev()
                    .map(|value| (*value).to_owned())
                    .collect(),
                labels: Vec::new(),
                status: String::new(),
            }
        }

        fn respond(&mut self, label: &str) -> io::Result<String> {
            self.labels.push(label.to_owned());
            self.responses
                .pop()
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "no mock response"))
        }
    }

    impl Prompt for MockPrompt {
        fn visible(&mut self, label: &str) -> io::Result<String> {
            self.respond(label)
        }

        fn hidden(&mut self, label: &str) -> io::Result<String> {
            self.respond(label)
        }

        fn status(&mut self, message: &str) -> io::Result<()> {
            self.status.push_str(message);
            Ok(())
        }
    }

    #[test]
    fn strips_one_unix_line_ending() {
        let mut value = "passphrase\n".to_owned();
        strip_one_line_ending(&mut value);
        assert_eq!(value, "passphrase");
    }

    #[test]
    fn diagnostics_exclude_dependency_targets() {
        assert!(is_application_diagnostic_target("secret_sorbet_cli"));
        assert!(is_application_diagnostic_target(
            "secret_sorbet_cli::clipboard"
        ));
        assert!(is_application_diagnostic_target("seso"));
        assert!(!is_application_diagnostic_target("arboard"));
        assert!(!is_application_diagnostic_target("other_dependency"));
    }

    #[test]
    fn strips_one_windows_line_ending() {
        let mut value = "passphrase\r\n".to_owned();
        strip_one_line_ending(&mut value);
        assert_eq!(value, "passphrase");
    }

    #[test]
    fn preserves_content_without_a_line_ending() {
        let mut value = "passphrase\r".to_owned();
        strip_one_line_ending(&mut value);
        assert_eq!(value, "passphrase\r");
    }

    #[test]
    fn rejects_multiline_standard_input() {
        let mut input = io::Cursor::new("first\nsecond\n");
        let error = read_passphrase(true, &mut input, &mut MockPrompt::default()).unwrap_err();

        assert_eq!(error.code(), "multiline_passphrase");
    }

    #[test]
    fn rejects_oversized_standard_input() {
        let value = "x".repeat(MAX_PASSPHRASE_BYTES + 1);
        let mut input = io::Cursor::new(value);
        let error = read_passphrase(true, &mut input, &mut MockPrompt::default()).unwrap_err();

        assert_eq!(error.code(), "passphrase_too_long");
    }

    #[test]
    fn stdin_is_only_selected_explicitly() {
        let prompted = Cli::try_parse_from([
            "seso",
            "generate",
            "--service",
            "example.com",
            "--username",
            "alice@example.com",
        ])
        .unwrap();
        let piped = Cli::try_parse_from([
            "seso",
            "generate",
            "--service",
            "example.com",
            "--username",
            "alice@example.com",
            "--passphrase-stdin",
        ])
        .unwrap();

        assert!(!prompted.uses_standard_input());
        assert!(piped.uses_standard_input());
    }

    #[test]
    fn interactive_mode_prompts_in_order_and_generates_the_same_password() {
        let cli = Cli::try_parse_from(["seso", "generate", "-i", "--print"]).unwrap();
        let mut prompt = MockPrompt::with_responses(&[
            "github.com",
            "ted@example.com",
            "long private passphrase",
        ]);
        let mut input = io::empty();
        let mut output = Vec::new();

        run_with_prompt(cli, &mut input, &mut output, &mut prompt).unwrap();

        assert_eq!(prompt.labels, ["Service: ", "Username: ", "Passphrase: "]);
        assert_eq!(output, b"jSOua2Pe8n-xhU-kui9#\n");
    }

    #[test]
    fn declining_clipboard_delivery_never_emits_the_password() {
        let cli = Cli::try_parse_from([
            "seso",
            "generate",
            "--service",
            "github.com",
            "--username",
            "ted@example.com",
        ])
        .unwrap();
        let mut prompt = MockPrompt::with_responses(&["long private passphrase", "n"]);
        let mut input = io::empty();
        let mut output = Vec::new();

        run_with_prompt(cli, &mut input, &mut output, &mut prompt).unwrap();

        assert!(output.is_empty());
        assert_eq!(
            prompt.labels,
            ["Passphrase: ", "Copy to clipboard for 2 minutes? [Y/n] "]
        );
        assert_eq!(prompt.status, "Password was not copied.\n");
    }

    #[test]
    fn long_interactive_option_selects_interactive_mode() {
        Cli::try_parse_from(["seso", "generate", "--interactive"]).unwrap();
    }

    #[test]
    fn argument_mode_still_requires_service_and_username() {
        let error = match Cli::try_parse_from(["seso", "generate"]) {
            Err(error) => error,
            Ok(_) => panic!("argument mode must reject missing inputs"),
        };
        let message = error.to_string();

        assert!(message.contains("--service <DOMAIN>"));
        assert!(message.contains("--username <USERNAME>"));
    }

    #[test]
    fn interactive_mode_rejects_argument_and_stdin_inputs() {
        for conflicting in ["--service", "--username", "--passphrase-stdin"] {
            let mut arguments = vec!["seso", "generate", "--interactive", conflicting];
            if conflicting != "--passphrase-stdin" {
                arguments.push("value");
            }

            assert!(Cli::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn clipboard_confirmation_defaults_to_yes_and_accepts_no() {
        let mut default_prompt = MockPrompt::with_responses(&[""]);
        assert!(confirm_clipboard(&mut default_prompt).unwrap());

        let mut no_prompt = MockPrompt::with_responses(&["no"]);
        assert!(!confirm_clipboard(&mut no_prompt).unwrap());
    }

    #[test]
    fn clipboard_confirmation_reprompts_after_an_invalid_answer() {
        let mut prompt = MockPrompt::with_responses(&["maybe", "y"]);
        assert!(confirm_clipboard(&mut prompt).unwrap());
        assert_eq!(prompt.labels.len(), 2);
        assert_eq!(prompt.status, "Please answer yes or no.\n");
    }

    struct MockClipboard {
        text: String,
        cleared: bool,
    }

    impl ClipboardAccess for MockClipboard {
        fn set_sensitive_text(&mut self, value: &str) -> Result<(), arboard::Error> {
            self.text = value.to_owned();
            Ok(())
        }

        fn get_text(&mut self) -> Result<String, arboard::Error> {
            Ok(self.text.clone())
        }

        fn clear(&mut self) -> Result<(), arboard::Error> {
            self.text.clear();
            self.cleared = true;
            Ok(())
        }
    }

    #[test]
    fn clears_the_generated_password_and_reports_cleanup() {
        let mut clipboard = MockClipboard {
            text: "generated secret".to_owned(),
            cleared: false,
        };
        let mut prompt = MockPrompt::default();

        clear_if_unchanged(&mut clipboard, "generated secret", &mut prompt).unwrap();

        assert!(clipboard.cleared);
        assert!(
            prompt
                .status
                .contains("Password removed from active clipboard")
        );
        assert!(!prompt.status.contains("generated secret"));
    }

    #[test]
    fn preserves_clipboard_content_that_replaced_the_password() {
        let mut clipboard = MockClipboard {
            text: "new clipboard content".to_owned(),
            cleared: false,
        };
        let mut prompt = MockPrompt::default();

        clear_if_unchanged(&mut clipboard, "generated secret", &mut prompt).unwrap();

        assert!(!clipboard.cleared);
        assert_eq!(clipboard.text, "new clipboard content");
        assert!(prompt.status.contains("Clipboard changed"));
    }

    #[test]
    fn shutdown_interrupts_the_countdown_without_exposing_the_password() {
        let (sender, receiver) = mpsc::channel();
        sender.send(()).unwrap();
        let mut prompt = MockPrompt::default();

        wait_with_countdown(&receiver, CLIPBOARD_TTL_SECONDS, &mut prompt).unwrap();

        assert!(prompt.status.contains("02:00"));
        assert!(!prompt.status.contains("generated secret"));
    }
}
