use super::statistics_configuration::OutputFormat;
use crate::git::{CommitHours, CommitHoursJson};
use anyhow::Result;
use prettytable::{format, row, Table};

fn get_totals(times: &[CommitHours]) -> (f32, usize) {
    let mut total_estimated_hours = 0.0;
    let mut total_commits = 0;
    for time in times.iter() {
        let commits = time.commit_count;
        let estimated_hours = (time.duration.num_minutes() as f32) / 60.0;
        total_commits += commits;
        total_estimated_hours += estimated_hours;
    }

    (total_estimated_hours, total_commits)
}

fn print_results_stdout(times: &[CommitHours]) -> Result<()> {
    let mut table = Table::new();

    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[format::LinePosition::Top, format::LinePosition::Bottom],
            format::LineSeparator::new('-', '+', '+', '+'),
        )
        .padding(1, 1)
        .build();
    table.set_format(format);

    table.set_titles(row!["Author", "Email", "Commits", "Estimated Hours"]);
    table.add_empty_row();

    for time in times.iter() {
        let author = match &time.author_name {
            Some(n) => n,
            None => "",
        };
        let email = match &time.email {
            Some(email) => email,
            None => "(none)",
        };
        let commits = time.commit_count;
        let estimated_hours = (time.duration.num_minutes() as f32) / 60.0;

        table.add_row(row![author, email, commits, estimated_hours]);
    }

    table.add_empty_row();

    let (total_estimated_hours, total_commits) = get_totals(times);
    table.add_row(row!["Total", "", total_commits, total_estimated_hours]);

    log::debug!("Results: {:?}", table);
    log::debug!("");
    // TODO: Tie this into the log printer.
    table.printstd();

    Ok(())
}

fn print_results_json(times: &[CommitHours]) -> Result<()> {
    let mut times_json = times.iter().map(CommitHoursJson::from).collect::<Vec<_>>();

    let (total_estimated_hours, total_commits) = get_totals(times);
    times_json.push(CommitHoursJson {
        email: None,
        author_name: Some(String::from("Total")),
        hours: total_estimated_hours,
        commit_count: total_commits,
    });

    let json = serde_json::to_string_pretty(&times_json)?;

    log::info!("{}", json);

    Ok(())
}

/// Print times with the specified format.
pub fn print_results(times: &[CommitHours], output_format: &OutputFormat) -> Result<()> {
    match output_format {
        OutputFormat::Stdout => print_results_stdout(times),
        OutputFormat::Json => print_results_json(times),
    }
}
