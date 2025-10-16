use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tracing::{error, info, trace};

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
    FinishByStartNumber(StartNumber, DateTime<Utc>),
    FinishEdit(StartNumber, Option<DateTime<Utc>>),
}

struct RFIDDevices {
    devices: HashMap<String, bool>,
}

impl RFIDDevices {
    fn new(devices: &[String]) -> Self {
        Self {
            devices: devices
                .iter()
                .map(|device| (device.clone(), false))
                .collect(),
        }
    }

    fn set(&mut self, device: &str, connected_now: bool) {
        if let Some(connected) = self.devices.get_mut(device) {
            *connected = connected_now;
        } else {
            error!("RFID {device} does not exist!");
        }
    }

    fn is_ok(&self) -> bool {
        self.devices.iter().all(|(_, connected)| *connected)
    }
}

fn handle_rfid_event(
    selected_race: &mut Signal<SelectedRace>,
    devices: &mut Signal<RFIDDevices>,
    event: rfid_reader::Event,
) {
    match event {
        rfid_reader::Event::Connected(device) => {
            devices.write().set(&device, true);
            info!("RFID {device} connected");
        }
        rfid_reader::Event::Disconnected { device, error } => {
            devices.write().set(&device, false);
            info!("RFID {device} disconnected: {error:?}")
        }
        rfid_reader::Event::Tag(tag) => {
            info!("Tag {tag}");
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.tag_finished(&tag, Some(Utc::now()));
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
        Action::FinishByStartNumber(starting_number, time) => {
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.finish_start_number(starting_number, time);
                }
            });
        }
        Action::FinishEdit(starting_number, time) => {
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.edit_finish_start_number(starting_number, time);
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
    let mut rfid_devices = use_signal(|| RFIDDevices::new(&config.rfid_devices));

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
                       handle_rfid_event(&mut selected_race, &mut rfid_devices, rfid_event);
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
                span {
                    class: "btn",
                    class: if rfid_devices.read().is_ok() { "btn-success" } else { "btn-danger" },
                    dangerous_inner_html: iconify::svg!("ri:rfid-line"),
                }
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
