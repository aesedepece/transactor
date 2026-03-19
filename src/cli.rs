/// A really basic CLI that provides only the required argument parsing functionality and nothing
/// else.
// For a more sophisticated CLI use case, I would absolutely have gone with the fantastic `clap`
// crate. However, for such a basic functionality, I think it is not worth having an extra
// dependency.
pub struct CLI {
    /// The path to the CSV file from which transactions will be read.
    pub csv_file_path: String,
}

impl CLI {
    pub fn start() -> Self {
        // Skip arg at index 0, which is always the command name itself
        let mut args = std::env::args().skip(1);

        // Read the first actual argument as a string
        let csv_file_path = match args.next() {
            Some(p) => p,
            None => {
                log::error!("no input transactions CSV file path provided");
                log::error!("usage: cargo run -- <path to transactions.csv>");
                std::process::exit(1);
            }
        };

        // The requirements namely state that the file path should be the "first and only argument",
        // so I am making sure that nothing else is found after the path
        if args.next().is_some() {
            log::error!("unexpected extra arguments after input transactions CSV file path");
            std::process::exit(1);
        }

        log::info!("Transactor starting with CSV file path: {}", csv_file_path);

        CLI { csv_file_path }
    }
}
