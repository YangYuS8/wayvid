//! Bounded particle preparation and CPU simulation, independent of graphics and
//! wall time. Supported components execute here; original-engine visual parity
//! remains a separate acceptance test.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use lwe_scene::AnimationMetadata;
use serde_json::{Map, Value};

use super::unsupported;

const MAX_SYSTEMS: usize = 32;
const MAX_DEPTH: usize = 8;
const MAX_COMPONENTS: usize = 64;
const MAX_PARTICLES: usize = 16_384;
const MAX_VALUE: f32 = 1_000_000.0;
const STEP: f64 = 1.0 / 60.0;
const MAX_STEPS: usize = 600;
const MAX_VERTICES: usize = 400_000;
const MAX_WORK: usize = 100_000;

#[derive(Debug, Clone)]
pub(crate) struct PreparedParticles {
    pub systems: Vec<ParticleSystem>,
    pub transform: ParticleTransform,
    pub modifiers: ParticleModifiers,
    pub particle_capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleTransform {
    pub origin: [f32; 3],
    /// Static Scene JSON stores radians, unlike the JavaScript angles API.
    pub angles: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleModifiers {
    pub alpha: f32,
    pub count: f32,
    pub lifetime: f32,
    pub rate: f32,
    pub speed: f32,
    pub size: f32,
    pub brightness: f32,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParticleSystem {
    pub path: String,
    pub material: String,
    pub max_count: usize,
    pub start_time: f32,
    pub world_space: bool,
    pub animation_mode: AnimationMode,
    pub sequence_multiplier: f32,
    pub frame_blending: bool,
    pub emitters: Vec<Emitter>,
    pub initializers: Vec<Initializer>,
    pub operators: Vec<Operator>,
    pub renderer: ParticleRenderer,
    pub children: Vec<ParticleChild>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimationMode {
    Sequence,
    RandomFrame,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RandomRange {
    pub min: f32,
    pub max: f32,
    pub exponent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EmitterShape {
    Box { min: [f32; 3], max: [f32; 3] },
    Sphere { min: f32, max: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Emitter {
    pub shape: EmitterShape,
    pub origin: [f32; 3],
    pub directions: Option<[f32; 3]>,
    pub rate: f32,
    pub instantaneous: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Initializer {
    Lifetime(RandomRange),
    Size(RandomRange),
    Alpha(RandomRange),
    Color {
        min: [f32; 3],
        max: [f32; 3],
    },
    Velocity {
        min: [f32; 3],
        max: [f32; 3],
    },
    Rotation(RandomRange),
    AngularVelocity(RandomRange),
    TurbulentVelocity {
        speed: RandomRange,
        phase: RandomRange,
        scale: f32,
        time_scale: f32,
        offset: f32,
        forward: [f32; 2],
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Oscillation {
    pub frequency: RandomRange,
    pub phase: RandomRange,
    pub scale: RandomRange,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Operator {
    SizeChange {
        start_time: f32,
        end_time: f32,
        start: f32,
        end: f32,
    },
    ColorChange {
        start_time: f32,
        end_time: f32,
        start: [f32; 3],
        end: [f32; 3],
    },
    Movement {
        gravity: [f32; 3],
        drag: f32,
    },
    AlphaFade {
        fade_in: f32,
        fade_out: f32,
    },
    OscillateAlpha(Oscillation),
    OscillateSize(Oscillation),
    AngularMovement {
        force: [f32; 3],
        drag: f32,
    },
    Turbulence {
        speed: RandomRange,
        time_scale: f32,
        spatial_scale: Option<f32>,
        phase: Option<RandomRange>,
        mask: [f32; 3],
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ParticleRenderer {
    Sprite {
        world_orientation: bool,
    },
    SpriteTrail {
        length: f32,
        min_length: f32,
        max_length: f32,
    },
    RopeTrail {
        length: f32,
        segments: usize,
        subdivision: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChildMode {
    Static,
    Spawn,
    Follow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleChild {
    pub system: usize,
    pub mode: ChildMode,
    pub max_instances: usize,
    pub scale: [f32; 3],
}

impl PreparedParticles {
    /// `read_json` must use the same property resolver and safe asset reader as
    /// Scene preparation. No resource is read after this function returns.
    pub fn prepare(
        object: &Value,
        read_json: &mut impl FnMut(&str) -> Result<Value>,
    ) -> Result<Self> {
        let o = object_map(object, "particle object")?;
        known_keys(
            o,
            &[
                "id",
                "name",
                "particle",
                "origin",
                "angles",
                "scale",
                "visible",
                "solid",
                "disablepropagation",
                "locktransforms",
                "instanceoverride",
                "parallaxDepth",
            ],
        )?;
        optional_id(o)?;
        optional_string(o, "name")?;
        if let Some(depth) = o.get("parallaxDepth") {
            let values = depth
                .as_str()
                .context("particle parallax depth must be a vector")?
                .split_whitespace()
                .map(str::parse::<f32>)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(
                values == [1.0, 1.0],
                "non-default particle parallax depth is unsupported"
            );
        }
        for key in ["visible", "solid", "disablepropagation", "locktransforms"] {
            if let Some(value) = o.get(key) {
                ensure!(value.is_boolean(), "particle {key} must be boolean");
            }
        }
        ensure!(
            o.get("disablepropagation").and_then(Value::as_bool) != Some(true),
            "particle propagation override is unsupported"
        );
        let path = resource_path(required_string(o, "particle")?)?;
        let modifiers = parse_modifiers(o.get("instanceoverride"))?;
        let transform = ParticleTransform {
            origin: vector(o, "origin", [0.0; 3])?,
            angles: vector(o, "angles", [0.0; 3])?,
            scale: vector(o, "scale", [1.0; 3])?,
        };
        ensure!(
            transform.angles[0] == 0.0 && transform.angles[1] == 0.0,
            "3D particle orientation is unsupported"
        );
        ensure!(
            transform.origin[2] == 0.0,
            "3D particle origin is unsupported"
        );
        ensure!(
            transform
                .scale
                .iter()
                .all(|v| *v != 0.0 && v.abs() <= 100.0),
            "invalid particle scale"
        );
        let mut loader = Loader {
            read_json,
            systems: Vec::new(),
            stack: BTreeSet::new(),
            capacity: 0,
        };
        loader.load(&path, 1, 0, true)?;
        Ok(Self {
            systems: loader.systems,
            transform,
            modifiers,
            particle_capacity: loader.capacity,
        })
    }

    pub fn material_paths(&self) -> impl Iterator<Item = &str> {
        self.systems.iter().map(|system| system.material.as_str())
    }

    /// Check resource and work limits before allocating simulation state.
    /// Original-engine visual parity remains a separate acceptance test.
    pub fn validate_runtime(&self) -> Result<()> {
        self.budgets().map(|_| ())
    }

    pub fn max_vertices(&self) -> Result<usize> {
        self.budgets().map(|(vertices, _)| vertices)
    }

    pub fn max_work(&self) -> Result<usize> {
        self.budgets().map(|(_, work)| work)
    }

    fn budgets(&self) -> Result<(usize, usize)> {
        ensure!(
            !self.systems.is_empty() && self.modifiers.lifetime > 0.0,
            "particle runtime requires positive lifetime"
        );
        let mut vertices = 0usize;
        let mut work = 0usize;
        let mut instances = vec![0usize; self.systems.len()];
        instances[0] = 1;
        for (index, system) in self.systems.iter().enumerate() {
            // Event children still require zero warmup; static children are
            // initialized once and pre-rolled independently with their parent.
            ensure!(
                index == 0
                    || system.start_time == 0.0
                    || self.systems.iter().any(|parent| parent
                        .children
                        .iter()
                        .any(|child| child.system == index && child.mode == ChildMode::Static)),
                "event child particle warmup is unsupported"
            );
            let count = system
                .max_count
                .checked_mul(instances[index])
                .context("particle draw budget overflow")?;
            ensure!(system.initializers.iter().any(|i| matches!(i, Initializer::Lifetime(range) if range.min > 0.0 && range.max > 0.0)), "particle runtime requires positive lifetime");
            let per_particle = match system.renderer {
                ParticleRenderer::Sprite { .. } | ParticleRenderer::SpriteTrail { .. } => 6,
                ParticleRenderer::RopeTrail {
                    segments,
                    subdivision,
                    ..
                } => (segments - 1) * (subdivision + 1) * 6,
            };
            vertices = vertices
                .checked_add(
                    count
                        .checked_mul(per_particle)
                        .context("particle vertex budget overflow")?,
                )
                .context("particle vertex budget overflow")?;
            work = work
                .checked_add(
                    count
                        .checked_mul(
                            system
                                .operators
                                .iter()
                                .map(|op| {
                                    if matches!(op, Operator::Turbulence { .. }) {
                                        128
                                    } else {
                                        1
                                    }
                                })
                                .sum::<usize>()
                                .saturating_add(
                                    system
                                        .initializers
                                        .iter()
                                        .filter(|initializer| {
                                            matches!(
                                                initializer,
                                                Initializer::TurbulentVelocity { .. }
                                            )
                                        })
                                        .count()
                                        * 128,
                                )
                                .max(1),
                        )
                        .context("particle work budget overflow")?,
                )
                .context("particle work budget overflow")?;
            for child in &system.children {
                instances[child.system] = instances[index]
                    .checked_mul(child.max_instances)
                    .context("particle child budget overflow")?;
            }
        }
        ensure!(
            vertices <= MAX_VERTICES && work <= MAX_WORK,
            "particle simulation or geometry exceeds budget"
        );
        Ok((vertices, work))
    }
}

struct Loader<'a, F> {
    read_json: &'a mut F,
    systems: Vec<ParticleSystem>,
    stack: BTreeSet<String>,
    capacity: usize,
}

impl<F: FnMut(&str) -> Result<Value>> Loader<'_, F> {
    fn load(
        &mut self,
        path: &str,
        instances: usize,
        depth: usize,
        static_context: bool,
    ) -> Result<usize> {
        ensure!(depth <= MAX_DEPTH, "particle child depth exceeds budget");
        ensure!(
            self.systems.len() < MAX_SYSTEMS,
            "particle system count exceeds budget"
        );
        ensure!(
            self.stack.insert(path.to_owned()),
            "particle child resource cycle"
        );
        let value =
            (self.read_json)(path).with_context(|| format!("read particle resource {path}"))?;
        let o = object_map(&value, "particle resource")?;
        known_keys(
            o,
            &[
                "material",
                "maxcount",
                "starttime",
                "flags",
                "animationmode",
                "sequencemultiplier",
                "controlpoint",
                "emitter",
                "initializer",
                "operator",
                "renderer",
                "children",
            ],
        )?;
        let material = resource_path(required_string(o, "material")?)?;
        let max_count = integer(o, "maxcount", 0, MAX_PARTICLES)?;
        ensure!(max_count > 0, "particle maxcount must be positive");
        let cost = max_count
            .checked_mul(instances)
            .context("particle capacity overflow")?;
        self.capacity = self
            .capacity
            .checked_add(cost)
            .context("particle capacity overflow")?;
        ensure!(
            self.capacity <= MAX_PARTICLES,
            "particle tree capacity exceeds budget"
        );
        let start_time = number(o, "starttime", 0.0)?;
        ensure!(
            (0.0..=30.0).contains(&start_time),
            "particle warmup exceeds budget"
        );
        let flags = integer(o, "flags", 0, u8::MAX as usize)?;
        // Native bytecode builder sets bit 2 for randomframe, disabling frame
        // blending. Other flags remain outside this 2D subset.
        ensure!(flags <= 3, "unsupported particle system flags");
        let animation_mode = match o.get("animationmode") {
            None | Some(Value::Null) => AnimationMode::Sequence,
            Some(value) if value.as_str() == Some("sequence") => AnimationMode::Sequence,
            Some(value) if value.as_str() == Some("randomframe") => AnimationMode::RandomFrame,
            _ => {
                return Err(unsupported(
                    "particle",
                    "unsupported particle animation mode",
                ));
            }
        };
        let sequence_multiplier = number(o, "sequencemultiplier", 1.0)?;
        ensure!(
            (0.0..=100.0).contains(&sequence_multiplier),
            "particle sequence multiplier exceeds budget"
        );
        parse_controlpoints(o)?;
        let emitters = components(o, "emitter")?
            .iter()
            .map(parse_emitter)
            .collect::<Result<Vec<_>>>()?;
        ensure!(!emitters.is_empty(), "particle emitter is missing");
        let initializers = components(o, "initializer")?
            .iter()
            .map(parse_initializer)
            .collect::<Result<Vec<_>>>()?;
        let operators = components(o, "operator")?
            .iter()
            .map(parse_operator)
            .collect::<Result<Vec<_>>>()?;
        let renderers = components(o, "renderer")?;
        ensure!(
            renderers.len() == 1,
            "exactly one particle renderer is required"
        );
        let renderer = parse_renderer(&renderers[0])?;
        let index = self.systems.len();
        self.systems.push(ParticleSystem {
            path: path.to_owned(),
            material,
            max_count,
            start_time,
            world_space: flags & 1 != 0,
            animation_mode,
            sequence_multiplier,
            frame_blending: flags & 2 == 0 && animation_mode == AnimationMode::Sequence,
            emitters,
            initializers,
            operators,
            renderer,
            children: Vec::new(),
        });
        let mut children = Vec::new();
        for child in components(o, "children")? {
            let c = object_map(child, "particle child")?;
            known_keys(c, &["id", "name", "type", "maxcount", "scale"])?;
            optional_id(c)?;
            let child_path = resource_path(required_string(c, "name")?)?;
            let mode = match c.get("type").and_then(Value::as_str).unwrap_or("static") {
                "static" => ChildMode::Static,
                "eventspawn" => ChildMode::Spawn,
                "eventfollow" => ChildMode::Follow,
                _ => return Err(unsupported("particle", "unsupported particle child event")),
            };
            ensure!(
                c.get("type").is_none_or(|value| value.is_string()),
                "particle child type must be a string"
            );
            let max_instances = if mode == ChildMode::Static {
                ensure!(
                    static_context,
                    "static children below event systems are unsupported"
                );
                1
            } else {
                integer(c, "maxcount", 20, MAX_PARTICLES)?
            };
            if mode == ChildMode::Static && c.contains_key("maxcount") {
                integer(c, "maxcount", 20, MAX_PARTICLES)?;
            }
            ensure!(
                max_instances > 0,
                "particle child maxcount must be positive"
            );
            let scale = vector(c, "scale", [1.0; 3])?;
            ensure!(
                scale.iter().all(|v| *v > 0.0 && *v <= 100.0),
                "invalid particle child scale"
            );
            let total_instances = instances
                .checked_mul(max_instances)
                .context("particle child instance overflow")?;
            let system = self.load(
                &child_path,
                total_instances,
                depth + 1,
                static_context && mode == ChildMode::Static,
            )?;
            children.push(ParticleChild {
                system,
                mode,
                max_instances,
                scale,
            });
        }
        self.systems[index].children = children;
        self.stack.remove(path);
        Ok(index)
    }
}

fn parse_modifiers(value: Option<&Value>) -> Result<ParticleModifiers> {
    let empty = Map::new();
    let o = match value {
        None | Some(Value::Null) => &empty,
        Some(value) => object_map(value, "particle instance override")?,
    };
    known_keys(
        o,
        &[
            "id",
            "alpha",
            "count",
            "lifetime",
            "rate",
            "speed",
            "size",
            "brightness",
            "colorn",
        ],
    )?;
    optional_id(o)?;
    let get = |key| {
        let value = number(o, key, 1.0)?;
        ensure!(
            (0.0..=100.0).contains(&value),
            "invalid particle override {key}"
        );
        Ok(value)
    };
    let color = o
        .get("colorn")
        .map(|_| vector(o, "colorn", [1.0; 3]))
        .transpose()?;
    ensure!(
        color.is_none_or(|c| c.iter().all(|v| (0.0..=1.0).contains(v))),
        "invalid normalized particle color"
    );
    Ok(ParticleModifiers {
        alpha: get("alpha")?,
        count: get("count")?,
        lifetime: get("lifetime")?,
        rate: get("rate")?,
        speed: get("speed")?,
        size: get("size")?,
        brightness: get("brightness")?,
        color,
    })
}

fn parse_controlpoints(o: &Map<String, Value>) -> Result<()> {
    let mut ids = BTreeSet::new();
    for point in components(o, "controlpoint")? {
        let p = object_map(point, "particle controlpoint")?;
        known_keys(p, &["id", "flags", "offset"])?;
        let id = integer(p, "id", usize::MAX, 7)?;
        ensure!(ids.insert(id), "duplicate particle controlpoint");
        ensure!(
            integer(p, "flags", 0, u32::MAX as usize)? == 0,
            "dynamic particle controlpoint is unsupported"
        );
        ensure!(
            vector(p, "offset", [0.0; 3])? == [0.0; 3],
            "particle controlpoint offset is unsupported"
        );
    }
    Ok(())
}

fn parse_emitter(value: &Value) -> Result<Emitter> {
    let o = component(value)?;
    reject_audio(o)?;
    known_keys(
        o,
        &[
            "id",
            "name",
            "origin",
            "directions",
            "rate",
            "instantaneous",
            "distancemin",
            "distancemax",
            "flags",
        ],
    )?;
    ensure!(
        matches!(integer(o, "flags", 0, 3)?, 0 | 2),
        "unsupported particle emitter flags"
    );
    let shape = match required_string(o, "name")? {
        "boxrandom" => EmitterShape::Box {
            min: vector(o, "distancemin", [0.0; 3])?,
            max: vector(o, "distancemax", [1.0; 3])?,
        },
        "sphererandom" => EmitterShape::Sphere {
            min: number(o, "distancemin", 0.0)?,
            max: number(o, "distancemax", 128.0)?,
        },
        _ => return Err(unsupported("particle", "unsupported particle emitter")),
    };
    let rate = number(o, "rate", 10.0)?; // Native FUN_140183ca0, not OWE's default of 5.
    ensure!(
        (0.0..=1000.0).contains(&rate),
        "particle emission rate exceeds budget"
    );
    Ok(Emitter {
        shape,
        origin: vector(o, "origin", [0.0; 3])?,
        directions: o
            .get("directions")
            .map(|_| vector(o, "directions", [0.0; 3]))
            .transpose()?,
        rate,
        instantaneous: integer(o, "instantaneous", 0, MAX_PARTICLES)?,
    })
}

fn parse_initializer(value: &Value) -> Result<Initializer> {
    let o = component(value)?;
    reject_audio(o)?;
    let name = required_string(o, "name")?;
    if name == "turbulentvelocityrandom" {
        known_keys(
            o,
            &[
                "id",
                "name",
                "scale",
                "timescale",
                "offset",
                "speedmin",
                "speedmax",
                "phasemin",
                "phasemax",
                "forward",
                "normal",
                "right",
            ],
        )?;
        let normal = vector(o, "normal", [0.0, 0.0, 1.0])?;
        let right = vector(o, "right", normal)?;
        ensure!(
            normal == [0.0, 0.0, 1.0] && right == normal,
            "3D particle turbulence basis is unsupported"
        );
        let forward = vector(o, "forward", [0.0, 1.0, 0.0])?;
        let length = forward[0].hypot(forward[1]);
        ensure!(
            forward[2] == 0.0 && length > 0.0,
            "invalid 2D particle turbulence forward"
        );
        return Ok(Initializer::TurbulentVelocity {
            speed: range(o, "speedmin", "speedmax", 100.0, 250.0)?,
            phase: range(o, "phasemin", "phasemax", 0.0, 0.1)?,
            scale: number(o, "scale", 1.0)?,
            time_scale: number(o, "timescale", 1.0)?,
            offset: number(o, "offset", 0.0)?,
            forward: [forward[0] / length, forward[1] / length],
        });
    }
    known_keys(o, &["id", "name", "min", "max", "exponent"])?;
    Ok(match name {
        "rotationrandom" | "angularvelocityrandom" => {
            let defaults = if name == "rotationrandom" {
                (0.0, std::f32::consts::TAU)
            } else {
                (-5.0, 5.0)
            };
            let min = vector(o, "min", [0.0, 0.0, defaults.0])?;
            let max = vector(o, "max", [0.0, 0.0, defaults.1])?;
            ensure!(
                min[..2] == [0.0, 0.0] && max[..2] == [0.0, 0.0],
                "3D particle rotation is unsupported"
            );
            let exponent = number(o, "exponent", 1.0)?;
            ensure!(
                (0.01..=100.0).contains(&exponent),
                "invalid particle rotation exponent"
            );
            let range = RandomRange {
                min: min[2],
                max: max[2],
                exponent,
            };
            if name == "rotationrandom" {
                Initializer::Rotation(range)
            } else {
                Initializer::AngularVelocity(range)
            }
        }
        "lifetimerandom" => Initializer::Lifetime(range(o, "min", "max", 0.0, 1.0)?),
        "sizerandom" => Initializer::Size(range(o, "min", "max", 0.0, 20.0)?),
        "alpharandom" => Initializer::Alpha(range(o, "min", "max", 0.05, 1.0)?),
        "colorrandom" => {
            ensure!(
                !o.contains_key("exponent"),
                "particle color exponent is unsupported"
            );
            Initializer::Color {
                min: vector(o, "min", [0.0; 3])?,
                max: vector(o, "max", [255.0; 3])?,
            }
        }
        "velocityrandom" => {
            ensure!(
                !o.contains_key("exponent"),
                "particle vector exponent is unsupported"
            );
            Initializer::Velocity {
                min: vector(o, "min", [-32.0, -32.0, 0.0])?,
                max: vector(o, "max", [32.0, 32.0, 0.0])?,
            }
        }
        _ => return Err(unsupported("particle", "unsupported particle initializer")),
    })
}

fn parse_operator(value: &Value) -> Result<Operator> {
    let o = component(value)?;
    reject_audio(o)?;
    let name = required_string(o, "name")?;
    let keys: &[&str] = match name {
        "sizechange" | "colorchange" => &[
            "id",
            "name",
            "starttime",
            "endtime",
            "startvalue",
            "endvalue",
        ],
        "movement" => &["id", "name", "gravity", "drag"],
        "alphafade" => &["id", "name", "fadeintime", "fadeouttime"],
        "oscillatealpha" | "oscillatesize" => &[
            "id",
            "name",
            "frequencymin",
            "frequencymax",
            "phasemin",
            "phasemax",
            "scalemin",
            "scalemax",
        ],
        "angularmovement" => &["id", "name", "force", "drag"],
        "turbulence" => &[
            "id",
            "name",
            "speedmin",
            "speedmax",
            "timescale",
            "scale",
            "phasemin",
            "phasemax",
            "mask",
        ],
        _ => return Err(unsupported("particle", "unsupported particle operator")),
    };
    known_keys(o, keys)?;
    Ok(match name {
        "sizechange" | "colorchange" => {
            let start_time = number(o, "starttime", 0.0)?;
            let end_time = number(o, "endtime", 1.0)?;
            ensure!(
                (0.0..=1.0).contains(&start_time) && (start_time..=1.0).contains(&end_time),
                "invalid particle value change interval"
            );
            if name == "sizechange" {
                let start = number(o, "startvalue", 1.0)?;
                let end = number(o, "endvalue", 0.0)?;
                ensure!(start >= 0.0 && end >= 0.0, "invalid particle size change");
                Operator::SizeChange {
                    start_time,
                    end_time,
                    start,
                    end,
                }
            } else {
                let start = vector(o, "startvalue", [1.0; 3])?;
                let end = vector(o, "endvalue", [0.0; 3])?;
                ensure!(
                    start
                        .iter()
                        .chain(&end)
                        .all(|value| (0.0..=1.0).contains(value)),
                    "invalid particle color change"
                );
                Operator::ColorChange {
                    start_time,
                    end_time,
                    start,
                    end,
                }
            }
        }
        "movement" => Operator::Movement {
            gravity: vector(o, "gravity", [0.0; 3])?,
            drag: number(o, "drag", 0.0)?,
        },
        "alphafade" => {
            let fade_in = number(o, "fadeintime", 0.5)?;
            let fade_out = number(o, "fadeouttime", 0.5)?;
            ensure!(
                (0.0..=1.0).contains(&fade_in) && (0.0..=1.0).contains(&fade_out),
                "invalid particle alpha fade"
            );
            Operator::AlphaFade { fade_in, fade_out }
        }
        "oscillatealpha" => Operator::OscillateAlpha(oscillation(o, 0.0, 1.0)?),
        "oscillatesize" => Operator::OscillateSize(oscillation(o, 0.8, 1.2)?),
        "angularmovement" => Operator::AngularMovement {
            force: vector(o, "force", [0.0; 3])?,
            drag: number(o, "drag", 0.0)?,
        },
        "turbulence" => Operator::Turbulence {
            speed: range(o, "speedmin", "speedmax", 500.0, 1000.0)?,
            time_scale: number(o, "timescale", 20.0)?,
            spatial_scale: o
                .get("scale")
                .map(|_| number(o, "scale", 0.0))
                .transpose()?,
            phase: if o.contains_key("phasemin") || o.contains_key("phasemax") {
                Some(range(o, "phasemin", "phasemax", 0.0, 0.0)?)
            } else {
                None
            },
            mask: vector(o, "mask", [1.0, 1.0, 0.0])?,
        },
        _ => unreachable!("validated operator name"),
    })
}

fn oscillation(o: &Map<String, Value>, min: f32, max: f32) -> Result<Oscillation> {
    // Native FUN_140188a50 / FUN_140188ce0 defaults; OWE uses a different frequency minimum.
    Ok(Oscillation {
        frequency: range(o, "frequencymin", "frequencymax", 1.0, 10.0)?,
        phase: range(o, "phasemin", "phasemax", 0.0, std::f32::consts::TAU)?,
        scale: range(o, "scalemin", "scalemax", min, max)?,
    })
}

fn parse_renderer(value: &Value) -> Result<ParticleRenderer> {
    let o = component(value)?;
    match required_string(o, "name")? {
        "sprite" => {
            known_keys(o, &["id", "name", "flags", "axis", "orientation"])?;
            ensure!(
                o.get("orientation")
                    .is_none_or(|v| v.as_str() == Some("screen")),
                "3D particle sprite orientation is unsupported"
            );
            let flags = integer(o, "flags", 0, u32::MAX as usize)?;
            ensure!(flags <= 1, "unsupported particle sprite flags");
            // Axis configures the unsupported fixed orientation; screen
            // orientation uses the camera plane independently of this vector.
            if o.contains_key("axis") {
                vector(o, "axis", [1.0; 3])?;
            }
            Ok(ParticleRenderer::Sprite {
                world_orientation: flags == 1,
            })
        }
        "ropetrail" => {
            known_keys(o, &["id", "name", "length", "segments", "subdivision"])?;
            let length = number(o, "length", 1.0)?;
            ensure!(
                (0.001..=30.0).contains(&length),
                "particle rope duration exceeds budget"
            );
            let segments = integer(o, "segments", 4, 32)?;
            ensure!(
                segments >= 2,
                "particle rope requires at least two segments"
            );
            Ok(ParticleRenderer::RopeTrail {
                length,
                segments,
                subdivision: integer(o, "subdivision", 1, 32)?,
            })
        }
        "spritetrail" => {
            known_keys(
                o,
                &[
                    "id",
                    "name",
                    "length",
                    "minlength",
                    "maxlength",
                    "flags",
                    "axis",
                    "orientation",
                ],
            )?;
            ensure!(
                integer(o, "flags", 0, 0)? == 0
                    && o.get("orientation")
                        .is_none_or(|v| v.as_str() == Some("screen")),
                "3D particle sprite trail orientation is unsupported"
            );
            if o.contains_key("axis") {
                vector(o, "axis", [1.0; 3])?;
            }
            let length = number(o, "length", 0.05)?;
            let min_length = number(o, "minlength", 0.0)?;
            let max_length = number(o, "maxlength", 10.0)?;
            ensure!(
                (0.0..=100.0).contains(&length)
                    && (0.0..=100.0).contains(&min_length)
                    && (min_length..=100.0).contains(&max_length),
                "invalid particle sprite trail length"
            );
            Ok(ParticleRenderer::SpriteTrail {
                length,
                min_length,
                max_length,
            })
        }
        _ => Err(unsupported("particle", "unsupported particle renderer")),
    }
}

fn reject_audio(o: &Map<String, Value>) -> Result<()> {
    if o.keys().any(|key| key.starts_with("audioprocessing")) {
        return Err(unsupported(
            "audio_analysis",
            "audio responsive particle component",
        ));
    }
    Ok(())
}

fn component(value: &Value) -> Result<&Map<String, Value>> {
    let o = object_map(value, "particle component")?;
    optional_id(o)?;
    Ok(o)
}

fn components<'a>(o: &'a Map<String, Value>, key: &str) -> Result<&'a [Value]> {
    if absent_or_null(o, key) {
        return Ok(&[]);
    }
    let a = o[key]
        .as_array()
        .with_context(|| format!("particle {key} must be an array"))?;
    ensure!(
        a.len() <= MAX_COMPONENTS,
        "particle component count exceeds budget"
    );
    Ok(a)
}

fn object_map<'a>(value: &'a Value, label: &str) -> Result<&'a Map<String, Value>> {
    value
        .as_object()
        .with_context(|| format!("{label} must be an object"))
}

fn known_keys(o: &Map<String, Value>, keys: &[&str]) -> Result<()> {
    if let Some(key) = o.keys().find(|key| !keys.contains(&key.as_str())) {
        return Err(unsupported(
            "particle",
            format!("unsupported particle field {key}"),
        ));
    }
    Ok(())
}

fn absent_or_null(o: &Map<String, Value>, key: &str) -> bool {
    o.get(key).is_none_or(Value::is_null)
}

fn required_string<'a>(o: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    let s = o
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("particle {key} must be a string"))?;
    ensure!(s.len() <= 512 && !s.is_empty(), "invalid particle string");
    Ok(s)
}

fn optional_string(o: &Map<String, Value>, key: &str) -> Result<()> {
    if o.contains_key(key) {
        required_string(o, key)?;
    }
    Ok(())
}

fn resource_path(s: &str) -> Result<String> {
    ensure!(
        !s.starts_with('/')
            && !s.contains('\\')
            && !s.contains('\0')
            && s.split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."),
        "invalid particle resource path"
    );
    ensure!(s.ends_with(".json"), "particle resource must be JSON");
    Ok(s.to_owned())
}

fn optional_id(o: &Map<String, Value>) -> Result<()> {
    if o.contains_key("id") {
        integer(o, "id", 0, u32::MAX as usize)?;
    }
    Ok(())
}

fn integer(o: &Map<String, Value>, key: &str, default: usize, max: usize) -> Result<usize> {
    let value = match o.get(key) {
        None | Some(Value::Null) => default,
        Some(value) => usize::try_from(
            value
                .as_u64()
                .with_context(|| format!("particle {key} must be a nonnegative integer"))?,
        )?,
    };
    ensure!(value <= max, "particle {key} exceeds budget");
    Ok(value)
}

fn number(o: &Map<String, Value>, key: &str, default: f32) -> Result<f32> {
    let value = match o.get(key) {
        None | Some(Value::Null) => default,
        Some(value) => value
            .as_f64()
            .with_context(|| format!("particle {key} must be numeric"))?
            as f32,
    };
    ensure!(
        value.is_finite() && value.abs() <= MAX_VALUE,
        "invalid particle numeric value"
    );
    Ok(value)
}

fn vector(o: &Map<String, Value>, key: &str, default: [f32; 3]) -> Result<[f32; 3]> {
    let Some(value) = o.get(key) else {
        return Ok(default);
    };
    let s = value
        .as_str()
        .with_context(|| format!("particle {key} must be a vector string"))?;
    ensure!(s.len() <= 256, "particle vector exceeds budget");
    let mut parts = s.split_whitespace();
    let mut result = [0.0; 3];
    for value in &mut result {
        *value = parts
            .next()
            .context("incomplete particle vector")?
            .parse::<f32>()?;
        ensure!(
            value.is_finite() && value.abs() <= MAX_VALUE,
            "invalid particle vector"
        );
    }
    ensure!(
        parts.next().is_none(),
        "particle vector must have three components"
    );
    Ok(result)
}

fn range(
    o: &Map<String, Value>,
    min_key: &str,
    max_key: &str,
    min: f32,
    max: f32,
) -> Result<RandomRange> {
    let exponent = number(o, "exponent", 1.0)?;
    ensure!(
        exponent > 0.0 && exponent <= 100.0,
        "invalid particle random exponent"
    );
    // Reversed endpoints are valid in shipped resources (alpha 0.6 -> 0.5).
    Ok(RandomRange {
        min: number(o, min_key, min)?,
        max: number(o, max_key, max)?,
        exponent,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub(crate) struct ParticleVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub uv_next: [f32; 2],
    pub uv_blend: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParticleBatch {
    /// Index in PreparedParticles::systems and material_paths(), depth first.
    pub material: usize,
    pub vertices: Vec<ParticleVertex>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParticleFrame {
    pub batches: Vec<ParticleBatch>,
    pub live_particles: usize,
}

pub(crate) struct ParticleRuntime {
    prepared: PreparedParticles,
    instances: Vec<Vec<Instance>>,
    rng: Random,
    next_id: u64,
    scene_seconds: f64,
    pending: f64,
    material_aspects: Vec<f64>,
    material_animations: Vec<Option<AnimationMetadata>>,
}

struct Instance {
    parent: Option<(u64, ChildMode)>,
    anchor: [f64; 2],
    scale: [f64; 2],
    emitting: bool,
    credit: Vec<f64>,
    particles: Vec<Particle>,
    time: f64,
}

struct Particle {
    id: u64,
    position: [f64; 3],
    velocity: [f64; 3],
    age: f64,
    life: f64,
    size: f64,
    alpha: f64,
    color: [f64; 3],
    angle: f64,
    angular_velocity: f64,
    random_frame: f64,
    oscillators: Vec<Option<(f64, f64)>>,
    trail: VecDeque<(f64, [f64; 2])>,
}

impl ParticleRuntime {
    pub fn new(prepared: &PreparedParticles, seed: u64) -> Result<Self> {
        prepared.validate_runtime()?;
        let mut runtime = Self {
            prepared: prepared.clone(),
            instances: (0..prepared.systems.len()).map(|_| Vec::new()).collect(),
            rng: Random(seed),
            next_id: 1,
            scene_seconds: 0.0,
            pending: 0.0,
            material_aspects: vec![1.0; prepared.systems.len()],
            material_animations: vec![None; prepared.systems.len()],
        };
        runtime.instances[0].push(Instance {
            parent: None,
            anchor: [
                prepared.transform.origin[0] as f64,
                prepared.transform.origin[1] as f64,
            ],
            scale: [
                prepared.transform.scale[0] as f64,
                prepared.transform.scale[1] as f64,
            ],
            emitting: true,
            credit: vec![0.0; prepared.systems[0].emitters.len()],
            particles: Vec::new(),
            time: 0.0,
        });
        for system in 0..prepared.systems.len() {
            for child in &prepared.systems[system].children {
                if child.mode != ChildMode::Static {
                    continue;
                }
                let child_definition = &prepared.systems[child.system];
                let instances: Vec<_> = runtime.instances[system]
                    .iter()
                    .map(|parent| Instance {
                        parent: None,
                        anchor: parent.anchor,
                        scale: [
                            parent.scale[0] * child.scale[0] as f64,
                            parent.scale[1] * child.scale[1] as f64,
                        ],
                        emitting: true,
                        credit: vec![0.0; child_definition.emitters.len()],
                        particles: Vec::new(),
                        time: 0.0,
                    })
                    .collect();
                runtime.instances[child.system].extend(instances);
            }
        }
        // Each static subsystem pre-rolls its own authored duration. Delay the
        // shorter simulations within one bounded warmup, without wall time.
        let warmup = prepared
            .systems
            .iter()
            .enumerate()
            .filter(|(index, _)| !runtime.instances[*index].is_empty())
            .map(|(_, system)| system.start_time as f64)
            .fold(0.0_f64, f64::max);
        for (index, instances) in runtime.instances.iter_mut().enumerate() {
            for instance in instances {
                instance.time = prepared.systems[index].start_time as f64 - warmup;
            }
        }
        let frames = ((warmup / STEP).ceil() as usize).min(MAX_STEPS);
        for _ in 0..frames {
            runtime.step(warmup / frames as f64)?;
        }
        Ok(runtime)
    }

    /// Texture storage height/width of each material, in material_paths order.
    /// Particle UVs sample the full texture, including padding.
    /// Validate the complete vector before changing any runtime state.
    pub fn configure_material_aspects(&mut self, aspects: &[f32]) -> Result<()> {
        ensure!(
            aspects.len() == self.prepared.systems.len(),
            "particle material aspect count mismatch"
        );
        ensure!(
            aspects
                .iter()
                .all(|value| value.is_finite() && (1.0 / 16384.0..=16384.0).contains(value)),
            "invalid particle material aspect"
        );
        self.material_aspects = aspects.iter().map(|value| *value as f64).collect();
        Ok(())
    }

    /// Frame tables are retained from the single-image atlas decode. All
    /// validation completes before replacing the current configuration.
    pub fn configure_material_animations(
        &mut self,
        animations: &[Option<AnimationMetadata>],
    ) -> Result<()> {
        ensure!(
            animations.len() == self.prepared.systems.len(),
            "particle material animation count mismatch"
        );
        let mut frames = 0usize;
        for (index, animation) in animations.iter().enumerate() {
            if let Some(animation) = animation {
                ensure!(
                    animation.version == 2
                        && animation.width > 0
                        && animation.height > 0
                        && !animation.frames.is_empty(),
                    "unsupported particle atlas metadata"
                );
                frames = frames
                    .checked_add(animation.frames.len())
                    .context("particle atlas frame budget overflow")?;
                ensure!(frames <= 8192, "particle atlas frames exceed budget");
                ensure!(
                    !matches!(
                        self.prepared.systems[index].renderer,
                        ParticleRenderer::RopeTrail { .. }
                    ),
                    "particle rope atlas is unsupported"
                );
                for frame in &animation.frames {
                    let [x, y, width, shear_y, shear_x, height] = frame.coordinates;
                    ensure!(
                        frame.image == 0
                            && frame.duration == 0.0
                            && frame.coordinates.iter().all(|value| value.is_finite())
                            && x >= 0.0
                            && y >= 0.0
                            && width > 0.0
                            && height > 0.0
                            && shear_y == 0.0
                            && shear_x == 0.0
                            && x + width <= animation.width as f32
                            && y + height <= animation.height as f32,
                        "invalid particle atlas frame"
                    );
                }
            } else {
                ensure!(
                    self.prepared.systems[index].animation_mode == AnimationMode::Sequence
                        && self.prepared.systems[index].sequence_multiplier == 1.0,
                    "particle animation requires a sprite atlas"
                );
            }
        }
        self.material_animations = animations.to_vec();
        Ok(())
    }

    /// The engine supplies SceneClock seconds; repeated times cannot advance RNG,
    /// particle age, child creation, or trails. Excessive catch-up is rejected
    /// before mutating state, so pause/resume cannot invent elapsed wall time.
    pub fn advance(&mut self, scene_seconds: f64) -> Result<ParticleFrame> {
        ensure!(
            scene_seconds.is_finite() && scene_seconds >= self.scene_seconds,
            "invalid or decreasing particle scene time"
        );
        let delta = scene_seconds - self.scene_seconds;
        ensure!(
            delta + self.pending <= STEP * MAX_STEPS as f64,
            "particle clock catch-up exceeds work budget"
        );
        self.scene_seconds = scene_seconds;
        self.pending += delta;
        while self.pending + 1e-10 >= STEP {
            self.step(STEP * self.prepared.modifiers.rate as f64)?;
            self.pending = (self.pending - STEP).max(0.0);
        }
        self.frame()
    }

    fn step(&mut self, delta: f64) -> Result<()> {
        let angle = self.prepared.transform.angles[2] as f64;
        let modifiers = self.prepared.modifiers;
        let mut parents = BTreeMap::<u64, [f64; 2]>::new();
        for system_index in 0..self.instances.len() {
            let definition = &self.prepared.systems[system_index];
            let mut spawns = Vec::new();
            for instance in &mut self.instances[system_index] {
                if let Some((parent, mode)) = instance.parent {
                    if let Some(position) = parents.get(&parent) {
                        if mode == ChildMode::Follow {
                            instance.anchor = *position;
                        }
                    } else {
                        instance.emitting = false;
                        if mode == ChildMode::Follow {
                            instance.particles.clear();
                        }
                    }
                }
                let before = instance.time.max(0.0);
                instance.time += delta;
                let delta = instance.time.max(0.0) - before;
                if delta == 0.0 {
                    continue;
                }
                for particle in &mut instance.particles {
                    particle.age += delta;
                    update_particle(
                        particle,
                        definition,
                        delta,
                        instance.time,
                        modifiers.speed as f64,
                    );
                }
                instance
                    .particles
                    .retain(|particle| particle.age < particle.life);
                if instance.emitting && delta > 0.0 {
                    for (emitter_index, emitter) in definition.emitters.iter().enumerate() {
                        instance.credit[emitter_index] +=
                            delta * emitter.rate as f64 * modifiers.count as f64;
                        let instantaneous =
                            emitter.instantaneous > 0 && instance.particles.is_empty();
                        let requested = if instantaneous {
                            emitter.instantaneous
                        } else {
                            instance.credit[emitter_index].floor() as usize
                        };
                        let emitted =
                            requested.min(definition.max_count - instance.particles.len());
                        // Drop capacity-blocked emission debt; it must not erupt when a slot expires.
                        if !instantaneous {
                            instance.credit[emitter_index] = instance.credit[emitter_index].fract();
                        }
                        for _ in 0..emitted {
                            let mut particle = make_particle_at(
                                definition,
                                emitter,
                                modifiers,
                                &mut self.rng,
                                self.next_id,
                                instance.time,
                            );
                            self.next_id = self
                                .next_id
                                .checked_add(1)
                                .context("particle identity exhausted")?;
                            if definition.world_space {
                                let world = point_world(
                                    particle.position,
                                    instance.anchor,
                                    instance.scale,
                                    angle,
                                );
                                particle.position[0] = world[0];
                                particle.position[1] = world[1];
                                let v = direction_world(
                                    [particle.velocity[0], particle.velocity[1]],
                                    instance.scale,
                                    angle,
                                );
                                particle.velocity[0] = v[0];
                                particle.velocity[1] = v[1];
                            }
                            let world =
                                particle_world(&particle, definition.world_space, instance, angle);
                            for child in definition
                                .children
                                .iter()
                                .filter(|child| child.mode != ChildMode::Static)
                            {
                                spawns.push((*child, particle.id, world, instance.scale));
                            }
                            particle.trail.push_back((0.0, world));
                            instance.particles.push(particle);
                        }
                    }
                }
                for particle in &mut instance.particles {
                    let world = if definition.world_space {
                        [particle.position[0], particle.position[1]]
                    } else {
                        point_world(particle.position, instance.anchor, instance.scale, angle)
                    };
                    ensure!(
                        world
                            .iter()
                            .all(|v| v.is_finite() && v.abs() < f32::MAX as f64),
                        "particle simulation produced invalid position in {}",
                        definition.path
                    );
                    parents.insert(particle.id, world);
                    if let ParticleRenderer::RopeTrail {
                        length, segments, ..
                    } = definition.renderer
                    {
                        let interval = length as f64 / (segments - 1) as f64;
                        if particle
                            .trail
                            .back()
                            .is_none_or(|(time, _)| particle.age - time >= interval)
                        {
                            particle.trail.push_back((particle.age, world));
                            while particle.trail.len() > segments + 1 {
                                particle.trail.pop_front();
                            }
                        }
                    }
                }
            }
            self.instances[system_index]
                .retain(|instance| instance.emitting || !instance.particles.is_empty());
            for (child, id, anchor, scale) in spawns {
                let child_system = &self.prepared.systems[child.system];
                self.instances[child.system].retain(|instance| !matches!(instance.parent, Some((parent, ChildMode::Follow)) if !parents.contains_key(&parent)));
                // Each definition belongs to exactly one tree edge; maxcount
                // bounds instances independently from particles per instance.
                if self.instances[child.system].len() >= child.max_instances {
                    continue;
                }
                self.instances[child.system].push(Instance {
                    parent: Some((id, child.mode)),
                    anchor,
                    scale: [
                        scale[0] * child.scale[0] as f64,
                        scale[1] * child.scale[1] as f64,
                    ],
                    emitting: true,
                    credit: vec![0.0; child_system.emitters.len()],
                    particles: Vec::new(),
                    time: 0.0,
                });
            }
        }
        Ok(())
    }

    fn frame(&self) -> Result<ParticleFrame> {
        let mut batches = Vec::with_capacity(self.instances.len());
        let mut live_particles = 0;
        let mut total_vertices = 0;
        let angle = self.prepared.transform.angles[2] as f64;
        for (material, instances) in self.instances.iter().enumerate() {
            let definition = &self.prepared.systems[material];
            let mut vertices = Vec::new();
            for instance in instances {
                for particle in &instance.particles {
                    live_particles += 1;
                    let vertex_start = vertices.len();
                    let (size, alpha) = appearance(particle, &definition.operators);
                    let (uv, uv_next, uv_blend, aspect) = atlas_frame(
                        self.material_animations[material].as_ref(),
                        definition,
                        particle,
                        self.material_aspects[material],
                    );
                    let appearance_color = particle_color(particle, &definition.operators);
                    let color = [
                        appearance_color[0] as f32,
                        appearance_color[1] as f32,
                        appearance_color[2] as f32,
                        alpha.clamp(0.0, 1.0) as f32,
                    ];
                    let center = particle_world(particle, definition.world_space, instance, angle);
                    match definition.renderer {
                        ParticleRenderer::Sprite { .. } | ParticleRenderer::SpriteTrail { .. } => {
                            // Sprite flags 1 selects world orientation, not velocity stretching.
                            let (orientation, stretch) = match definition.renderer {
                                ParticleRenderer::Sprite { world_orientation } => (
                                    particle.angle + if world_orientation { 0.0 } else { angle },
                                    1.0,
                                ),
                                ParticleRenderer::SpriteTrail {
                                    length,
                                    min_length,
                                    max_length,
                                } => {
                                    let speed = particle.velocity[0].hypot(particle.velocity[1]);
                                    (
                                        if speed > 1e-10 {
                                            particle.velocity[1].atan2(particle.velocity[0])
                                                - std::f64::consts::FRAC_PI_2
                                                + angle
                                        } else {
                                            particle.angle + angle
                                        },
                                        (speed * length as f64)
                                            .clamp(min_length as f64, max_length as f64),
                                    )
                                }
                                _ => unreachable!("sprite renderer matched"),
                            };
                            let corners =
                                [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]].map(|p| {
                                    let d = direction_world(
                                        [p[0] * size, p[1] * size * aspect * stretch],
                                        instance.scale,
                                        orientation,
                                    );
                                    [center[0] + d[0], center[1] + d[1]]
                                });
                            quad(
                                &mut vertices,
                                corners,
                                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                                color,
                            );
                        }
                        ParticleRenderer::RopeTrail {
                            length,
                            segments,
                            subdivision,
                        } => {
                            let mut points = Vec::with_capacity(segments);
                            for i in 0..segments {
                                points.push(trail_position(
                                    particle,
                                    particle.age - i as f64 * length as f64 / (segments - 1) as f64,
                                    center,
                                ));
                            }
                            let pieces = subdivision + 1;
                            for segment in 0..segments - 1 {
                                for piece in 0..pieces {
                                    let start = smooth_trail(
                                        &points,
                                        segment,
                                        piece as f64 / pieces as f64,
                                    );
                                    let end = smooth_trail(
                                        &points,
                                        segment,
                                        (piece + 1) as f64 / pieces as f64,
                                    );
                                    let tangent = [end[0] - start[0], end[1] - start[1]];
                                    let distance = tangent[0].hypot(tangent[1]);
                                    if distance < 1e-8 {
                                        continue;
                                    }
                                    let width = size
                                        * self.material_aspects[material]
                                        * instance.scale[0].abs()
                                        * 0.5;
                                    let n = [
                                        -tangent[1] * width / distance,
                                        tangent[0] * width / distance,
                                    ];
                                    let u0 = (segment as f32 + piece as f32 / pieces as f32)
                                        / (segments - 1) as f32;
                                    let u1 = (segment as f32 + (piece + 1) as f32 / pieces as f32)
                                        / (segments - 1) as f32;
                                    quad(
                                        &mut vertices,
                                        [
                                            [start[0] + n[0], start[1] + n[1]],
                                            [start[0] - n[0], start[1] - n[1]],
                                            [end[0] - n[0], end[1] - n[1]],
                                            [end[0] + n[0], end[1] + n[1]],
                                        ],
                                        [[u0, 0.0], [u0, 1.0], [u1, 1.0], [u1, 0.0]],
                                        color,
                                    );
                                }
                            }
                        }
                    }
                    for vertex in &mut vertices[vertex_start..] {
                        let local = vertex.uv;
                        vertex.uv = [uv[0] + local[0] * uv[2], uv[1] + local[1] * uv[3]];
                        vertex.uv_next = [
                            uv_next[0] + local[0] * uv_next[2],
                            uv_next[1] + local[1] * uv_next[3],
                        ];
                        vertex.uv_blend = uv_blend;
                    }
                }
            }
            total_vertices += vertices.len();
            ensure!(
                total_vertices <= MAX_VERTICES && live_particles <= self.prepared.particle_capacity,
                "particle frame exceeds preparation budget"
            );
            ensure!(
                vertices.iter().all(|v| v
                    .position
                    .iter()
                    .chain(&v.color)
                    .chain(&v.uv)
                    .chain(&v.uv_next)
                    .chain(std::iter::once(&v.uv_blend))
                    .all(|x| x.is_finite())),
                "particle geometry contains nonfinite values in {}",
                definition.path
            );
            batches.push(ParticleBatch { material, vertices });
        }
        Ok(ParticleFrame {
            batches,
            live_particles,
        })
    }
}

fn direction_world(p: [f64; 2], scale: [f64; 2], angle: f64) -> [f64; 2] {
    let (sin, cos) = angle.sin_cos();
    let x = p[0] * scale[0];
    let y = p[1] * scale[1];
    [x * cos - y * sin, x * sin + y * cos]
}

fn point_world(p: [f64; 3], anchor: [f64; 2], scale: [f64; 2], angle: f64) -> [f64; 2] {
    let d = direction_world([p[0], p[1]], scale, angle);
    [anchor[0] + d[0], anchor[1] + d[1]]
}

fn particle_world(p: &Particle, world: bool, instance: &Instance, angle: f64) -> [f64; 2] {
    if world {
        [p.position[0], p.position[1]]
    } else {
        point_world(p.position, instance.anchor, instance.scale, angle)
    }
}

#[cfg(test)]
fn make_particle(
    def: &ParticleSystem,
    emitter: &Emitter,
    mods: ParticleModifiers,
    rng: &mut Random,
    id: u64,
) -> Particle {
    make_particle_at(def, emitter, mods, rng, id, 0.0)
}

fn make_particle_at(
    def: &ParticleSystem,
    emitter: &Emitter,
    mods: ParticleModifiers,
    rng: &mut Random,
    id: u64,
    time: f64,
) -> Particle {
    let mut p = Particle {
        id,
        position: emitter.origin.map(f64::from),
        velocity: [0.0; 3],
        age: 0.0,
        life: 1.0,
        size: 20.0,
        alpha: 1.0,
        color: [1.0; 3],
        angle: 0.0,
        angular_velocity: 0.0,
        random_frame: if def.animation_mode == AnimationMode::RandomFrame {
            rng.unit()
        } else {
            0.0
        },
        oscillators: Vec::new(),
        trail: VecDeque::new(),
    };
    let directions = emitter.directions.unwrap_or([1.0, 1.0, 0.0]);
    match emitter.shape {
        EmitterShape::Box { min, max } => {
            for i in 0..3 {
                p.position[i] += (min[i] as f64
                    + (2.0 * rng.unit() - 1.0) * (max[i] - min[i]) as f64)
                    * directions[i] as f64;
            }
        }
        EmitterShape::Sphere { min, max } => {
            let theta = rng.unit() * std::f64::consts::TAU;
            let radius = ((min as f64).powi(2)
                + rng.unit() * ((max as f64).powi(2) - (min as f64).powi(2)))
            .max(0.0)
            .sqrt();
            p.position[0] += radius * theta.cos() * directions[0] as f64;
            p.position[1] += radius * theta.sin() * directions[1] as f64;
        }
    }
    for init in &def.initializers {
        match *init {
            Initializer::Lifetime(r) => p.life = rng.range(r),
            Initializer::Size(r) => p.size = rng.range(r),
            Initializer::Alpha(r) => p.alpha = rng.range(r),
            Initializer::Color { min, max } => {
                let t = rng.unit();
                p.color = std::array::from_fn(|i| {
                    ((min[i] as f64 + t * (max[i] - min[i]) as f64) / 255.0).powi(2)
                });
            }
            Initializer::Velocity { min, max } => {
                p.velocity =
                    std::array::from_fn(|i| min[i] as f64 + rng.unit() * (max[i] - min[i]) as f64)
            }
            Initializer::Rotation(range) => p.angle = rng.range(range),
            Initializer::AngularVelocity(range) => p.angular_velocity = rng.range(range),
            Initializer::TurbulentVelocity {
                speed,
                phase,
                scale,
                time_scale,
                offset,
                forward,
            } => {
                let phase = rng.range(phase);
                // Reuse LWE's independent coherent curl field. Field identity is
                // an LWE choice; it does not claim the native engine's RNG/noise.
                let noise = curl_noise([time * time_scale as f64, phase, 0.0]);
                let forward_angle = (forward[1] as f64).atan2(forward[0] as f64);
                let heading = if noise[0].hypot(noise[1]) > 1e-10 {
                    noise[1].atan2(noise[0])
                } else {
                    forward_angle
                };
                let relative = (heading - forward_angle + std::f64::consts::PI)
                    .rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI;
                let angle =
                    forward_angle + relative * (scale as f64 * 0.5).max(0.0) + offset as f64;
                let speed = rng.range(speed);
                p.velocity[0] += angle.cos() * speed;
                p.velocity[1] += angle.sin() * speed;
            }
        }
    }
    p.life *= mods.lifetime as f64;
    p.size *= mods.size as f64;
    p.alpha *= mods.alpha as f64;
    p.velocity = p.velocity.map(|v| v * mods.speed as f64);
    p.angular_velocity *= mods.speed as f64;
    if let Some(color) = mods.color {
        p.color = color.map(|v| (v as f64).powi(2));
    }
    p.color = p.color.map(|v| v * mods.brightness as f64);
    p.oscillators = def
        .operators
        .iter()
        .map(|op| match *op {
            Operator::OscillateAlpha(o) | Operator::OscillateSize(o) => {
                Some((rng.range(o.frequency), rng.range(o.phase)))
            }
            Operator::Turbulence { speed, phase, .. } => Some((
                rng.range(speed),
                rng.range(phase.unwrap_or(RandomRange {
                    min: 0.0,
                    max: 0.0,
                    exponent: 1.0,
                })),
            )),
            _ => None,
        })
        .collect();
    p
}

fn update_particle(p: &mut Particle, def: &ParticleSystem, dt: f64, time: f64, speed: f64) {
    for (index, op) in def.operators.iter().enumerate() {
        match *op {
            Operator::Movement { gravity, drag } => {
                for (i, g) in gravity.iter().enumerate() {
                    p.velocity[i] += (*g as f64 * speed - p.velocity[i] * drag as f64) * dt;
                    p.position[i] += p.velocity[i] * dt;
                }
            }
            Operator::AngularMovement { force, drag } => {
                p.angular_velocity += (force[2] as f64 - p.angular_velocity * drag as f64) * dt;
                p.angle += p.angular_velocity * dt;
            }
            Operator::Turbulence {
                time_scale,
                spatial_scale,
                mask,
                ..
            } => {
                let (force, phase) = p.oscillators[index].expect("prepared turbulence state");
                let scale = spatial_scale.unwrap_or(0.01) as f64 * 2.0;
                let q = [
                    (p.position[0] + phase + time_scale as f64 * time) * scale,
                    p.position[1] * scale,
                    p.position[2] * scale,
                ];
                let curl = curl_noise(q);
                let length = (curl.iter().map(|v| v * v).sum::<f64>()).sqrt();
                if length > 1e-12 {
                    for i in 0..3 {
                        p.velocity[i] += curl[i] / length * mask[i] as f64 * force * speed * dt;
                    }
                }
            }
            _ => {}
        }
    }
}

fn appearance(p: &Particle, operators: &[Operator]) -> (f64, f64) {
    let mut size = p.size;
    let mut alpha = p.alpha;
    for (index, op) in operators.iter().enumerate() {
        match *op {
            Operator::SizeChange {
                start_time,
                end_time,
                start,
                end,
            } => {
                size *= value_change(
                    p.age / p.life,
                    start_time,
                    end_time,
                    start as f64,
                    end as f64,
                );
            }
            Operator::AlphaFade { fade_in, fade_out } => {
                let age = p.age / p.life;
                if fade_in > 0.0 && age < fade_in as f64 {
                    alpha *= age / fade_in as f64;
                }
                if fade_out < 1.0 && age > fade_out as f64 {
                    alpha *= (1.0 - age) / (1.0 - fade_out as f64);
                }
            }
            Operator::OscillateAlpha(o) | Operator::OscillateSize(o) => {
                let (frequency, phase) = p.oscillators[index].expect("prepared oscillation state");
                let factor = o.scale.min as f64
                    + ((frequency * p.age + phase).cos() * 0.5 + 0.5)
                        * (o.scale.max - o.scale.min) as f64;
                if matches!(op, Operator::OscillateAlpha(_)) {
                    alpha *= factor;
                } else {
                    size *= factor;
                }
            }
            _ => {}
        }
    }
    (size, alpha)
}

fn value_change(age: f64, start_time: f32, end_time: f32, start: f64, end: f64) -> f64 {
    let progress = if age <= start_time as f64 {
        0.0
    } else if age >= end_time as f64 {
        1.0
    } else {
        (age - start_time as f64) / (end_time - start_time) as f64
    };
    start + (end - start) * progress
}

fn particle_color(p: &Particle, operators: &[Operator]) -> [f64; 3] {
    let mut color = p.color;
    for operator in operators {
        if let Operator::ColorChange {
            start_time,
            end_time,
            start,
            end,
        } = *operator
        {
            color = std::array::from_fn(|i| {
                color[i]
                    * value_change(
                        p.age / p.life,
                        start_time,
                        end_time,
                        start[i] as f64,
                        end[i] as f64,
                    )
            });
        }
    }
    color
}

fn atlas_frame(
    animation: Option<&AnimationMetadata>,
    system: &ParticleSystem,
    particle: &Particle,
    static_aspect: f64,
) -> ([f32; 4], [f32; 4], f32, f64) {
    let Some(animation) = animation else {
        return (
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            0.0,
            static_aspect,
        );
    };
    let count = animation.frames.len();
    let position = if system.animation_mode == AnimationMode::RandomFrame {
        particle.random_frame * count as f64
    } else {
        (particle.age / particle.life * system.sequence_multiplier as f64).rem_euclid(1.0)
            * count as f64
    };
    let index = (position.floor() as usize).min(count - 1);
    let next = if system.frame_blending {
        (index + 1) % count
    } else {
        index
    };
    let rect = |index: usize| {
        let [x, y, width, _, _, height] = animation.frames[index].coordinates;
        [
            x / animation.width as f32,
            y / animation.height as f32,
            width / animation.width as f32,
            height / animation.height as f32,
        ]
    };
    let coordinates = animation.frames[index].coordinates;
    (
        rect(index),
        rect(next),
        if system.frame_blending {
            position.fract() as f32
        } else {
            0.0
        },
        (coordinates[5] / coordinates[2]) as f64,
    )
}

fn quad(
    vertices: &mut Vec<ParticleVertex>,
    points: [[f64; 2]; 4],
    uvs: [[f32; 2]; 4],
    color: [f32; 4],
) {
    for index in [0, 1, 2, 0, 2, 3] {
        vertices.push(ParticleVertex {
            position: points[index].map(|v| v as f32),
            uv: uvs[index],
            uv_next: uvs[index],
            uv_blend: 0.0,
            color,
        });
    }
}

fn trail_position(p: &Particle, time: f64, current: [f64; 2]) -> [f64; 2] {
    let Some(&(first_time, first)) = p.trail.front() else {
        return current;
    };
    if time <= first_time {
        return first;
    }
    let mut previous = (first_time, first);
    for &(next_time, next) in p
        .trail
        .iter()
        .skip(1)
        .chain(std::iter::once(&(p.age, current)))
    {
        if time <= next_time {
            let t = ((time - previous.0) / (next_time - previous.0).max(1e-12)).clamp(0.0, 1.0);
            return std::array::from_fn(|i| previous.1[i] + (next[i] - previous.1[i]) * t);
        }
        previous = (next_time, next);
    }
    current
}

fn smooth_trail(points: &[[f64; 2]], segment: usize, t: f64) -> [f64; 2] {
    let a = points[segment.saturating_sub(1)];
    let b = points[segment];
    let c = points[segment + 1];
    let d = points[(segment + 2).min(points.len() - 1)];
    std::array::from_fn(|i| {
        0.5 * (2.0 * b[i]
            + (-a[i] + c[i]) * t
            + (2.0 * a[i] - 5.0 * b[i] + 4.0 * c[i] - d[i]) * t * t
            + (-a[i] + 3.0 * b[i] - 3.0 * c[i] + d[i]) * t * t * t)
    })
}

// Independent smooth gradient field. The curl construction follows the
// mathematical vector-potential definition; hash/gradient choices are LWE's.
// This is not a claim of native or OWE random-field parity.
fn noise(p: [f64; 3], seed: u64) -> f64 {
    let base = p.map(|v| v.floor() as i64);
    let f = std::array::from_fn::<_, 3, _>(|i| p[i] - p[i].floor());
    let blend = f.map(|t| t * t * t * (t * (t * 6.0 - 15.0) + 10.0));
    let mut result = 0.0;
    for corner in 0..8 {
        let mut h = seed;
        let mut weight = 1.0;
        let mut offset = [0.0; 3];
        for i in 0..3 {
            let bit = (corner >> i) & 1;
            h = mix(h
                ^ (base[i].wrapping_add(bit as i64) as u64)
                    .wrapping_mul([0x9e3779b185ebca87, 0xc2b2ae3d27d4eb4f, 0x165667b19e3779f9][i]));
            offset[i] = f[i] - bit as f64;
            weight *= if bit == 0 { 1.0 - blend[i] } else { blend[i] };
        }
        let axis = (h % 3) as usize;
        let other = (axis + 1 + (h >> 8) as usize % 2) % 3;
        let gradient = offset[axis] * if h & 8 == 0 { 1.0 } else { -1.0 }
            + offset[other] * if h & 16 == 0 { 1.0 } else { -1.0 };
        result += weight * gradient;
    }
    result
}

fn curl_noise(p: [f64; 3]) -> [f64; 3] {
    let mut derivative = [[0.0; 3]; 3];
    let epsilon = 1e-4;
    for axis in 0..3 {
        let mut a = p;
        let mut b = p;
        a[axis] -= epsilon;
        b[axis] += epsilon;
        for (field, output) in derivative[axis].iter_mut().enumerate() {
            *output = (noise(b, field as u64 + 17) - noise(a, field as u64 + 17)) / (2.0 * epsilon);
        }
    }
    [
        derivative[1][2] - derivative[2][1],
        derivative[2][0] - derivative[0][2],
        derivative[0][1] - derivative[1][0],
    ]
}

fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

struct Random(u64);
impl Random {
    fn unit(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        (mix(self.0) >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, r: RandomRange) -> f64 {
        r.min as f64 + self.unit().powf(r.exponent as f64) * (r.max - r.min) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn resource() -> Value {
        json!({"material":"materials/test.json", "maxcount":4,
            "emitter":[{"name":"boxrandom", "distancemax":"100 100 0"}],
            "initializer":[{"name":"lifetimerandom","min":3,"max":5},
                {"name":"colorrandom","min":"255 255 255"}],
            "operator":[{"name":"alphafade"},{"name":"oscillatealpha","frequencymax":3}],
            "renderer":[{"name":"sprite"}]})
    }

    fn prepare(value: &Value) -> Result<PreparedParticles> {
        PreparedParticles::prepare(&json!({"particle":"particles/test.json"}), &mut |_| {
            Ok(value.clone())
        })
    }

    #[test]
    fn native_defaults_and_null_optional_fields_are_preserved() {
        let mut value = resource();
        value["children"] = Value::Null;
        value["flags"] = Value::Null;
        value["sequencemultiplier"] = Value::Null;
        let prepared = prepare(&value).unwrap();
        assert_eq!(prepared.systems[0].emitters[0].rate, 10.0);
        assert_eq!(
            prepared.material_paths().collect::<Vec<_>>(),
            ["materials/test.json"]
        );
        let Operator::OscillateAlpha(oscillation) = prepared.systems[0].operators[1] else {
            panic!("expected oscillation")
        };
        assert_eq!(oscillation.frequency.min, 1.0);
        assert_eq!(oscillation.phase.max, std::f32::consts::TAU);
        assert!(prepared.validate_runtime().is_ok());
    }

    #[test]
    fn reversed_alpha_endpoints_are_retained() {
        let mut value = resource();
        value["initializer"] = json!([{"name":"alpharandom","min":0.6,"max":0.5}]);
        let prepared = prepare(&value).unwrap();
        let Initializer::Alpha(range) = prepared.systems[0].initializers[0] else {
            panic!("expected alpha")
        };
        assert_eq!((range.min, range.max), (0.6, 0.5));
    }

    #[test]
    fn child_preparation_is_depth_first_and_capacity_counts_instances() {
        let mut root = resource();
        root["children"] = json!([{"name":"particles/child.json","type":"eventfollow","maxcount":5,"scale":"2.5 2.5 1"}]);
        let prepared =
            PreparedParticles::prepare(&json!({"particle":"particles/root.json"}), &mut |path| {
                Ok(if path.ends_with("root.json") {
                    root.clone()
                } else {
                    resource()
                })
            })
            .unwrap();
        assert_eq!(prepared.systems.len(), 2);
        assert_eq!(prepared.particle_capacity, 4 + 5 * 4);
        assert_eq!(prepared.systems[0].children[0].system, 1);
        assert_eq!(prepared.systems[0].children[0].mode, ChildMode::Follow);
        assert_eq!(prepared.systems[0].children[0].scale, [2.5, 2.5, 1.0]);
    }

    #[test]
    fn cycles_and_capacity_exhaustion_fail_before_unbounded_reads() {
        let mut value = resource();
        value["children"] =
            json!([{"name":"particles/test.json","type":"eventspawn","maxcount":4}]);
        let mut reads = 0;
        let error =
            PreparedParticles::prepare(&json!({"particle":"particles/test.json"}), &mut |_| {
                reads += 1;
                Ok(value.clone())
            })
            .unwrap_err();
        assert_eq!(reads, 1);
        assert!(error.to_string().contains("cycle"));
        value["children"][0]["name"] = json!("particles/child.json");
        value["children"][0]["maxcount"] = json!(MAX_PARTICLES);
        let error = prepare(&value).unwrap_err();
        assert!(error.to_string().contains("capacity"));
    }

    #[test]
    fn unknown_behavior_and_audio_processing_are_never_ignored() {
        for (key, field) in [
            ("operator", json!({"name":"mystery"})),
            ("operator", json!({"name":"movement","flags":4})),
            ("renderer", json!({"name":"sprite","custom":true})),
            ("initializer", json!({"name":"colorrandom","exponent":3})),
        ] {
            let mut value = resource();
            value[key] = json!([field]);
            assert!(prepare(&value).is_err());
        }
        let mut value = resource();
        value["emitter"][0]["audioprocessingmode"] = json!(3);
        assert_eq!(
            super::super::unsupported_feature_code(&prepare(&value).unwrap_err()),
            Some("audio_analysis")
        );
    }

    #[test]
    fn malformed_values_vectors_paths_and_controlpoints_are_rejected() {
        for bad in [
            json!("1 2"),
            json!("1 2 3 4"),
            json!("NaN 2 0"),
            json!([1, 2, 0]),
        ] {
            let mut value = resource();
            value["emitter"][0]["distancemax"] = bad;
            assert!(prepare(&value).is_err());
        }
        for path in [
            "../bad.json",
            "/bad.json",
            "particles\\bad.json",
            "particles//bad.json",
            "particles/a.tex",
        ] {
            assert!(
                PreparedParticles::prepare(&json!({"particle":path}), &mut |_| Ok(resource()))
                    .is_err()
            );
        }
        let mut value = resource();
        value["controlpoint"] = json!([{"id":0}, {"id":0}]);
        assert!(prepare(&value).is_err());
        value["controlpoint"] = json!([{"id":0,"flags":1}]);
        assert!(prepare(&value).is_err());
        value["maxcount"] = json!(4.5);
        assert!(prepare(&value).is_err());
    }

    #[test]
    fn overrides_and_serialized_transform_are_validated_without_conversion() {
        let prepared = PreparedParticles::prepare(&json!({"particle":"particles/test.json","angles":"0 0 -1.57719","scale":"-1 1 1","instanceoverride":{"count":0.54,"size":1.74,"colorn":"0 1 1"}}), &mut |_| Ok(resource())).unwrap();
        assert_eq!(prepared.transform.angles[2], -1.57719);
        assert_eq!(prepared.transform.scale[0], -1.0);
        assert_eq!(prepared.modifiers.color, Some([0.0, 1.0, 1.0]));
        assert_eq!(prepared.modifiers.count, 0.54);
        assert!(
            PreparedParticles::prepare(
                &json!({"particle":"particles/test.json","instanceoverride":{"count":-1}}),
                &mut |_| Ok(resource())
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_clock_pause_and_chunking_are_deterministic() {
        let prepared = prepare(&resource()).unwrap();
        let mut a = ParticleRuntime::new(&prepared, 42).unwrap();
        let mut b = ParticleRuntime::new(&prepared, 42).unwrap();
        let first = a.advance(0.5).unwrap();
        assert_eq!(a.advance(0.5).unwrap(), first);
        b.advance(0.25).unwrap();
        assert_eq!(b.advance(0.5).unwrap(), first);
        assert!(a.advance(f64::NAN).is_err());
        assert!(a.advance(0.4).is_err());
        assert!(a.advance(20.0).is_err());
        assert_eq!(a.advance(0.5).unwrap(), first);
        assert!(first.live_particles > 0);
    }

    #[test]
    fn movement_and_appearance_do_not_accumulate_render_side_effects() {
        let mut value = resource();
        value["initializer"] = json!([{"name":"lifetimerandom","min":4,"max":4},{"name":"sizerandom","min":20,"max":20},{"name":"velocityrandom","min":"10 0 0","max":"10 0 0"}]);
        value["operator"] = json!([{"name":"movement","gravity":"0 -2 0"},{"name":"alphafade","fadeintime":0.5,"fadeouttime":0.5}]);
        let prepared = prepare(&value).unwrap();
        let def = &prepared.systems[0];
        let mut p = make_particle(def, &def.emitters[0], prepared.modifiers, &mut Random(1), 1);
        p.position = [0.0; 3];
        p.age = 1.0;
        update_particle(&mut p, def, 1.0, 1.0, 1.0);
        assert_eq!(p.position, [10.0, -2.0, 0.0]);
        assert_eq!(appearance(&p, &def.operators), (20.0, 0.5));
        assert_eq!(appearance(&p, &def.operators), (20.0, 0.5));
    }

    #[test]
    fn children_follow_or_spawn_and_follow_is_released_when_parent_dies() {
        let mut root = resource();
        root["emitter"] = json!([{"name":"sphererandom","distancemin":0,"distancemax":0,"instantaneous":1,"rate":0}]);
        root["initializer"] = json!([{"name":"lifetimerandom","min":0.2,"max":0.2},{"name":"velocityrandom","min":"10 0 0","max":"10 0 0"}]);
        root["operator"] = json!([{"name":"movement"}]);
        root["children"] = json!([{"name":"particles/follow.json","type":"eventfollow","maxcount":4},{"name":"particles/spawn.json","type":"eventspawn","maxcount":4}]);
        let mut child = root.clone();
        child["children"] = Value::Null;
        child["initializer"] = json!([{"name":"lifetimerandom","min":2,"max":2}]);
        child["operator"] = json!([]);
        let prepared =
            PreparedParticles::prepare(&json!({"particle":"particles/root.json"}), &mut |path| {
                Ok(if path.ends_with("root.json") {
                    root.clone()
                } else {
                    child.clone()
                })
            })
            .unwrap();
        let mut runtime = ParticleRuntime::new(&prepared, 4).unwrap();
        runtime.advance(0.1).unwrap();
        let following = runtime.instances[1][0].anchor;
        let spawned = runtime.instances[2][0].anchor;
        assert!(following[0] > spawned[0]);
        let old_parent = runtime.instances[1][0].parent.unwrap().0;
        runtime.advance(0.4).unwrap();
        assert!(
            runtime.instances[1]
                .iter()
                .all(|i| i.parent.unwrap().0 != old_parent)
        );
        assert!(
            runtime.instances[2]
                .iter()
                .any(|i| i.parent.unwrap().0 == old_parent)
        );
    }

    #[test]
    fn turbulence_changes_velocity_and_rope_draws_a_bounded_path() {
        let mut value = resource();
        value["emitter"] = json!([{"name":"sphererandom","distancemin":0,"distancemax":0,"rate":0,"instantaneous":1}]);
        value["initializer"] = json!([{"name":"lifetimerandom","min":4,"max":4},{"name":"velocityrandom","min":"100 0 0","max":"100 0 0"}]);
        value["operator"] = json!([{"name":"movement"},{"name":"turbulence","speedmin":100,"speedmax":100,"mask":"0.5 0.5 0","timescale":1}]);
        value["renderer"] = json!([{"name":"ropetrail","length":1,"segments":4,"subdivision":2}]);
        let prepared = prepare(&value).unwrap();
        let mut runtime = ParticleRuntime::new(&prepared, 3).unwrap();
        let frame = runtime.advance(1.0).unwrap();
        assert_eq!(frame.live_particles, 1);
        assert!(!frame.batches[0].vertices.is_empty());
        assert!(frame.batches[0].vertices.len() <= (4 - 1) * 3 * 6);
        let p = &runtime.instances[0][0].particles[0];
        assert!(p.velocity[0] != 100.0 || p.velocity[1] != 0.0);
        assert!(p.trail.len() <= 5);
        assert!(
            frame.batches[0]
                .vertices
                .iter()
                .all(|v| v.position.iter().all(|p| p.is_finite()))
        );
    }

    #[test]
    fn texture_aspect_uses_storage_dimensions_and_is_atomic() {
        let prepared = prepare(&resource()).unwrap();
        let mut runtime = ParticleRuntime::new(&prepared, 6).unwrap();
        let square = runtime.advance(0.5).unwrap();
        runtime.configure_material_aspects(&[3.0]).unwrap();
        let tall = runtime.advance(0.5).unwrap();
        let extent = |frame: &ParticleFrame| {
            let vertices = &frame.batches[0].vertices[..6];
            vertices
                .iter()
                .map(|v| v.position[1])
                .fold(f32::NEG_INFINITY, f32::max)
                - vertices
                    .iter()
                    .map(|v| v.position[1])
                    .fold(f32::INFINITY, f32::min)
        };
        assert!((extent(&tall) - 3.0 * extent(&square)).abs() < 1e-4);
        assert!(runtime.configure_material_aspects(&[]).is_err());
        assert!(runtime.configure_material_aspects(&[0.0]).is_err());
        assert!(runtime.configure_material_aspects(&[f32::NAN]).is_err());
        assert_eq!(runtime.advance(0.5).unwrap(), tall);
        assert_eq!(prepared.max_vertices().unwrap(), 4 * 6);
    }

    #[test]
    fn vertex_layout_matches_streaming_vbo_attributes() {
        assert_eq!(std::mem::size_of::<ParticleVertex>(), 44);
        assert_eq!(std::mem::align_of::<ParticleVertex>(), 4);
        assert_eq!(std::mem::offset_of!(ParticleVertex, position), 0);
        assert_eq!(std::mem::offset_of!(ParticleVertex, uv), 8);
        assert_eq!(std::mem::offset_of!(ParticleVertex, uv_next), 16);
        assert_eq!(std::mem::offset_of!(ParticleVertex, uv_blend), 24);
        assert_eq!(std::mem::offset_of!(ParticleVertex, color), 28);
    }

    #[test]
    fn authored_starttime_warms_particles_and_random_ranges_keep_endpoints() {
        let mut value = resource();
        value["starttime"] = json!(9);
        let prepared = prepare(&value).unwrap();
        let mut runtime = ParticleRuntime::new(&prepared, 12).unwrap();
        assert!(runtime.advance(0.0).unwrap().live_particles > 0);
        let mut rng = Random(13);
        for _ in 0..1000 {
            let value = rng.range(RandomRange {
                min: 0.6,
                max: 0.5,
                exponent: 1.0,
            });
            assert!((0.5..=0.60000003).contains(&value));
        }
        let mut pathological = resource();
        pathological["renderer"] = json!([{"name":"ropetrail","segments":32,"subdivision":32}]);
        pathological["maxcount"] = json!(MAX_PARTICLES);
        assert!(prepare(&pathological).unwrap().max_vertices().is_err());
    }

    fn atlas() -> AnimationMetadata {
        AnimationMetadata {
            version: 2,
            width: 128,
            height: 32,
            frames: (0..4)
                .map(|index| lwe_scene::AnimationFrame {
                    image: 0,
                    duration: 0.0,
                    coordinates: [index as f32 * 32.0, 0.0, 32.0, 0.0, 0.0, 32.0],
                })
                .collect(),
        }
    }

    #[test]
    fn atlas_sequence_blends_by_particle_lifetime_and_random_frame_stays_fixed() {
        let mut value = resource();
        value["sequencemultiplier"] = json!(2);
        let prepared = prepare(&value).unwrap();
        let system = &prepared.systems[0];
        let mut p = make_particle(
            system,
            &system.emitters[0],
            prepared.modifiers,
            &mut Random(7),
            1,
        );
        p.life = 4.0;
        p.age = 1.25;
        let animation = atlas();
        let (uv, next, blend, aspect) = atlas_frame(Some(&animation), system, &p, 9.0);
        assert_eq!(uv, [0.5, 0.0, 0.25, 1.0]);
        assert_eq!(next, [0.75, 0.0, 0.25, 1.0]);
        assert_eq!((blend, aspect), (0.5, 1.0));
        p.age = 2.0;
        assert_eq!(atlas_frame(Some(&animation), system, &p, 9.0).0[0], 0.0);
        let mut runtime = ParticleRuntime::new(&prepared, 7).unwrap();
        runtime
            .configure_material_animations(&[Some(animation.clone())])
            .unwrap();
        let before = runtime.advance(0.5).unwrap();
        assert!(before.batches[0].vertices.iter().all(|v| {
            v.uv.iter()
                .chain(&v.uv_next)
                .all(|c| (0.0..=1.0).contains(c))
        }));
        assert_eq!(runtime.advance(0.5).unwrap(), before);
        let mut invalid = animation.clone();
        invalid.frames[0].coordinates[0] = 129.0;
        assert!(
            runtime
                .configure_material_animations(&[Some(invalid)])
                .is_err()
        );
        assert_eq!(runtime.advance(0.5).unwrap(), before);
        assert!(runtime.configure_material_animations(&[None]).is_err());
        value["animationmode"] = json!("randomframe");
        let prepared = prepare(&value).unwrap();
        let system = &prepared.systems[0];
        let mut p = make_particle(
            system,
            &system.emitters[0],
            prepared.modifiers,
            &mut Random(8),
            1,
        );
        let first = atlas_frame(Some(&animation), system, &p, 1.0);
        p.age = p.life * 0.9;
        assert_eq!(atlas_frame(Some(&animation), system, &p, 1.0), first);
        assert_eq!(first.0, first.1);
        assert_eq!(first.2, 0.0);
    }

    #[test]
    fn static_child_capacity_and_independent_warmup_do_not_spawn_per_particle() {
        let mut root = resource();
        root["children"] = json!([{"name":"particles/child.json"}]);
        let mut child = resource();
        child["starttime"] = json!(2);
        let prepared = PreparedParticles::prepare(
            &json!({"particle":"particles/root.json","instanceoverride":null}),
            &mut |path| {
                Ok(if path.ends_with("root.json") {
                    root.clone()
                } else {
                    child.clone()
                })
            },
        )
        .unwrap();
        assert_eq!(prepared.particle_capacity, 8);
        assert_eq!(prepared.max_vertices().unwrap(), 48);
        let mut runtime = ParticleRuntime::new(&prepared, 18).unwrap();
        let frame = runtime.advance(0.0).unwrap();
        assert!(frame.batches[0].vertices.is_empty());
        assert!(!frame.batches[1].vertices.is_empty());
        runtime.advance(3.0).unwrap();
        assert_eq!(runtime.instances[1].len(), 1);
    }

    #[test]
    fn additional_particle_components_execute_and_unmodeled_axes_still_fail() {
        let mut value = resource();
        value["initializer"] = json!([
            {"name":"lifetimerandom","min":4,"max":4},
            {"name":"sizerandom","min":20,"max":20},
            {"name":"rotationrandom","min":"0 0 1","max":"0 0 1"},
            {"name":"angularvelocityrandom","min":"0 0 2","max":"0 0 2"},
            {"name":"turbulentvelocityrandom","scale":0,"speedmin":10,"speedmax":10}
        ]);
        value["operator"] = json!([
            {"name":"movement"},{"name":"angularmovement"},
            {"name":"sizechange","startvalue":1,"endvalue":3},
            {"name":"colorchange","startvalue":"1 1 1","endvalue":"0 0 0"}
        ]);
        value["renderer"] = json!([{"name":"spritetrail","length":1,"minlength":1,"maxlength":1}]);
        let prepared = prepare(&value).unwrap();
        let system = &prepared.systems[0];
        let mut p = make_particle(
            system,
            &system.emitters[0],
            prepared.modifiers,
            &mut Random(8),
            1,
        );
        assert!((p.velocity[0]).abs() < 1e-12);
        assert_eq!(p.velocity[1], 10.0);
        assert_eq!((p.angle, p.angular_velocity), (1.0, 2.0));
        update_particle(&mut p, system, 0.5, 0.5, 1.0);
        assert_eq!(p.angle, 2.0);
        p.age = 2.0;
        assert_eq!(appearance(&p, &system.operators).0, 40.0);
        assert_eq!(particle_color(&p, &system.operators), [0.5; 3]);
        value["initializer"][2]["max"] = json!("1 0 0");
        assert!(prepare(&value).is_err());
        value["initializer"][2]["max"] = json!("0 0 1");
        value["animationmode"] = json!("unknown");
        assert!(prepare(&value).is_err());
    }

    #[test]
    #[ignore = "requires the private original Yakuza project and built-in assets"]
    fn original_yakuza_all_particle_trees_prepare_decode_and_simulate() {
        let project = std::env::var("LWE_YAKUZA_PROJECT").unwrap();
        let builtin = std::env::var("LWE_YAKUZA_ASSETS").unwrap();
        let assets = lwe_scene::SceneAssets::open(
            std::path::Path::new(&project),
            Some(std::path::Path::new(&builtin)),
        )
        .unwrap();
        let scene: Value =
            serde_json::from_slice(&assets.read("scene.json", 1024 * 1024).unwrap()).unwrap();
        let mut roots = 0;
        for object in scene["objects"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|object| object.get("particle").is_some())
        {
            let mut object = object.clone();
            if object["visible"].is_object() {
                object["visible"] = object["visible"]["value"].clone();
            }
            let prepared = PreparedParticles::prepare(&object, &mut |path| {
                Ok(serde_json::from_slice(&assets.read(path, 1024 * 1024)?)?)
            })
            .unwrap();
            let mut runtime = ParticleRuntime::new(&prepared, 2228594093).unwrap();
            let mut animations = Vec::new();
            let mut aspects = Vec::new();
            for path in prepared.material_paths() {
                let material: Value =
                    serde_json::from_slice(&assets.read(path, 1024 * 1024).unwrap()).unwrap();
                let name = material["passes"][0]["textures"][0].as_str().unwrap();
                let texture = lwe_scene::Texture::decode_atlas(
                    &assets
                        .read(&format!("materials/{name}.tex"), 16 * 1024 * 1024)
                        .unwrap(),
                )
                .unwrap();
                aspects.push(texture.height as f32 / texture.width as f32);
                animations.push(texture.metadata.animation);
            }
            runtime.configure_material_aspects(&aspects).unwrap();
            runtime.configure_material_animations(&animations).unwrap();
            let mut nonempty = vec![false; prepared.systems.len()];
            let mut peak = 0;
            for tick in 0..=300 {
                let frame = runtime.advance(tick as f64 / 10.0).unwrap();
                peak = peak.max(frame.live_particles);
                for batch in frame.batches {
                    nonempty[batch.material] |= !batch.vertices.is_empty();
                    assert!(batch.vertices.iter().all(|v| {
                        v.position
                            .iter()
                            .chain(&v.uv)
                            .chain(&v.uv_next)
                            .chain(&v.color)
                            .chain(std::iter::once(&v.uv_blend))
                            .all(|c| c.is_finite())
                    }));
                }
            }
            assert!(
                nonempty.iter().all(|value| *value),
                "{}",
                prepared.systems[0].path
            );
            eprintln!(
                "{} systems={} capacity={} vertices={} work={} peak={}",
                prepared.systems[0].path,
                prepared.systems.len(),
                prepared.particle_capacity,
                prepared.max_vertices().unwrap(),
                prepared.max_work().unwrap(),
                peak
            );
            roots += 1;
        }
        assert_eq!(roots, 6);
    }
}
