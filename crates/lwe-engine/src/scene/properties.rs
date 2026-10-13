//! Bounded declarative defaults and pure scalar SceneScript property bindings.
//!
//! Direct user bindings and exact combo equality conditions are admitted,
//! together with exact `{"script":module,"value":scalar}` bindings
//! accepted by the independent bounded ECMAScript runtime. Broader scripts,
//! expressions, and unknown wrapper fields stay gated.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;
use serde_json::{Map, Value};

use super::{script, unsupported};

const MAX_PROPERTIES: usize = 256;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;
const MAX_LABEL_BYTES: usize = 4096;
const MAX_COMBO_OPTIONS: usize = 256;
const MAX_COMBO_VALUE_BYTES: usize = 256;
const MAX_SCRIPTS: usize = 64;
const MAX_SCRIPT_BYTES: usize = 256 * 1024;

/// The finite, declarative controls admitted by the Scene runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    Bool,
    Slider,
    Color,
    Combo,
}

/// Values exposed to the property editor. Colors are normalized RGB triples.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Slider(f64),
    Color([f32; 3]),
    Combo(String),
}

/// An author-declared selection, retaining its display label and stored value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PropertyOption {
    pub label: String,
    pub value: String,
}

/// An author-declared control and its current literal value.
///
/// `label` preserves the project's `text` verbatim, falling back to its name
/// when absent. Optional bounds, step and order remain absent when undeclared.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PropertyDefinition {
    pub name: String,
    pub label: String,
    pub kind: PropertyKind,
    pub value: PropertyValue,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub order: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<PropertyOption>,
}

