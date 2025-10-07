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

#[derive(Debug)]
pub enum Action {
    Start(String),
    FinishByStartNumber(u32),
}

#[component]
pub fn App() -> Element {
    let mut selected_race = use_signal(|| Option::<Result<Race, Box<dyn std::error::Error>>>::None);

    use_coroutine(
        move |mut actions_rx: UnboundedReceiver<Action>| async move {
            let (tx, mut rfid_rx) = broadcast::channel::<rfid_reader::Event>(128);
            for serial in use_context::<Config>().rfid_devices {
                tokio::spawn(rfid_reader::rfid_serial(serial, tx.clone()));
            }

            loop {
                tokio::select! {
                    Ok(rfid_event) = rfid_rx.recv() => {
                        println!("{rfid_event:?}");
                        match rfid_event {
                            rfid_reader::Event::Connected(device) => info!("RFID {device} connected"),
                            rfid_reader::Event::Disconnected { device, error } => info!("RFID {device} disconnected: {error:?}"),
                            rfid_reader::Event::Tag(tag) => {
                                info!("Tag {tag}");
                                selected_race.with_mut(|maybe_race| {
                                    if let Some(Ok(race)) = maybe_race {
                                        race.tag_finished(&tag);
                                    }
                                });
                            },
                        }
                    }

                    Some(msg) = actions_rx.next() => {
                        println!("{msg:?}");
                        match msg {
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
                }
            }
        },
    );

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
                None => rsx! {},
            }
        }
    }
}
