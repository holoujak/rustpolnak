use crate::race::{Category, Race, Racer};
use crate::time_utils::format_time_delta_millis;
use std::path::PathBuf;
use tracing::{error, info};

use genpdf::Alignment;
use genpdf::Document;
use genpdf::Element as _;
use genpdf::{elements, fonts, style};

const FONT_DIR: &str = "./assets/fonts";
const DEFAULT_FONT_NAME: &str = "DejaVuSansCondensed";

pub fn print_result(race: &Race, output_file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(FONT_DIR).exists() {
        return Err(format!("Font directory {} not found", FONT_DIR).into());
    }

    let font_dir = FONT_DIR;
    let default_font = fonts::from_files(font_dir, DEFAULT_FONT_NAME, None)
        .expect("Failed to load the default font family");

    let mut doc = genpdf::Document::new(default_font);
    doc.set_title("Race results");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    print_heading(&mut doc, "Race results");
    print_tracks(&mut doc, race);
    print_categories(&mut doc, race);
    print_winners(&mut doc, race);

    match doc.render_to_file(&output_file) {
        Ok(()) => {
            info!("Successfully wrote output file: {}", &output_file.display());
            Ok(())
        }
        Err(e) => {
            error!("Failed to write output file: {}", e);
            Err(Box::<dyn std::error::Error>::from(e))
        }
    }
}

fn print_heading(doc: &mut Document, text: &str) {
    let mut heading = elements::Paragraph::new(text);
    heading.set_alignment(Alignment::Center);
    heading.clone().styled(style::Effect::Bold);
    doc.push(heading);
    doc.push(elements::Break::new(1));
}

fn print_table_header(table: &mut elements::TableLayout) {
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
    table
        .row()
        .element(
            elements::Paragraph::new("Start number")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .element(
            elements::Paragraph::new("First name")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Last name")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Time")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Rank")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .push()
        .expect("Invalid table row");
}

fn print_tracks(doc: &mut Document, race: &Race) {
    for track in &race.tracks {
        let mut finished = race.get_finished_racers_by_track(track);

        if finished.is_empty() {
            continue;
        }

        finished.sort_by(|a, b| a.track_rank.cmp(&b.track_rank));

        doc.push(elements::PageBreak::new());

        let mut table = elements::TableLayout::new(vec![1, 2, 2, 2, 1]);

        print_heading(doc, format!("Track: {}", track.0).as_str());
        print_table_header(&mut table);

        for racer in finished {
            table
                .row()
                .element(elements::Paragraph::new(format!("{}", racer.start_number)).padded(1))
                .element(elements::Paragraph::new(&racer.first_name).padded(1))
                .element(elements::Paragraph::new(&racer.last_name).padded(1))
                .element(elements::Paragraph::new(format_time_delta_millis(racer.time)).padded(1))
                .element(
                    elements::Paragraph::new(format!("{}", racer.track_rank.unwrap_or_default()))
                        .padded(1),
                )
                .push()
                .expect("Invalid table row");
        }

        doc.push(table);
    }
}

fn print_category_table(
    doc: &mut Document,
    category: &Category,
    racers: Vec<&Racer>,
    limit: Option<usize>,
) {
    let mut finished = racers;

    if finished.is_empty() {
        return;
    }

    finished.sort_by(|a, b| a.categories_rank[category].cmp(&b.categories_rank[category]));

    if let Some(limit) = limit {
        finished.truncate(limit);
    } else {
        doc.push(elements::PageBreak::new());
    }

    let mut table = elements::TableLayout::new(vec![1, 2, 2, 2, 1]);

    print_heading(doc, format!("Category: {}", category.0).as_str());
    print_table_header(&mut table);

    for racer in finished {
        table
            .row()
            .element(elements::Paragraph::new(format!("{}", racer.start_number)).padded(1))
            .element(elements::Paragraph::new(&racer.first_name).padded(1))
            .element(elements::Paragraph::new(&racer.last_name).padded(1))
            .element(elements::Paragraph::new(format_time_delta_millis(racer.time)).padded(1))
            .element(
                elements::Paragraph::new(format!("{}", racer.categories_rank[category])).padded(1),
            )
            .push()
            .expect("Invalid table row");
    }

    doc.push(table);
}

fn print_categories(doc: &mut Document, race: &Race) {
    for category in &race.categories {
        let finished = race.get_finished_racers_by_category(category);

        print_category_table(doc, category, finished, None);
    }
}

fn print_winners(doc: &mut Document, race: &Race) {
    doc.push(elements::PageBreak::new());
    print_heading(doc, "Winners");

    for category in &race.categories {
        let finished = race.get_finished_racers_by_category(category);

        print_category_table(doc, category, finished, Some(3));
    }
}