/// List validated author controls in order, with a stable name tie-breaker.
///
/// This CPU-only operation performs no resource I/O or code evaluation. Scene
/// preparation still owns a separate, shared project-and-resources traversal
/// budget through `ProjectProperties`.
pub fn property_definitions(project: &Value) -> Result<Vec<PropertyDefinition>> {
    let properties = ProjectProperties::from_project(project)?;
    let Some(declarations) = properties.declarations else {
        return Ok(Vec::new());
    };
    let mut definitions = Vec::with_capacity(declarations.len());
    for (name, declaration) in declarations {
        if name.is_empty() || name.len() > 256 {
            return Err(invalid("invalid project property name"));
        }
        let declaration = validated_declaration(declaration)?;
        let label = declaration
            .object
            .get("text")
            .map(|text| {
                text.as_str()
                    .filter(|text| text.len() <= MAX_LABEL_BYTES)
                    .ok_or_else(|| invalid("invalid project property label"))
            })
            .transpose()?
            .unwrap_or(name);
        let order = declaration.object.get("order").map(scalar).transpose()?;
        definitions.push(PropertyDefinition {
            name: name.clone(),
            label: label.to_owned(),
            kind: declaration.kind,
            value: declaration.value,
            min: declaration.min,
            max: declaration.max,
            step: declaration.step,
            order,
            options: declaration.options,
        });
    }
    definitions.sort_by(|left, right| {
        left.order
            .unwrap_or(0.0)
            .total_cmp(&right.order.unwrap_or(0.0))
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(definitions)
}

/// Validate every override before replacing any author value.
///
/// Unknown names, types, non-finite values and out-of-range values are errors.
/// RGB overrides accept a normalized three-element array or the project's RGB
/// string notation, and are stored in that original notation. Omitted controls
/// and all other project metadata are retained exactly. Step is a positive
/// editor increment; it does not quantize otherwise valid slider values.
pub fn apply_property_overrides(
    project: &mut Value,
    overrides: &BTreeMap<String, Value>,
) -> Result<()> {
    // Validate the unmodified project first: an override cannot erase a script,
    // expression, unsupported type or invalid declared default.
    let definitions = property_definitions(project)?;
    let mut updates = Vec::with_capacity(overrides.len().min(MAX_PROPERTIES));
    for (name, candidate) in overrides {
        let definition = definitions
            .iter()
            .find(|definition| definition.name == *name)
            .ok_or_else(|| invalid("property override has no project declaration"))?;
        let normalized_color = match definition.kind {
            PropertyKind::Color => {
                let rgb = override_color(candidate)?;
                Some(Value::String(format!("{} {} {}", rgb[0], rgb[1], rgb[2])))
            }
            _ => None,
        };
        let candidate = normalized_color.as_ref().unwrap_or(candidate);
        validated_value(
            definition.kind,
            candidate,
            definition.min,
            definition.max,
            true,
            &definition.options,
        )?;
        updates.push((name, candidate.clone()));
    }
    if updates.is_empty() {
        return Ok(());
    }
    let declarations = project
        .get_mut("general")
        .and_then(|general| general.get_mut("properties"))
        .and_then(Value::as_object_mut)
        .expect("property definitions validated the declaration map");
    for (name, value) in updates {
        declarations
            .get_mut(name)
            .and_then(Value::as_object_mut)
            .expect("property definitions validated each declaration")
            .insert("value".to_owned(), value);
    }
    Ok(())
}

pub(super) struct ProjectProperties<'a> {
    declarations: Option<&'a Map<String, Value>>,
    remaining_nodes: Cell<usize>,
    remaining_scripts: Cell<usize>,
    remaining_script_bytes: Cell<usize>,
    script_values: RefCell<BTreeMap<(String, u64), f64>>,
}

impl<'a> ProjectProperties<'a> {
    pub(super) fn from_project(project: &'a Value) -> Result<Self> {
        let declarations = project
            .get("general")
            .and_then(|general| general.get("properties"))
            .map(|properties| {
                properties.as_object().ok_or_else(|| {
                    unsupported("property_binding", "invalid project property declarations")
                })
            })
            .transpose()?;
        if declarations.is_some_and(|declarations| declarations.len() > MAX_PROPERTIES) {
            return Err(unsupported(
                "property_binding",
                "project property count exceeds the preparation limit",
            ));
        }
        let properties = Self {
            declarations,
            remaining_nodes: Cell::new(MAX_NODES),
            remaining_scripts: Cell::new(MAX_SCRIPTS),
            remaining_script_bytes: Cell::new(MAX_SCRIPT_BYTES),
            script_values: RefCell::new(BTreeMap::new()),
        };
        // Project declarations stay literal; a script cannot replace their
        // typed defaults or metadata. Resource bindings are evaluated later.
        validate_document(project, 0, &properties.remaining_nodes, None)?;
        Ok(properties)
    }

    pub(super) fn resolve_document(&self, document: &mut Value) -> Result<()> {
        validate_document(document, 0, &self.remaining_nodes, Some(self))?;
        self.resolve(document)
    }

    pub(super) fn shader_value(&self, name: &str) -> Result<Value> {
        if name.is_empty() || name.len() > 256 {
            return Err(invalid("invalid shader property name"));
        }
        let declaration = self
            .declarations
            .and_then(|declarations| declarations.get(name))
            .ok_or_else(|| invalid("shader property has no project declaration"))?;
        let declaration = validated_declaration(declaration)?;
        Ok(match declaration.value {
            PropertyValue::Bool(value) => Value::from(u8::from(value)),
            PropertyValue::Slider(value) => Value::from(value),
            PropertyValue::Color(value) => serde_json::to_value(value)?,
            PropertyValue::Combo(_) => {
                return Err(invalid("combo properties cannot be used as shader values"));
            }
        })
    }

    fn resolve(&self, value: &mut Value) -> Result<()> {
        match value {
            Value::Object(object) => {
                if let Some(source) = object.get("script").and_then(Value::as_str) {
                    let input = object["value"]
                        .as_f64()
                        .expect("scalar script input validated before resolution");
                    let output = self.script_values.borrow()[&(source.to_owned(), input.to_bits())];
                    *value = Value::from(output);
                    return Ok(());
                }
                if object.contains_key("binding") {
                    return Err(unsupported(
                        "property_binding",
                        "property binding expressions are not supported",
                    ));
                }
                if let Some(user) = object.get("user") {
                    if object.len() != 2 || !object.contains_key("value") {
                        return Err(unsupported(
                            "property_binding",
                            "unsupported user property binding shape",
                        ));
                    }
                    let (name, condition) = user_binding(user)?;
                    let declaration = self
                        .declarations
                        .and_then(|declarations| declarations.get(name))
                        .ok_or_else(|| {
                            unsupported(
                                "property_binding",
                                "user binding has no project declaration",
                            )
                        })?;
                    let fallback = &object["value"];
                    *value = if let Some(condition) = condition {
                        let declaration = validated_declaration(declaration)?;
                        if declaration.kind != PropertyKind::Combo || !fallback.is_boolean() {
                            return Err(invalid("combo conditions require a boolean binding"));
                        }
                        validated_value(
                            PropertyKind::Combo,
                            condition,
                            None,
                            None,
                            true,
                            &declaration.options,
                        )?;
                        Value::Bool(declaration.default == condition)
                    } else {
                        default_value(declaration, fallback)?
                    };
                } else {
                    for child in object.values_mut() {
                        self.resolve(child)?;
                    }
                }
            }
            Value::Array(array) => {
                for child in array {
                    self.resolve(child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_script(&self, object: &Map<String, Value>) -> Result<()> {
        if object.len() != 2 || !object.contains_key("value") {
            return Err(unsupported(
                "scenescript",
                "unsupported script binding shape",
            ));
        }
        let source = object
            .get("script")
            .and_then(Value::as_str)
            .ok_or_else(|| unsupported("scenescript", "script binding requires a module source"))?;
        let input = object["value"]
            .as_f64()
            .filter(|value| value.is_finite() && (*value as f32).is_finite())
            .ok_or_else(|| {
                unsupported(
                    "scenescript",
                    "script binding requires a finite scalar input",
                )
            })?;
        let remaining = self.remaining_scripts.get();
        let bytes = self.remaining_script_bytes.get();
        if remaining == 0 || source.len() > bytes {
            return Err(unsupported(
                "scenescript",
                "project scripts exceed the preparation limit",
            ));
        }
        self.remaining_scripts.set(remaining - 1);
        self.remaining_script_bytes.set(bytes - source.len());
        let key = (source.to_owned(), input.to_bits());
        if self.script_values.borrow().contains_key(&key) {
            return Ok(());
        }
        let mut snapshot = script::PropertySnapshot::new();
        if let Some(declarations) = self.declarations {
            for (name, declaration) in declarations {
                match validated_declaration(declaration)?.value {
                    PropertyValue::Bool(value) => {
                        snapshot.insert(name.clone(), script::ScalarProperty::Bool(value));
                    }
                    PropertyValue::Slider(value) => {
                        snapshot.insert(name.clone(), script::ScalarProperty::Number(value));
                    }
                    // Color/vector access requires SceneScript Vec3 semantics
                    // and remains outside this scalar subset.
                    PropertyValue::Color(_) | PropertyValue::Combo(_) => {}
                }
            }
        }
        let output = script::evaluate(source, input, &snapshot)?;
        self.script_values.borrow_mut().insert(key, output);
        Ok(())
    }
}

// Scan the bounded document before replacing bindings so a JS declaration in
// another object cannot lose its existing SceneScript blocker to an earlier
// unsupported declarative binding. The second traversal can only shrink it.
fn validate_document(
    value: &Value,
    depth: usize,
    nodes: &Cell<usize>,
    properties: Option<&ProjectProperties<'_>>,
) -> Result<()> {
    let remaining = nodes.get();
    if depth > MAX_DEPTH || remaining == 0 {
        return Err(invalid("property traversal exceeds the preparation limit"));
    }
    nodes.set(remaining - 1);
    match value {
        Value::Object(object) => {
            if object
                .keys()
                .any(|key| matches!(key.to_ascii_lowercase().as_str(), "script" | "scenescript"))
            {
                if let (true, Some(properties)) = (object.contains_key("script"), properties) {
                    properties.validate_script(object)?;
                } else {
                    return Err(unsupported(
                        "scenescript",
                        "resource contains unsupported script bindings",
                    ));
                }
            }
            for child in object.values() {
                validate_document(child, depth + 1, nodes, properties)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                validate_document(child, depth + 1, nodes, properties)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn invalid(detail: &'static str) -> anyhow::Error {
    unsupported("property_binding", detail)
}

fn scalar(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .filter(|number| number.is_finite() && (*number as f32).is_finite())
        .ok_or_else(|| invalid("invalid finite slider property value"))
}

fn color(value: &Value) -> Result<[f32; 3]> {
    let numbers = match value {
        Value::String(string) if string.len() <= 256 => string
            .split_whitespace()
            .take(4)
            .map(str::parse::<f64>)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| invalid("invalid color property value"))?,
        _ => return Err(invalid("invalid color property value")),
    };
    let values: [f64; 3] = numbers
        .try_into()
        .map_err(|_| invalid("color properties require three components"))?;
    if !values
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
    {
        return Err(invalid(
            "color property components must be finite and within 0..1",
        ));
    }
    Ok(values.map(|value| value as f32))
}

fn override_color(value: &Value) -> Result<[f32; 3]> {
    let Some(components) = value.as_array() else {
        return color(value);
    };
    if components.len() != 3 {
        return Err(invalid("color properties require three components"));
    }
    let mut rgb = [0.0; 3];
    for (channel, component) in rgb.iter_mut().zip(components) {
        let number = scalar(component)?;
        if !(0.0..=1.0).contains(&number) {
            return Err(invalid(
                "color property components must be finite and within 0..1",
            ));
        }
        *channel = number as f32;
    }
    Ok(rgb)
}

struct ValidatedDeclaration<'a> {
    object: &'a Map<String, Value>,
    default: &'a Value,
    kind: PropertyKind,
    value: PropertyValue,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    options: Vec<PropertyOption>,
}

fn validated_declaration(declaration: &Value) -> Result<ValidatedDeclaration<'_>> {
    let object = declaration
        .as_object()
        .ok_or_else(|| invalid("invalid project property declaration"))?;
    if object.keys().any(|key| {
        matches!(
            key.to_ascii_lowercase().as_str(),
            "binding" | "expr" | "expression" | "user"
        )
    }) {
        return Err(invalid(
            "property declaration expressions are not supported",
        ));
    }
    let default = object
        .get("value")
        .ok_or_else(|| invalid("project property has no default value"))?;
    let kind = match object.get("type").and_then(Value::as_str) {
        Some("bool") => PropertyKind::Bool,
        Some("slider") => PropertyKind::Slider,
        Some("color") => PropertyKind::Color,
        Some("combo") => PropertyKind::Combo,
        _ => return Err(invalid("user property type is not in the validated subset")),
    };
    let (min, max, step) = if kind == PropertyKind::Slider {
        let min = object.get("min").map(scalar).transpose()?;
        let max = object.get("max").map(scalar).transpose()?;
        let step = object.get("step").map(scalar).transpose()?;
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            return Err(invalid("invalid slider property range"));
        }
        if step.is_some_and(|step| step <= 0.0) {
            return Err(invalid("slider property step must be positive and finite"));
        }
        (min, max, step)
    } else {
        (None, None, None)
    };
    let options = if kind == PropertyKind::Combo {
        combo_options(
            object
                .get("options")
                .ok_or_else(|| invalid("combo property has no options"))?,
        )?
    } else {
        Vec::new()
    };
    let value = validated_value(kind, default, min, max, true, &options)?;
    Ok(ValidatedDeclaration {
        object,
        default,
        kind,
        value,
        min,
        max,
        step,
        options,
    })
}

fn validated_value(
    kind: PropertyKind,
    value: &Value,
    min: Option<f64>,
    max: Option<f64>,
    enforce_range: bool,
    options: &[PropertyOption],
) -> Result<PropertyValue> {
    match kind {
        PropertyKind::Bool => value
            .as_bool()
            .map(PropertyValue::Bool)
            .ok_or_else(|| invalid("boolean property binding requires boolean values")),
        PropertyKind::Slider => {
            let number = scalar(value)?;
            if enforce_range
                && (min.is_some_and(|min| number < min) || max.is_some_and(|max| number > max))
            {
                return Err(invalid(
                    "slider property value is outside its declared range",
                ));
            }
            Ok(PropertyValue::Slider(number))
        }
        PropertyKind::Color => color(value).map(PropertyValue::Color),
        PropertyKind::Combo => {
            let value = combo_value(value)?;
            if enforce_range && !options.iter().any(|option| option.value == value) {
                return Err(invalid(
                    "combo property value is outside its declared options",
                ));
            }
            Ok(PropertyValue::Combo(value.to_owned()))
        }
    }
}

fn default_value(declaration: &Value, fallback: &Value) -> Result<Value> {
    // Project and document were already scanned once under the shared budget.
    // Do not rescan declaration metadata for every use of the same property.
    let declaration = validated_declaration(declaration)?;
    // A binding's fallback is not used when a declaration exists. Validate its
    // literal type, while the declared or overridden value enforces its bounds.
    validated_value(
        declaration.kind,
        fallback,
        declaration.min,
        declaration.max,
        false,
        &declaration.options,
    )?;
    Ok(declaration.default.clone())
}

fn combo_value(value: &Value) -> Result<&str> {
    value
        .as_str()
        .filter(|value| {
            !value.is_empty() && value.len() <= MAX_COMBO_VALUE_BYTES && !value.contains('\0')
        })
        .ok_or_else(|| invalid("combo property requires a bounded nonempty string"))
}

fn combo_options(value: &Value) -> Result<Vec<PropertyOption>> {
    let options = value
        .as_array()
        .filter(|options| !options.is_empty() && options.len() <= MAX_COMBO_OPTIONS)
        .ok_or_else(|| invalid("combo property requires a bounded nonempty option list"))?;
    let mut validated: Vec<PropertyOption> = Vec::with_capacity(options.len());
    for option in options {
        let object = option
            .as_object()
            .filter(|object| object.len() == 2)
            .ok_or_else(|| invalid("invalid combo property option"))?;
        let label = object
            .get("label")
            .and_then(Value::as_str)
            .filter(|label| label.len() <= MAX_LABEL_BYTES)
            .ok_or_else(|| invalid("invalid combo property option label"))?;
        let value = combo_value(
            object
                .get("value")
                .ok_or_else(|| invalid("combo option has no value"))?,
        )?;
        if validated.iter().any(|option| option.value == value) {
            return Err(invalid("combo property options contain duplicate values"));
        }
        validated.push(PropertyOption {
            label: label.to_owned(),
            value: value.to_owned(),
        });
    }
    Ok(validated)
}

fn user_binding(user: &Value) -> Result<(&str, Option<&Value>)> {
    let (name, condition) = if let Some(name) = user.as_str() {
        (name, None)
    } else {
        let object = user
            .as_object()
            .filter(|object| object.len() == 2)
            .ok_or_else(|| invalid("unsupported conditional user binding shape"))?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("conditional user binding requires a property name"))?;
        let condition = object
            .get("condition")
            .ok_or_else(|| invalid("conditional user binding requires an equality condition"))?;
        (name, Some(condition))
    };
    if name.is_empty() || name.len() > 256 {
        return Err(invalid("user binding must name one property"));
    }
    Ok((name, condition))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::unsupported_feature_code;
    use serde_json::json;

    fn resolve(project: Value, mut document: Value) -> Result<Value> {
        ProjectProperties::from_project(&project)?.resolve_document(&mut document)?;
        Ok(document)
    }

    #[test]
    fn project_defaults_replace_binding_fallbacks_with_matching_types() {
        let project = json!({"general":{"properties":{
            "enabled":{"type":"bool","value":false},
            "opacity":{"type":"slider","value":0.25,"min":0,"max":1},
            "tint":{"type":"color","value":"0.1 0.2 0.3"}
        }}});
        let resolved = resolve(
            project,
            json!({
                "visible":{"user":"enabled","value":true},
                "alpha":{"user":"opacity","value":1},
                "color":{"user":"tint","value":"1 1 1"}
            }),
        )
        .unwrap();
        assert_eq!(
            resolved,
            json!({"visible":false,"alpha":0.25,"color":"0.1 0.2 0.3"})
        );
    }

    fn editable_project() -> Value {
        json!({"general":{"properties":{
            "enabled":{"type":"bool","value":false,"text":"  作者开关  ","order":20},
            "gain":{"type":"slider","value":0.4,"min":0.1,"max":0.9,
                "step":0.03,"text":"Author gain","order":10,"precision":3,"fraction":true},
            "accent":{"type":"color","value":"0.125 0.5 0.75","text":"Author tint","order":10},
            "plain":{"type":"bool","value":true}
        }},"title":"Untouched metadata"})
    }

    fn combo_project() -> Value {
        json!({"general":{"properties":{
            "selection":{"type":"combo","text":"Author selection","value":"first",
                "options":[{"label":"First choice","value":"first"},{"label":"Second choice","value":"second"}]}
        }}})
    }

    #[test]
    fn combo_options_preserve_labels_and_values_and_resolve_boolean_conditions() {
        let mut project = combo_project();
        let definitions = property_definitions(&project).unwrap();
        assert_eq!(definitions[0].kind, PropertyKind::Combo);
        assert_eq!(definitions[0].value, PropertyValue::Combo("first".into()));
        assert_eq!(
            definitions[0].options[1],
            PropertyOption {
                label: "Second choice".into(),
                value: "second".into()
            }
        );
        assert_eq!(
            serde_json::to_value(&definitions).unwrap()[0]["options"][0],
            json!({"label":"First choice","value":"first"})
        );
        let bindings = json!({
            "first":{"user":{"name":"selection","condition":"first"},"value":false},
            "second":{"user":{"name":"selection","condition":"second"},"value":true},
            "direct":{"user":"selection","value":"second"}
        });
        assert_eq!(
            resolve(project.clone(), bindings.clone()).unwrap(),
            json!({"first":true,"second":false,"direct":"first"})
        );
        apply_property_overrides(
            &mut project,
            &BTreeMap::from([("selection".into(), json!("second"))]),
        )
        .unwrap();
        assert_eq!(
            resolve(project, bindings).unwrap(),
            json!({"first":false,"second":true,"direct":"second"})
        );
    }

    #[test]
    fn invalid_combo_declarations_and_overrides_fail_without_modifying_project() {
        let original = combo_project();
        for candidate in [
            json!("missing"),
            json!(0),
            json!(""),
            json!("first\0"),
            json!("x".repeat(MAX_COMBO_VALUE_BYTES + 1)),
        ] {
            let mut project = original.clone();
            assert!(
                apply_property_overrides(
                    &mut project,
                    &BTreeMap::from([("selection".into(), candidate)])
                )
                .is_err()
            );
            assert_eq!(project, original);
        }
        for options in [
            json!([]),
            json!(null),
            json!([{"label":"First","value":0}]),
            json!([{"label":"First","value":"first"},{"label":"Duplicate","value":"first"}]),
            json!([{"label":"First","value":"first","script":"code"}]),
            json!([{"label":0,"value":"first"}]),
            json!([{"label":"x".repeat(MAX_LABEL_BYTES + 1),"value":"first"}]),
            json!([{"label":"Other","value":"other"}]),
            json!(
                (0..=MAX_COMBO_OPTIONS)
                    .map(|index| json!({"label":"Choice","value":index.to_string()}))
                    .collect::<Vec<_>>()
            ),
        ] {
            let mut project = original.clone();
            project["general"]["properties"]["selection"]["options"] = options;
            assert!(property_definitions(&project).is_err());
        }
        for binding in [
            json!({"user":{"name":"selection","condition":"missing"},"value":true}),
            json!({"user":{"name":"selection","condition":"first"},"value":1}),
            json!({"user":{"name":"selection","condition":1},"value":true}),
            json!({"user":{"name":"selection","condition":"first","invert":true},"value":true}),
            json!({"user":{"name":"selection"},"value":true}),
        ] {
            assert!(resolve(original.clone(), binding).is_err());
        }
        assert!(
            ProjectProperties::from_project(&original)
                .unwrap()
                .shader_value("selection")
                .is_err()
        );
    }

    #[test]
    fn definitions_expose_typed_values_and_preserve_author_labels_and_order() {
        let definitions = property_definitions(&editable_project()).unwrap();
        assert_eq!(
            definitions
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            ["plain", "accent", "gain", "enabled"]
        );
        assert_eq!(definitions[0].label, "plain");
        assert_eq!(definitions[0].value, PropertyValue::Bool(true));
        assert_eq!(definitions[0].order, None);
        assert_eq!(
            definitions[1].value,
            PropertyValue::Color([0.125, 0.5, 0.75])
        );
        assert_eq!(definitions[2].value, PropertyValue::Slider(0.4));
        assert_eq!(definitions[2].min, Some(0.1));
        assert_eq!(definitions[2].max, Some(0.9));
        assert_eq!(definitions[2].step, Some(0.03));
        assert_eq!(definitions[3].label, "  作者开关  ");
        let encoded = serde_json::to_value(&definitions).unwrap();
        assert_eq!(encoded[0]["kind"], "bool");
        assert_eq!(encoded[0]["value"], true);
        assert_eq!(encoded[0]["min"], Value::Null);
        assert_eq!(encoded[1]["kind"], "color");
        assert_eq!(encoded[1]["value"], json!([0.125, 0.5, 0.75]));
        assert_eq!(encoded[2]["kind"], "slider");
        assert_eq!(encoded[2]["value"], 0.4);
        assert_eq!(encoded[3]["label"], "  作者开关  ");
    }

    #[test]
    fn overrides_change_only_selected_literals_and_resolve_with_the_same_rules() {
        let mut project = editable_project();
        let original = project.clone();
        apply_property_overrides(
            &mut project,
            &BTreeMap::from([
                ("enabled".to_owned(), json!(true)),
                ("gain".to_owned(), json!(0.43)),
                ("accent".to_owned(), json!([0.25, 0.5, 0.875])),
            ]),
        )
        .unwrap();
        let mut expected = original.clone();
        expected["general"]["properties"]["enabled"]["value"] = json!(true);
        expected["general"]["properties"]["gain"]["value"] = json!(0.43);
        expected["general"]["properties"]["accent"]["value"] = json!("0.25 0.5 0.875");
        assert_eq!(project, expected);
        assert_eq!(
            resolve(
                project,
                json!({
                    "enabled":{"user":"enabled","value":false},
                    "gain":{"user":"gain","value":99},
                    "accent":{"user":"accent","value":"0 0 0"},
                    "plain":{"user":"plain","value":false}
                })
            )
            .unwrap(),
            json!({"enabled":true,"gain":0.43,"accent":"0.25 0.5 0.875","plain":true})
        );
        let mut untouched = original.clone();
        apply_property_overrides(&mut untouched, &BTreeMap::new()).unwrap();
        assert_eq!(untouched, original);
    }

    #[test]
    fn invalid_overrides_retain_the_entire_original_project() {
        let original = editable_project();
        for (name, candidate) in [
            ("zz_unknown", json!(false)),
            ("enabled", json!(1)),
            ("enabled", json!("true")),
            ("gain", json!("0.5")),
            ("gain", json!(0.099)),
            ("gain", json!(0.901)),
            ("gain", json!(1e100)),
            ("accent", json!([0, 0])),
            ("accent", json!([0, 0, 0, 0])),
            ("accent", json!([0, "NaN", 0])),
            ("accent", json!([0, 1e100, 0])),
            ("accent", json!([0, -0.01, 0])),
            ("accent", json!([0, 1.00000000001, 0])),
            ("accent", json!("NaN 0 0")),
            ("accent", json!("Infinity 0 0")),
            ("accent", json!("0 1.00000000001 0")),
            ("accent", json!({"value":"0 0 0","binding":"expression"})),
        ] {
            let mut project = original.clone();
            let mut overrides = BTreeMap::from([("plain".to_owned(), json!(false))]);
            overrides.insert(name.to_owned(), candidate);
            let error = apply_property_overrides(&mut project, &overrides).unwrap_err();
            assert_eq!(unsupported_feature_code(&error), Some("property_binding"));
            assert_eq!(project, original);
        }
        // A valid, lexically earlier change must not escape when a later one
        // fails. This covers transactional validation independent of key order.
        let mut project = original.clone();
        assert!(
            apply_property_overrides(
                &mut project,
                &BTreeMap::from([
                    ("accent".to_owned(), json!([0.25, 0.5, 0.75])),
                    ("enabled".to_owned(), json!(1))
                ])
            )
            .is_err()
        );
        assert_eq!(project, original);
    }

    #[test]
    fn invalid_declarations_cannot_be_hidden_by_valid_overrides() {
        for declaration in [
            json!({"type":"bool","value":{"script":"code","value":true}}),
            json!({"type":"bool","value":{"binding":"expression","value":true}}),
            json!({"type":"bool","value":true,"binding":"expression"}),
            json!({"type":"bool","value":true,"Expression":"expression"}),
            json!({"type":"bool","value":true,"expr":"expression"}),
            json!({"type":"textinput","value":"unsupported"}),
            json!({"type":"slider","value":2,"min":0,"max":1}),
            json!({"type":"bool","value":1}),
            json!({"type":"color","value":"NaN 0 0"}),
            json!({"type":"bool"}),
        ] {
            let original = json!({"general":{"properties":{"flag":declaration}}});
            assert!(property_definitions(&original).is_err());
            let mut project = original.clone();
            assert!(
                apply_property_overrides(
                    &mut project,
                    &BTreeMap::from([("flag".to_owned(), json!(true))])
                )
                .is_err()
            );
            assert_eq!(project, original);
            assert!(apply_property_overrides(&mut project, &BTreeMap::new()).is_err());
            assert_eq!(project, original);
        }
    }

    #[test]
    fn invalid_metadata_and_declarations_are_rejected_even_when_unused() {
        for declaration in [
            json!({"type":"bool","value":true,"text":1}),
            json!({"type":"bool","value":true,"text":"x".repeat(MAX_LABEL_BYTES + 1)}),
            json!({"type":"bool","value":true,"order":"first"}),
            json!({"type":"bool","value":true,"order":1e100}),
            json!({"type":"slider","value":0.5,"min":1,"max":0}),
            json!({"type":"slider","value":0.5,"min":"0"}),
            json!({"type":"slider","value":0.5,"max":1e100}),
            json!({"type":"slider","value":0.5,"step":0}),
            json!({"type":"slider","value":0.5,"step":-0.1}),
            json!({"type":"slider","value":0.5,"step":"0.1"}),
            json!({"type":"slider","value":0.5,"step":1e100}),
            json!({"type":"unknown","value":true}),
            json!({"value":true}),
            json!(false),
        ] {
            let mut project = json!({"general":{"properties":{"unused":declaration}}});
            assert!(property_definitions(&project).is_err());
            assert!(apply_property_overrides(&mut project, &BTreeMap::new()).is_err());
        }
        for name in [String::new(), "x".repeat(257)] {
            let project = json!({"general":{"properties":{name:{"type":"bool","value":true}}}});
            assert!(property_definitions(&project).is_err());
        }
    }

    #[test]
    fn color_string_compatibility_and_declared_slider_boundaries_are_retained() {
        let mut project = editable_project();
        apply_property_overrides(
            &mut project,
            &BTreeMap::from([
                ("accent".to_owned(), json!(" 0.25  0.5\t0.875 ")),
                ("gain".to_owned(), json!(0.1)),
            ]),
        )
        .unwrap();
        assert_eq!(
            project["general"]["properties"]["accent"]["value"],
            "0.25 0.5 0.875"
        );
        apply_property_overrides(
            &mut project,
            &BTreeMap::from([("gain".to_owned(), json!(0.9))]),
        )
        .unwrap();
        assert_eq!(project["general"]["properties"]["gain"]["value"], 0.9);
        let mut absent = json!({});
        assert!(property_definitions(&absent).unwrap().is_empty());
        apply_property_overrides(&mut absent, &BTreeMap::new()).unwrap();
        assert_eq!(absent, json!({}));
        assert!(
            apply_property_overrides(
                &mut absent,
                &BTreeMap::from([("flag".to_owned(), json!(true))])
            )
            .is_err()
        );
        assert_eq!(absent, json!({}));
    }

    #[test]
    fn workshop_365_declarations_resolve_all_six_observed_direct_bindings() {
        // Reduced, authored metadata reproduces the types/defaults and binding
        // shapes read from local 3659880761. No original media is included.
        let project = json!({"general":{"properties":{
            "hdr":{"type":"bool","value":true},
            "newproperty":{"type":"bool","value":true},
            "_1":{"type":"bool","value":true},
            "_2":{"type":"bool","value":true},
            "newproperty3":{"type":"slider","value":1,"min":0,"max":1},
            "newproperty2":{"type":"slider","value":0,"min":0,"max":1}
        }}});
        let scene = json!({
            "general":{"hdr":false,"bloom":{"user":"hdr","value":true}},
            "objects":[
                {"visible":{"user":"_2","value":true},"effects":[
                    {"visible":{"user":"newproperty","value":true}},
                    {"visible":{"user":"_1","value":true}}
                ]},
                {"volume":{"user":"newproperty3","value":1.0}},
                {"volume":{"user":"newproperty2","value":0.0}}
            ]
        });
        let resolved = resolve(project, scene).unwrap();
        assert_eq!(resolved["general"]["hdr"], false);
        assert_eq!(resolved["general"]["bloom"], true);
        assert_eq!(resolved["objects"][0]["visible"], true);
        assert_eq!(resolved["objects"][0]["effects"][0]["visible"], true);
        assert_eq!(resolved["objects"][0]["effects"][1]["visible"], true);
        assert_eq!(resolved["objects"][1]["volume"], 1);
        assert_eq!(resolved["objects"][2]["volume"], 0);
    }

    #[test]
    fn missing_malformed_or_unknown_bindings_are_never_defaulted() {
        let project = json!({"general":{"properties":{
            "flag":{"type":"bool","value":true},
            "slider":{"type":"slider","value":0.5,"min":0,"max":1},
            "text":{"type":"textinput","value":"anything"},
            "missing":{"type":"bool"},
            "wrong":{"type":"bool","value":1},
            "range":{"type":"slider","value":2,"min":0,"max":1}
        }}});
        for binding in [
            json!({"user":"unknown","value":true}),
            json!({"user":{"name":"flag"},"value":true}),
            json!({"user":"flag"}),
            json!({"user":"flag","value":true,"multiply":2}),
            json!({"binding":"flag","value":true}),
            json!({"user":"text","value":"anything"}),
            json!({"user":"missing","value":true}),
            json!({"user":"wrong","value":true}),
            json!({"user":"flag","value":1}),
            json!({"user":"slider","value":"0.5"}),
            json!({"user":"range","value":0.5}),
        ] {
            let error = resolve(project.clone(), binding).unwrap_err();
            assert_eq!(unsupported_feature_code(&error), Some("property_binding"));
        }
    }

    #[test]
    fn scripts_and_non_finite_colors_remain_blocked() {
        let error = resolve(
            json!({}),
            json!({"a":{"user":"missing","value":true},"z":{"script":"code"}}),
        )
        .unwrap_err();
        assert_eq!(unsupported_feature_code(&error), Some("scenescript"));
        for key in ["script", "scenescript", "SceneScript"] {
            let error =
                resolve(json!({}), json!({"unused":{(key):"code","value":true}})).unwrap_err();
            assert_eq!(unsupported_feature_code(&error), Some("scenescript"));
            let project = json!({"general":{"properties":{"flag":{"type":"bool","value":true}}}});
            let error = resolve(
                project,
                json!({"user":"flag","value":{(key):"code","value":true}}),
            )
            .unwrap_err();
            assert_eq!(unsupported_feature_code(&error), Some("scenescript"));
        }
        for value in [
            json!("NaN 0 0"),
            json!("0 0"),
            json!("0 2 0"),
            json!([0, 0, 1e100]),
            json!([0.1, 0.2, 0.3]),
            json!(format!("{}0 0 0", " ".repeat(256))),
        ] {
            let project = json!({"general":{"properties":{"tint":{"type":"color","value":value}}}});
            let error = resolve(project, json!({"user":"tint","value":"1 1 1"})).unwrap_err();
            assert_eq!(unsupported_feature_code(&error), Some("property_binding"));
        }
    }

    #[test]
    fn pure_script_bindings_follow_overrides_and_prescan_hidden_scripts() {
        let source = "let gain=0; export function applyUserProperties(p) { if (p.hasOwnProperty('gain')) { gain=p.gain; } } export function update(v) { return gain / 10; }";
        let mut project =
            json!({"general":{"properties":{"gain":{"type":"slider","value":4,"min":0,"max":10}}}});
        let document = json!({"objects":[{"visible":false,"alpha":{"script":source,"value":1}}]});
        assert_eq!(
            resolve(project.clone(), document.clone()).unwrap()["objects"][0]["alpha"],
            0.4
        );
        apply_property_overrides(&mut project, &BTreeMap::from([("gain".into(), json!(9))]))
            .unwrap();
        assert_eq!(
            resolve(project.clone(), document.clone()).unwrap()["objects"][0]["alpha"],
            0.9
        );
        let properties = ProjectProperties::from_project(&project).unwrap();
        let mut invalid = json!({"a":{"user":"missing","value":true},"z":{"visible":false,"alpha":{"script":"export function update(v){return engine.runtime;}","value":0}}});
        let before = invalid.clone();
        let error = properties.resolve_document(&mut invalid).unwrap_err();
        assert_eq!(unsupported_feature_code(&error), Some("scenescript"));
        assert_eq!(invalid, before);
        for wrapper in [
            json!({"script":source,"value":1,"extra":true}),
            json!({"script":source,"value":true}),
            json!({"scenescript":source,"value":1}),
        ] {
            assert_eq!(
                unsupported_feature_code(&resolve(project.clone(), wrapper).unwrap_err()),
                Some("scenescript")
            );
        }
    }

    #[test]
    fn scripts_share_a_bounded_project_budget_and_cached_values() {
        let source = "let gain=0; export function applyUserProperties(p){gain=p.gain;} export function update(v){return gain/2;}";
        let project = json!({"general":{"properties":{"gain":{"type":"slider","value":6}}}});
        let properties = ProjectProperties::from_project(&project).unwrap();
        let wrapper = json!({"script":source,"value":1});
        for _ in 0..MAX_SCRIPTS {
            let mut document = wrapper.clone();
            properties.resolve_document(&mut document).unwrap();
            assert_eq!(document, 3.0);
        }
        assert_eq!(properties.script_values.borrow().len(), 1);
        let mut document = wrapper;
        let error = properties.resolve_document(&mut document).unwrap_err();
        assert_eq!(unsupported_feature_code(&error), Some("scenescript"));
    }

    #[test]
    fn traversal_and_declaration_limits_fail_closed() {
        let mut document = json!(0);
        for _ in 0..MAX_DEPTH + 1 {
            document = json!([document]);
        }
        assert!(resolve(json!({}), document).is_err());
        let declarations: Map<String, Value> = (0..=MAX_PROPERTIES)
            .map(|index| {
                (
                    format!("property{index}"),
                    json!({"type":"bool","value":true}),
                )
            })
            .collect();
        assert!(
            ProjectProperties::from_project(&json!({"general":{"properties":declarations}}))
                .is_err()
        );
        let error = resolve(json!({}), Value::Array(vec![Value::Null; MAX_NODES])).unwrap_err();
        assert_eq!(unsupported_feature_code(&error), Some("property_binding"));
    }

    #[test]
    fn all_project_resources_share_one_budget_and_failures_consume_it() {
        let project = json!({"general":{"properties":{"flag":{"type":"bool","value":true}}}});
        let properties = ProjectProperties::from_project(&project).unwrap();
        assert!(properties.remaining_nodes.get() < MAX_NODES);
        let mut invalid = json!({"user":"missing","value":true});
        let before = properties.remaining_nodes.get();
        assert!(properties.resolve_document(&mut invalid).is_err());
        assert_eq!(properties.remaining_nodes.get(), before - 3);
        let mut resource = Value::Array(vec![Value::Null; MAX_NODES / 2]);
        properties.resolve_document(&mut resource).unwrap();
        let error = properties.resolve_document(&mut resource).unwrap_err();
        assert_eq!(unsupported_feature_code(&error), Some("property_binding"));
        assert_eq!(properties.remaining_nodes.get(), 0);
        assert!(properties.resolve_document(&mut Value::Null).is_err());
    }
}
