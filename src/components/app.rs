use std::path::PathBuf;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tracing::{info, trace};

use crate::{
    components::{
        manual_start_number_input::ManualStartNumberInput, racers::Racers, races_list::RacesList,
        track_start::TrackStart, upload_results::UploadResults,
    },
    config::Config,
    printer::print_result,
    race::{Race, StartNumber, Track},
    rfid_reader,
};

type SelectedRace = Option<Result<Race, Box<dyn std::error::Error>>>;

const LOADING: Asset = asset!("/assets/loading.webp");

#[derive(Debug)]
pub enum Action {
    Start(Track, DateTime<Utc>),
    FinishByStartNumber(StartNumber),
}

fn handle_rfid_event(selected_race: &mut Signal<SelectedRace>, event: rfid_reader::Event) {
    match event {
        rfid_reader::Event::Connected(device) => info!("RFID {device} connected"),
        rfid_reader::Event::Disconnected { device, error } => {
            info!("RFID {device} disconnected: {error:?}")
        }
        rfid_reader::Event::Tag(tag) => {
            info!("Tag {tag}");
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.tag_finished(&tag);
                }
            });
        }
    }
}

fn handle_action(selected_race: &mut Signal<SelectedRace>, action: Action) {
    match action {
        Action::Start(track, time) => {
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.start(track, time);
                }
            });
        }
        Action::FinishByStartNumber(starting_number) => {
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.finish_start_number(starting_number);
                }
            });
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut selected_race = use_signal(|| SelectedRace::None);
    let config = use_context::<Config>();
    let mut show_starts = use_signal(|| true);
    let results_output_path =
        PathBuf::from(shellexpand::tilde(&config.results_path.clone()).to_string());

    use_coroutine(move |mut actions_rx: UnboundedReceiver<Action>| {
        let config = config.clone();
        async move {
            let (tx, mut rfid_rx) = broadcast::channel::<rfid_reader::Event>(128);
            for serial in config.rfid_devices {
                tokio::spawn(rfid_reader::rfid_serial(serial, tx.clone()));
            }

            loop {
                tokio::select! {
                    Ok(rfid_event) = rfid_rx.recv() => {
                        println!("{rfid_event:?}");
                       handle_rfid_event(&mut selected_race, rfid_event);
                    }
                    Some(action) = actions_rx.next() => {
                        println!("{action:?}");
                        handle_action(&mut selected_race, action);
                    }
                }
            }
        }
    });

    let race = selected_race
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned();

    rsx! {
        div { class: "d-flex flex-column", style: "height: 100vh",
            div { class: "d-flex column-gap-1 mb-1",
                button {
                    class: "btn btn-light",
                    dangerous_inner_html: iconify::svg!("mdi:schedule"),
                    onclick: move |_| {
                        show_starts.toggle();
                    },
                }
                RacesList { selected_race }
                match race {
                    Some(race) => rsx! {
                        UploadResults { race: race.clone() }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                info!("Printing results for race {}", race.id);
                                match print_result(&race, results_output_path.clone()) {
                                    Ok(()) => {
                                        info!("Print job completed successfully.");
                                        open::that(&results_output_path).ok();
                                    }
                                    Err(e) => trace!("Print job failed: {}", e),
                                }
                            },
                            dangerous_inner_html: iconify::svg!("mdi:printer"),
                        }
                    },
                    _ => rsx! {},
                }
            }

            match &*selected_race.read() {
                Some(Ok(race)) => rsx! {
                    if show_starts() {
                        div { class: "d-flex flex-column row-gap-1 mb-1",
                            for (track , start) in race.tracks_with_start() {
                                TrackStart { track, start }
                            }
                        }
                    }
                    ManualStartNumberInput {}
                    Racers { race: race.clone() }
                },
                Some(Err(err)) => rsx! {
                    p { class: "alert alert-danger", "{err:#?}" }
                },
                None => rsx! {
                    div { class: "d-flex justify-content-center align-items-center vh-100",
                        img { src: LOADING }
                    }
                },
            }
        }
    }
}
