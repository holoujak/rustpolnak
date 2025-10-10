use dioxus::prelude::*;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    components::{race::RaceComponent, races_list::RacesList},
    config::Config,
    race::Race,
    rfid_reader,
};

type SelectedRace = Option<Result<Race, Box<dyn std::error::Error>>>;

const LOADING: Asset = asset!("/assets/loading.webp");

#[derive(Debug)]
pub enum Action {
    Start(String),
    FinishByStartNumber(u32),
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
        Action::Start(track) => {
            selected_race.with_mut(|maybe_race| {
                if let Some(Ok(race)) = maybe_race {
                    race.start(track);
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

    rsx! {
        div {
            RacesList { selected_race }
            match &*selected_race.read() {
                Some(Ok(race)) => rsx! {
                    RaceComponent { race: race.clone() }
                },
                Some(Err(err)) => rsx! {
                "{err:#?}"
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
