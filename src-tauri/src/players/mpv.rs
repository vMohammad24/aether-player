use crate::models::config::MpvConfig;
use crate::models::entities::PlayerEvent;
use crate::models::PlayerState;
use crate::traits::{AudioEngine, AudioStream};
use async_trait::async_trait;
use libmpv2::{
    events::{Event, PropertyData},
    Mpv,
};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};

enum EngineCommand {
    Load { url: String, auto_play: bool },
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    GetState(oneshot::Sender<PlayerState>),
}

#[derive(Clone)]
pub struct MpvPlayer {
    cmd_tx: mpsc::Sender<EngineCommand>,
    event_tx: broadcast::Sender<PlayerEvent>,
}

impl MpvPlayer {
    pub fn new(config: MpvConfig) -> anyhow::Result<Self> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let (event_tx, _) = broadcast::channel(128);
        let event_tx_actor = event_tx.clone();

        std::thread::Builder::new()
            .name("mpv-actor".to_string())
            .spawn(move || {
                let mut mpv = Mpv::new().expect("Failed to initialize MPV");

                if let Err(e) = mpv.set_property("vo", "null") {
                    log::warn!("MPV: Failed to set vo=null: {}", e);
                }
                if let Err(e) = mpv.set_property("title", "Aether Player") {
                    log::warn!("MPV: Failed to set title: {}", e);
                }

                if config.hardware_decoding {
                    if let Err(e) = mpv.set_property("hwdec", "auto") {
                        log::warn!("MPV: Failed to enable hwdec: {}", e);
                    }
                }

                if let Some(device) = &config.audio_device {
                    if let Err(e) = mpv.set_property("audio-device", device.clone()) {
                        log::error!("MPV: Failed to set audio device '{}': {}", device, e);
                    }
                }

                if let Err(e) = mpv.observe_property("time-pos", libmpv2::Format::Double, 0) {
                    log::warn!("MPV: Failed to observe time-pos: {}", e);
                }
                if let Err(e) = mpv.observe_property("pause", libmpv2::Format::Flag, 0) {
                    log::warn!("MPV: Failed to observe pause: {}", e);
                }
                if let Err(e) = mpv.observe_property("duration", libmpv2::Format::Double, 0) {
                    log::warn!("MPV: Failed to observe duration: {}", e);
                }
                if let Err(e) = mpv.observe_property("volume", libmpv2::Format::Double, 0) {
                    log::warn!("MPV: Failed to observe volume: {}", e);
                }

                let mut cached_state = PlayerState::default();

                'actor: loop {
                    while let Some(Ok(ev)) = mpv.wait_event(0.01) {
                        match ev {
                            Event::PropertyChange { name, change, .. } => match name {
                                "time-pos" => {
                                    if let PropertyData::Double(v) = change {
                                        cached_state.position = v;
                                        let _ = event_tx_actor.send(PlayerEvent::TimeUpdate(v));
                                    }
                                }
                                "pause" => {
                                    if let PropertyData::Flag(v) = change {
                                        cached_state.paused = v;
                                        let _ = event_tx_actor.send(if v {
                                            PlayerEvent::Paused
                                        } else {
                                            PlayerEvent::Playing
                                        });
                                    }
                                }
                                "duration" => {
                                    if let PropertyData::Double(v) = change {
                                        cached_state.duration = v;
                                        let _ = event_tx_actor.send(PlayerEvent::DurationChange(v));
                                    }
                                }
                                "volume" => {
                                    if let PropertyData::Double(v) = change {
                                        cached_state.volume = (v / 100.0) as f32;
                                    }
                                }
                                _ => {}
                            },
                            Event::EndFile(_) => {
                                let _ = event_tx_actor.send(PlayerEvent::Ended);
                            }
                            Event::Shutdown => break 'actor,
                            _ => {}
                        }
                    }

                    match cmd_rx.try_recv() {
                        Ok(cmd) => match cmd {
                            EngineCommand::Load { url, auto_play } => {
                                let mode = if auto_play { "replace" } else { "append-play" };
                                if let Err(e) = mpv.command("loadfile", &[&url, mode]) {
                                    log::error!("MPV Load Error: {}", e);
                                }
                            }
                            EngineCommand::Play => {
                                let _ = mpv.set_property("pause", false);
                            }
                            EngineCommand::Pause => {
                                let _ = mpv.set_property("pause", true);
                            }
                            EngineCommand::Stop => {
                                let _ = mpv.command("stop", &[]);
                            }
                            EngineCommand::Seek(t) => {
                                let _ = mpv.command("seek", &[&t.to_string(), "absolute"]);
                            }
                            EngineCommand::SetVolume(v) => {
                                let _ = mpv.set_property("volume", (v * 100.0) as i64);
                            }
                            EngineCommand::GetState(tx) => {
                                let _ = tx.send(cached_state.clone());
                            }
                        },
                        Err(mpsc::error::TryRecvError::Empty) => {
                            std::thread::sleep(Duration::from_millis(16));
                        }
                        Err(mpsc::error::TryRecvError::Disconnected) => break 'actor,
                    }
                }
            })?;

        Ok(Self { cmd_tx, event_tx })
    }

    async fn send(&self, cmd: EngineCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| "Audio engine actor died".to_string())
    }
}

#[async_trait]
impl AudioEngine for MpvPlayer {
    async fn load(&self, stream: AudioStream, auto_play: bool) -> Result<(), String> {
        match stream {
            AudioStream::Url(url) => self.send(EngineCommand::Load { url, auto_play }).await,
            AudioStream::Bytes(_) => Err(
                "MpvPlayer: Raw byte streams are not supported in this configuration.".to_string(),
            ),
        }
    }

    async fn play(&self) -> Result<(), String> {
        self.send(EngineCommand::Play).await
    }
    async fn pause(&self) -> Result<(), String> {
        self.send(EngineCommand::Pause).await
    }
    async fn stop(&self) -> Result<(), String> {
        self.send(EngineCommand::Stop).await
    }
    async fn seek(&self, seconds: f64) -> Result<(), String> {
        self.send(EngineCommand::Seek(seconds)).await
    }
    async fn set_volume(&self, vol: f32) -> Result<(), String> {
        self.send(EngineCommand::SetVolume(vol)).await
    }

    async fn get_state(&self) -> PlayerState {
        let (tx, rx) = oneshot::channel();
        if self.cmd_tx.send(EngineCommand::GetState(tx)).await.is_ok() {
            rx.await.unwrap_or_default()
        } else {
            PlayerState::default()
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
        self.event_tx.subscribe()
    }
}
