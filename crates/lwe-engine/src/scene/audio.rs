//! CPU-prepared, immutable Scene audio and its reversible runtime ownership.
//! No package reads, extraction, or media paths are delegated to libmpv.

use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use lwe_scene::{SceneAssets, SceneResource};
use serde_json::{Map, Value};

use super::unsupported;
use crate::mpv::MpvAudioPlayer;

const MAX_SOUNDS: usize = 8;
const MAX_CLIP_BYTES: u64 = 64 * 1024 * 1024;
const MAX_AUDIO_BYTES: u64 = 128 * 1024 * 1024;
const READ_CHUNK: usize = 64 * 1024;

#[derive(Clone)]
pub(crate) struct PreparedAudio {
    clip: Arc<AudioClip>,
    pub(crate) volume: f64,
}

impl std::fmt::Debug for PreparedAudio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedAudio")
            .field("bytes", &self.clip.bytes)
            .field("volume", &self.volume)
            .finish()
    }
}

impl PreparedAudio {
    pub(crate) fn clip_identity(&self) -> usize {
        Arc::as_ptr(&self.clip) as usize
    }

    pub(crate) fn payload_bytes(&self) -> u64 {
        self.clip.bytes
    }
}

struct AudioClip {
    source: Arc<File>,
    bytes: u64,
}

/// Call only from CPU preparation, after the project-property resolver. This
/// subset admits ordinary, initially audible loop sounds containing one FLAC.
/// Multi-track selection, random intervals, scripts, and spatial sound remain
/// explicit capability failures.
pub(crate) fn prepare_audio(assets: &SceneAssets, scene: &Value) -> Result<Vec<PreparedAudio>> {
    let objects = scene
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| unsupported("audio", "Scene audio needs an object array"))?;
    let mut declarations = Vec::new();
    let mut resources = BTreeMap::<String, SceneResource>::new();
    let mut total = 0_u64;
    for object in objects
        .iter()
        .filter(|object| object.get("sound").is_some())
    {
        ensure!(
            declarations.len() < MAX_SOUNDS,
            "Scene sound layer limit exceeded"
        );
        let (name, volume) = declaration(object)?;
        if !resources.contains_key(name) {
            let resource = assets.resource(name)?;
            ensure!(
                resource.len() <= MAX_CLIP_BYTES,
                "Scene audio clip byte limit exceeded"
            );
            total = total
                .checked_add(resource.len())
                .context("Scene audio byte total overflow")?;
            ensure!(
                total <= MAX_AUDIO_BYTES,
                "Scene audio total byte limit exceeded"
            );
            validate_flac(&resource)?;
            resources.insert(name.to_string(), resource);
        }
        declarations.push((name.to_string(), volume));
    }
    // All declarations, resource sizes, and headers pass before any payload is
    // copied. Duplicate sound paths share one sealed anonymous descriptor.
    let clips = resources
        .into_iter()
        .map(|(name, resource)| {
            let bytes = resource.len();
            Ok((
                name,
                Arc::new(AudioClip {
                    source: Arc::new(sealed_source(&resource)?),
                    bytes,
                }),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(declarations
        .into_iter()
        .map(|(name, volume)| PreparedAudio {
            clip: clips[&name].clone(),
            volume,
        })
        .collect())
}

fn declaration(object: &Value) -> Result<(&str, f64)> {
    let object = object
        .as_object()
        .ok_or_else(|| unsupported("audio", "Invalid sound object"))?;
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "id" | "name"
                | "sound"
                | "playbackmode"
                | "volume"
                | "mintime"
                | "maxtime"
                | "startsilent"
                | "muteineditor"
                | "solid"
                | "disablepropagation"
        ) {
            return Err(unsupported(
                "audio",
                format!("Unsupported sound field {key}"),
            ));
        }
    }
    if !object
        .get("id")
        .is_some_and(|value| value.as_u64().is_some())
        || !object.get("name").is_none_or(Value::is_string)
    {
        return Err(unsupported("audio", "Invalid sound identity"));
    }
    let sounds = object
        .get("sound")
        .and_then(Value::as_array)
        .ok_or_else(|| unsupported("audio", "Sound must be a single-file array"))?;
    if sounds.len() != 1 || object.get("playbackmode").and_then(Value::as_str) != Some("loop") {
        return Err(unsupported(
            "audio",
            "Only a single-file loop sound is supported",
        ));
    }
    let name = sounds[0]
        .as_str()
        .filter(|name| name.ends_with(".flac"))
        .ok_or_else(|| unsupported("audio", "Only prepared FLAC sound resources are supported"))?;
    require_bool(object, "startsilent", false)?;
    require_bool(object, "muteineditor", false)?;
    require_bool(object, "disablepropagation", false)?;
    require_bool(object, "solid", true)?;
    // The verified native single-file loop branch sets loop=true/countdown=0;
    // only the random mode consumes these intervals. Still validate the fields.
    let minimum = finite_number(object, "mintime", 1.0)?;
    let maximum = finite_number(object, "maxtime", 5.0)?;
    if minimum < 0.0 || maximum < minimum || maximum > 120.0 {
        return Err(unsupported("audio", "Unsupported sound timing interval"));
    }
    let volume = finite_number(object, "volume", 1.0)?;
    if !(0.0..=1.0).contains(&volume) {
        return Err(unsupported(
            "audio",
            "Sound volume must be between zero and one",
        ));
    }
    Ok((name, volume))
}

fn require_bool(object: &Map<String, Value>, key: &str, expected: bool) -> Result<()> {
    if !object
        .get(key)
        .is_none_or(|value| value.as_bool() == Some(expected))
    {
        return Err(unsupported(
            "audio",
            format!("Unsupported sound setting {key}"),
        ));
    }
    Ok(())
}

fn finite_number(object: &Map<String, Value>, key: &str, default: f64) -> Result<f64> {
    match object.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|number| number.is_finite())
            .ok_or_else(|| {
                unsupported(
                    "audio",
                    format!("Sound {key} must be a resolved finite number"),
                )
            }),
    }
}

