use crate::models::config::MpvConfig;
use crate::models::entities::PlayerEvent;
use crate::models::{AudioDevice, PlayerState};
use crate::traits::{AudioEngine, AudioStream};
use async_trait::async_trait;
use libmpv2::{
    events::{Event, PropertyData},
    Mpv,
};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(serde::Deserialize)]
struct MpvDeviceEntry {
    name: String,
    description: String,
}

enum EngineCommand {
    Load { url: String, auto_play: bool },
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    GetState(oneshot::Sender<PlayerState>),
    GetAudioDevices(oneshot::Sender<Result<Vec<AudioDevice>, String>>),
    SetAudioDevice(Option<String>),
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
                            Event::EndFile(0) => {
                                let _ = event_tx_actor.send(PlayerEvent::Ended);
                            }
                            Event::Shutdown => break 'actor,
                            _ => {}
                        }
                    }

                    match cmd_rx.try_recv() {
                        Ok(cmd) => match cmd {
                            EngineCommand::Load { url, auto_play } => {
                                if let Err(e) = mpv.command("loadfile", &[&url, "replace"]) {
                                    log::error!("MPV Load Error: {}", e);
                                } else {
                                    let should_pause = !auto_play;
                                    let _ = mpv.set_property("pause", should_pause);
                                    cached_state.paused = should_pause;

                                    let event = if should_pause {
                                        PlayerEvent::Paused
                                    } else {
                                        PlayerEvent::Playing
                                    };
                                    let _ = event_tx_actor.send(event);
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
                            EngineCommand::GetAudioDevices(tx) => {
                                let res = match mpv.get_property::<String>("audio-device-list") {
                                    Ok(json) => {
                                        match serde_json::from_str::<Vec<MpvDeviceEntry>>(&json) {
                                            Ok(devices) => Ok(devices
                                                .into_iter()
                                                .map(|d| AudioDevice {
                                                    id: d.name.clone(),
                                                    name: d.description,
                                                    is_default: d.name == "auto",
                                                    is_current: match mpv
                                                        .get_property::<String>("audio-device")
                                                    {
                                                        Ok(current) => d.name == current,
                                                        Err(_) => false,
                                                    },
                                                })
                                                .collect()),
                                            Err(e) => {
                                                Err(format!("Failed to parse device list: {}", e))
                                            }
                                        }
                                    }
                                    Err(e) => Err(format!("MPV Error: {}", e)),
                                };
                                let _ = tx.send(res);
                            }
                            EngineCommand::SetAudioDevice(id) => {
                                let val = id.unwrap_or_else(|| "auto".to_string());
                                if let Err(e) = mpv.set_property("audio-device", val.clone()) {
                                    log::error!("MPV: Failed to set audio device '{}': {}", val, e);
                                }
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

    async fn get_audio_devices(&self) -> Result<Vec<AudioDevice>, String> {
        let (tx, rx) = oneshot::channel();
        self.send(EngineCommand::GetAudioDevices(tx)).await?;
        rx.await.map_err(|_| "Actor dropped".to_string())?
    }

    async fn set_audio_device(&self, device_id: Option<String>) -> Result<(), String> {
        self.send(EngineCommand::SetAudioDevice(device_id)).await
    }

    fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
        self.event_tx.subscribe()
    }
}
