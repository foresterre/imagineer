use crate::cli::config::SelectedLicenses;

pub struct LicenseTexts<'a> {
    pub(crate) this_software: &'a str,
}

impl<'a> LicenseTexts<'a> {
    pub fn new(this_software: &'a str) -> Self {
        Self { this_software }
    }
}

pub(crate) trait PrintTextFor {
    fn print(&self, texts: &LicenseTexts) -> anyhow::Result<()>;
}

impl PrintTextFor for SelectedLicenses {
    fn print(&self, texts: &LicenseTexts) -> anyhow::Result<()> {
        let print_for_this_software = || {
            println!("imagineer image tools license:\n\n{}", texts.this_software);

            Ok(())
        };

        let print_for_dependencies = || {
            use std::io::Write;
            println!(
                "You should have received a licenses.html file with the distribution, but you can download another copy here."
            );
            print!("Open new copy [yes / no (default)]: ");
            std::io::stdout().flush()?;

            if let Ok(true) = request_another_copy() {
                open::that(concat!(
                    "https://github.com/foresterre/imagineer/releases/download/v",
                    clap::crate_version!(),
                    "/licenses.html"
                ))?;
            }

            Ok(())
        };

        match self {
            SelectedLicenses::ThisSoftware => print_for_this_software(),
            SelectedLicenses::Dependencies => print_for_dependencies(),
        }
    }
}

fn request_another_copy() -> anyhow::Result<bool> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    let input = buffer.to_ascii_lowercase();

    match input.trim() {
        "yes" | "y" => Ok(true),
        _ => Ok(false),
    }
}