fn validate_flac(resource: &SceneResource) -> Result<()> {
    ensure!(resource.len() >= 42, "Scene FLAC resource is truncated");
    let header = resource.read_range(0, 42, 42)?;
    ensure!(
        &header[..4] == b"fLaC" && header[4] & 0x7f == 0 && header[5..8] == [0, 0, 34],
        "Scene audio requires a FLAC STREAMINFO header"
    );
    let packed = u64::from_be_bytes(header[18..26].try_into().unwrap());
    let rate = packed >> 44;
    let channels = ((packed >> 41) & 7) + 1;
    let bits = ((packed >> 36) & 31) + 1;
    let samples = packed & ((1_u64 << 36) - 1);
    ensure!(
        (8_000..=192_000).contains(&rate)
            && channels <= 2
            && (8..=32).contains(&bits)
            && samples > 0
            && samples <= rate * 6 * 3600,
        "Unsupported FLAC stream dimensions or duration"
    );
    Ok(())
}

fn sealed_source(resource: &SceneResource) -> Result<File> {
    let name = CString::new("lwe-scene-audio")?;
    let descriptor =
        unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING) };
    ensure!(
        descriptor >= 0,
        "Unable to create prepared Scene audio: {}",
        std::io::Error::last_os_error()
    );
    let mut source = unsafe { File::from_raw_fd(descriptor) };
    let mut offset = 0;
    while offset < resource.len() {
        let count = (resource.len() - offset).min(READ_CHUNK as u64) as usize;
        source
            .write_all(&resource.read_range(offset, count, READ_CHUNK)?)
            .context("Copy prepared Scene audio")?;
        offset += count as u64;
    }
    source
        .seek(SeekFrom::Start(0))
        .context("Rewind prepared Scene audio")?;
    ensure!(
        unsafe { libc::fchmod(source.as_raw_fd(), 0o400) } == 0,
        "Unable to protect prepared Scene audio"
    );
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    ensure!(
        unsafe { libc::fcntl(source.as_raw_fd(), libc::F_ADD_SEALS, seals) } == 0,
        "Unable to seal prepared Scene audio"
    );
    Ok(source)
}

/// Candidate players are always paused and muted. The owner activates them only
/// after atomic assignment persistence and deactivates the previous generation
/// first. Keeping a deactivated previous player preserves its rollback timer.
pub(crate) struct SceneAudio {
    players: Vec<(MpvAudioPlayer, f64)>,
    active: bool,
    paused: bool,
    master_volume: f64,
}

impl SceneAudio {
    pub(crate) fn new(prepared: &[PreparedAudio]) -> Result<Self> {
        let players = prepared
            .iter()
            .map(|sound| {
                Ok((
                    MpvAudioPlayer::new(sound.clip.source.clone())?,
                    sound.volume,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            players,
            active: false,
            paused: true,
            master_volume: 1.0,
        })
    }

    pub(crate) fn poll_ready(&mut self) -> Result<bool> {
        let mut ready = true;
        for (player, _) in &mut self.players {
            ready &= player.poll_ready()?;
        }
        Ok(ready)
    }

    pub(crate) fn activate(&mut self, paused: bool, master_volume: f64) -> Result<()> {
        ensure!(self.poll_ready()?, "Candidate Scene audio is not ready");
        ensure!(
            master_volume.is_finite() && (0.0..=1.0).contains(&master_volume),
            "Invalid Scene audio master volume"
        );
        self.active = true;
        self.paused = paused;
        self.master_volume = master_volume;
        self.apply_policy()
    }

    pub(crate) fn deactivate(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        self.apply_policy()
    }

    pub(crate) fn set_paused(&mut self, paused: bool) -> Result<()> {
        if self.paused == paused {
            return Ok(());
        }
        self.paused = paused;
        self.apply_policy()
    }

    pub(crate) fn set_master_volume(&mut self, volume: f64) -> Result<()> {
        ensure!(
            volume.is_finite() && (0.0..=1.0).contains(&volume),
            "Invalid Scene audio master volume"
        );
        if self.master_volume == volume {
            return Ok(());
        }
        self.master_volume = volume;
        self.apply_policy()
    }

    fn apply_policy(&mut self) -> Result<()> {
        let mut failure = None;
        for (player, volume) in &mut self.players {
            if let Err(error) = player.set_policy(
                self.active,
                self.paused,
                layer_gain(*volume, self.master_volume),
            ) {
                failure.get_or_insert(error);
            }
        }
        if let Some(error) = failure {
            self.active = false;
            for (player, _) in &mut self.players {
                let _ = player.set_policy(false, true, 0.0);
            }
            return Err(error);
        }
        Ok(())
    }
}

fn layer_gain(volume: f64, master_volume: f64) -> f64 {
    // Native sound updates square object volume before multiplying global gain.
    volume * volume * master_volume
}

pub(crate) fn prepared_audio_bytes(prepared: &[PreparedAudio]) -> u64 {
    let mut clips = Vec::new();
    let mut total = 0;
    for sound in prepared {
        if !clips.iter().any(|clip| Arc::ptr_eq(clip, &sound.clip)) {
            total += sound.clip.bytes;
            clips.push(sound.clip.clone());
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::os::unix::fs::FileExt;
    use std::path::PathBuf;

    struct AuthoredResources(PathBuf);
    impl AuthoredResources {
        fn new() -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "lwe-authored-audio-{}-{unique}",
                std::process::id()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn streaminfo(&self, name: &str, size: u64) {
            let mut bytes = vec![0_u8; 42];
            bytes[..8].copy_from_slice(b"fLaC\x00\x00\x00\x22");
            let packed = (48_000_u64 << 44) | (1_u64 << 41) | (23_u64 << 36) | 48_000;
            bytes[18..26].copy_from_slice(&packed.to_be_bytes());
            let path = self.0.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(&bytes).unwrap();
            file.set_len(size.max(42)).unwrap();
        }
        fn assets(&self) -> SceneAssets {
            SceneAssets::open(&self.0, None).unwrap()
        }
    }
    impl Drop for AuthoredResources {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn sound(id: u64, name: &str, volume: f64) -> Value {
        json!({"id":id,"name":"Authored sound","sound":[name],"playbackmode":"loop","volume":volume,"mintime":1.0,"maxtime":5.0,"startsilent":false,"muteineditor":false,"solid":true,"disablepropagation":false})
    }

    #[test]
    fn preparation_preserves_declared_volumes_and_seals_shared_payload() {
        let resources = AuthoredResources::new();
        resources.streaminfo("sounds/authored.flac", 42);
        let prepared = prepare_audio(&resources.assets(), &json!({"objects":[sound(1,"sounds/authored.flac",1.0),sound(2,"sounds/authored.flac",0.0)]})).unwrap();
        assert_eq!(prepared.len(), 2);
        assert_eq!((prepared[0].volume, prepared[1].volume), (1.0, 0.0));
        assert_eq!(prepared[0].clip_identity(), prepared[1].clip_identity());
        assert_eq!(prepared_audio_bytes(&prepared), 42);
        assert_eq!(prepared[0].payload_bytes(), 42);
        let source = &prepared[0].clip.source;
        let required =
            libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
        assert_eq!(
            unsafe { libc::fcntl(source.as_raw_fd(), libc::F_GET_SEALS) } & required,
            required
        );
        assert_eq!(
            source.write_at(&[0], 0).unwrap_err().raw_os_error(),
            Some(libc::EPERM)
        );
        assert!(source.set_len(0).is_err());
    }

    #[test]
    fn selectors_scripts_unknown_fields_and_unevaluated_properties_stay_blocked() {
        let resources = AuthoredResources::new();
        resources.streaminfo("sounds/authored.flac", 42);
        let good = sound(1, "sounds/authored.flac", 1.0);
        let mut cases = Vec::new();
        for (key, value) in [
            ("playbackmode", json!("random")),
            ("startsilent", json!(true)),
            ("volume", json!({"user":"unevaluated","value":1})),
            ("volume", json!(-1)),
            ("spatialization", json!(true)),
            ("script", json!("never execute")),
            (
                "sound",
                json!(["sounds/authored.flac", "sounds/authored.flac"]),
            ),
        ] {
            let mut object = good.clone();
            object[key] = value;
            cases.push(object);
        }
        for object in cases {
            assert!(prepare_audio(&resources.assets(), &json!({"objects":[object]})).is_err());
        }
        let unsafe_path = sound(1, "../authored.flac", 1.0);
        let error =
            prepare_audio(&resources.assets(), &json!({"objects":[unsafe_path]})).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<lwe_scene::SceneAssetError>(),
            Some(lwe_scene::SceneAssetError::UnsafePath { .. })
        ));
    }

    #[test]
    fn resource_and_layer_budgets_reject_before_payload_copy() {
        let resources = AuthoredResources::new();
        resources.streaminfo("oversize.flac", MAX_CLIP_BYTES + 1);
        assert!(
            prepare_audio(
                &resources.assets(),
                &json!({"objects":[sound(1,"oversize.flac",1.0)]})
            )
            .unwrap_err()
            .to_string()
            .contains("clip byte limit")
        );
        let mut objects = Vec::new();
        for index in 0..3 {
            let name = format!("clip-{index}.flac");
            resources.streaminfo(&name, 45 * 1024 * 1024);
            objects.push(sound(index, &name, 1.0));
        }
        assert!(
            prepare_audio(&resources.assets(), &json!({"objects":objects}))
                .unwrap_err()
                .to_string()
                .contains("total byte limit")
        );
        resources.streaminfo("small.flac", 42);
        let objects: Vec<_> = (0..9)
            .map(|index| sound(index, "small.flac", 1.0))
            .collect();
        assert!(
            prepare_audio(&resources.assets(), &json!({"objects":objects}))
                .unwrap_err()
                .to_string()
                .contains("layer limit")
        );
    }

    #[test]
    fn empty_scene_audio_is_ready_without_creating_a_player() {
        let mut audio = SceneAudio::new(&[]).unwrap();
        assert!(audio.poll_ready().unwrap());
        audio.activate(false, 1.0).unwrap();
        audio.set_paused(true).unwrap();
        audio.deactivate().unwrap();
        assert!(audio.players.is_empty());
    }

    #[test]
    fn native_sound_volume_curve_squares_the_layer_before_global_gain() {
        assert_eq!(layer_gain(1.0, 1.0), 1.0);
        assert_eq!(layer_gain(0.0, 1.0), 0.0);
        assert_eq!(layer_gain(0.5, 1.0), 0.25);
        assert_eq!(layer_gain(0.5, 0.5), 0.125);
    }
}
