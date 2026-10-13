//! Bounded SceneScript layer bindings. Only preparation and a per-renderer
//! worker execute ECMAScript; frame submission and snapshot reads never wait.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, TryLockError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentTarget, BindingPatternKind, Declaration, Expression, ForStatementInit,
    SimpleAssignmentTarget, Statement, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rquickjs::context::intrinsic;
use rquickjs::{Context, Function, Module, Object, Persistent, Runtime};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::unsupported;

pub(super) const MAX_DYNAMIC_LAYERS: usize = 128;
const MAX_BINDINGS: usize = 32;
const MAX_SOURCE_BYTES: usize = 64 * 1024;
const MAX_TOTAL_SOURCE_BYTES: usize = 256 * 1024;
const MAX_AST_DEPTH: usize = 48;
const MEMORY_BYTES: usize = 8 * 1024 * 1024;
const STACK_BYTES: usize = 256 * 1024;
const INITIALIZATION_LIMIT: Duration = Duration::from_millis(100);
const FRAME_LIMIT: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct LayerState {
    pub handle: usize,
    pub origin: [f32; 3],
    pub scale: [f32; 3],
    pub angles: [f32; 3],
    pub color: [f32; 3],
    pub alpha: f32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub(super) struct Binding {
    pub layer: usize,
    pub property: String,
    pub source: String,
    pub input: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(super) struct CreatedLayer {
    pub path: String,
    pub state: LayerState,
}

#[derive(Debug, Clone)]
pub(super) struct PreparedScripts {
    bindings: Vec<Binding>,
    layers: Vec<LayerState>,
    properties: Value,
    created: Vec<CreatedLayer>,
    initial: Vec<LayerState>,
    audio: bool,
}

fn blocked(detail: impl Into<String>) -> anyhow::Error {
    unsupported("scenescript", detail)
}

impl PreparedScripts {
    pub(super) fn prepare(
        bindings: Vec<Binding>,
        layers: Vec<LayerState>,
        properties: Value,
    ) -> Result<Self> {
        if bindings.is_empty() || bindings.len() > MAX_BINDINGS {
            return Err(blocked("dynamic script binding count exceeds its budget"));
        }
        if layers.is_empty() || layers.len() > MAX_DYNAMIC_LAYERS {
            return Err(blocked("dynamic script layer count exceeds its budget"));
        }
        let mut handles = BTreeSet::new();
        for layer in &layers {
            validate_layer(layer)?;
            if !handles.insert(layer.handle) || layer.handle > 1_000_000 {
                return Err(blocked("invalid or duplicate dynamic layer handle"));
            }
        }
        let properties_map = properties
            .as_object()
            .ok_or_else(|| blocked("dynamic script properties require an object"))?;
        if properties_map.len() > 256 {
            return Err(blocked("dynamic script property count exceeds its budget"));
        }
        for (name, value) in properties_map {
            if !safe_name(name) {
                return Err(blocked("invalid dynamic script property name"));
            }
            match value {
                Value::Bool(_) => {}
                Value::Number(number) if number.as_f64().is_some_and(finite) => {}
                Value::String(text) if text.len() <= 4096 => {}
                Value::Array(values)
                    if values.len() == 3
                        && values
                            .iter()
                            .all(|value| value.as_f64().is_some_and(finite)) => {}
                _ => return Err(blocked("unsupported dynamic script property value")),
            }
        }
        let mut total_bytes = 0_usize;
        for binding in &bindings {
            if !handles.contains(&binding.layer)
                || !matches!(
                    binding.property.as_str(),
                    "visible" | "alpha" | "origin" | "scale" | "angles" | "color"
                )
            {
                return Err(blocked("unsupported dynamic script layer binding"));
            }
            validate_binding_input(binding)?;
            total_bytes = total_bytes
                .checked_add(binding.source.len())
                .ok_or_else(|| blocked("dynamic script source budget overflow"))?;
            if total_bytes > MAX_TOTAL_SOURCE_BYTES {
                return Err(blocked(
                    "dynamic script sources exceed their combined budget",
                ));
            }
            validate_source(&binding.source, properties_map.keys().cloned().collect())?;
        }
        let mut prepared = Self {
            bindings,
            layers,
            properties,
            created: Vec::new(),
            initial: Vec::new(),
            audio: false,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (mut vm, initialization) = Vm::new(&prepared, stop)?;
        prepared.created = initialization.created;
        prepared.audio = initialization.audio;
        prepared.initial = vm.frame(0.0, [0.0; 64])?;
        Ok(prepared)
    }

    pub(super) fn created_layers(&self) -> &[CreatedLayer] {
        &self.created
    }

    pub(super) fn initial_states(&self) -> &[LayerState] {
        &self.initial
    }

    pub(super) fn uses_audio(&self) -> bool {
        self.audio
    }
}

fn finite(value: f64) -> bool {
    value.is_finite() && (value as f32).is_finite()
}

fn validate_layer(layer: &LayerState) -> Result<()> {
    if !layer
        .origin
        .iter()
        .chain(&layer.scale)
        .chain(&layer.angles)
        .chain(&layer.color)
        .chain(std::iter::once(&layer.alpha))
        .all(|value| value.is_finite())
        || !(0.0..=1.0).contains(&layer.alpha)
        || layer.angles[0] != 0.0
        || layer.angles[1] != 0.0
    {
        return Err(blocked("invalid or unsupported dynamic layer transform"));
    }
    Ok(())
}

fn validate_binding_input(binding: &Binding) -> Result<()> {
    let valid = match binding.property.as_str() {
        "visible" => binding.input.is_boolean(),
        "alpha" => binding
            .input
            .as_f64()
            .is_some_and(|number| finite(number) && (0.0..=1.0).contains(&number)),
        _ => binding.input.as_array().is_some_and(|values| {
            values.len() == 3
                && values
                    .iter()
                    .all(|value| value.as_f64().is_some_and(finite))
        }),
    };
    if !valid {
        return Err(blocked("invalid dynamic script binding input"));
    }
    Ok(())
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && !matches!(name, "constructor" | "prototype" | "__proto__")
        && !name.starts_with("__lwe")
}

// This is a source-language gate, not a JavaScript translator. QuickJS executes
// the original module. New syntax or host capabilities must be admitted here
// deliberately, rather than surviving preparation as an ignored property.
fn validate_source(source: &str, properties: BTreeSet<String>) -> Result<()> {
    if source.is_empty()
        || source.len() > MAX_SOURCE_BYTES
        || source
            .bytes()
            .filter(|byte| b"{}()[]".contains(byte))
            .count()
            > 4096
    {
        return Err(blocked("dynamic script source exceeds its parsing budget"));
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return Err(blocked("invalid dynamic ECMAScript module"));
    }
    let mut validation = Validation {
        names: BTreeSet::new(),
        numeric_names: BTreeSet::new(),
        non_numeric_names: BTreeSet::new(),
        properties,
    };
    let mut callbacks = BTreeSet::new();
    for statement in &parsed.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                validation.declarations(declaration)?;
            }
            Statement::ExportNamedDeclaration(export)
                if export.source.is_none() && export.specifiers.is_empty() =>
            {
                let Some(Declaration::FunctionDeclaration(function)) = &export.declaration else {
                    return Err(blocked("only SceneScript callbacks may be exported"));
                };
                let name = function
                    .id
                    .as_ref()
                    .map(|name| name.name.as_str())
                    .ok_or_else(|| blocked("dynamic callback requires a name"))?;
                if !matches!(name, "init" | "update" | "applyUserProperties")
                    || !callbacks.insert(name)
                    || function.r#async
                    || function.generator
                    || function.params.rest.is_some()
                    || function.body.is_none()
                    || function.params.items.len() > usize::from(name != "init")
                {
                    return Err(blocked("unsupported dynamic script callback"));
                }
                for parameter in &function.params.items {
                    validation.binding(&parameter.pattern.kind)?;
                    if let BindingPatternKind::BindingIdentifier(identifier) =
                        &parameter.pattern.kind
                    {
                        validation
                            .non_numeric_names
                            .insert(identifier.name.to_string());
                        validation.numeric_names.remove(identifier.name.as_str());
                    }
                }
                for statement in &function
                    .body
                    .as_ref()
                    .expect("checked callback body")
                    .statements
                {
                    validation.collect(statement, 0)?;
                }
            }
            _ => return Err(blocked("unsupported dynamic module statement or import")),
        }
    }
    if !callbacks.contains("update") {
        return Err(blocked("dynamic script requires an update callback"));
    }
    for statement in &parsed.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                validation.variable_expressions(declaration, 0)?;
            }
            Statement::ExportNamedDeclaration(export) => {
                let Some(Declaration::FunctionDeclaration(function)) = &export.declaration else {
                    unreachable!("declarations validated above")
                };
                for statement in &function.body.as_ref().expect("checked body").statements {
                    validation.statement(statement, 0)?;
                }
            }
            _ => unreachable!("module statements validated above"),
        }
    }
    Ok(())
}

struct Validation {
    names: BTreeSet<String>,
    numeric_names: BTreeSet<String>,
    non_numeric_names: BTreeSet<String>,
    properties: BTreeSet<String>,
}

impl Validation {
    fn depth(&self, depth: usize) -> Result<()> {
        if depth > MAX_AST_DEPTH {
            return Err(blocked("dynamic script AST exceeds its nesting budget"));
        }
        Ok(())
    }

    fn binding(&mut self, pattern: &BindingPatternKind<'_>) -> Result<()> {
        let BindingPatternKind::BindingIdentifier(identifier) = pattern else {
            return Err(blocked("dynamic destructuring bindings are not supported"));
        };
        let name = identifier.name.as_str();
        if !safe_name(name)
            || matches!(
                name,
                "engine" | "thisScene" | "thisLayer" | "Vec3" | "Math" | "undefined"
            )
        {
            return Err(blocked("dynamic binding shadows a host capability"));
        }
        self.names.insert(name.to_owned());
        if self.names.len() > 256 {
            return Err(blocked("dynamic binding count exceeds its budget"));
        }
        Ok(())
    }

    fn declarations(&mut self, declaration: &VariableDeclaration<'_>) -> Result<()> {
        for declarator in &declaration.declarations {
            self.binding(&declarator.id.kind)?;
            if let BindingPatternKind::BindingIdentifier(identifier) = &declarator.id.kind {
                let name = identifier.name.as_str();
                if matches!(&declarator.init, Some(Expression::NumericLiteral(_)))
                    && !self.non_numeric_names.contains(name)
                {
                    self.numeric_names.insert(name.to_owned());
                } else {
                    self.numeric_names.remove(name);
                    self.non_numeric_names.insert(name.to_owned());
                }
            }
        }
        Ok(())
    }

    fn collect(&mut self, statement: &Statement<'_>, depth: usize) -> Result<()> {
        self.depth(depth)?;
        match statement {
            Statement::VariableDeclaration(declaration) => self.declarations(declaration),
            Statement::BlockStatement(block) => {
                for statement in &block.body {
                    self.collect(statement, depth + 1)?;
                }
                Ok(())
            }
            Statement::ForStatement(statement) => {
                if let Some(ForStatementInit::VariableDeclaration(declaration)) = &statement.init {
                    self.declarations(declaration)?;
                }
                self.collect(&statement.body, depth + 1)
            }
            Statement::WhileStatement(statement) => self.collect(&statement.body, depth + 1),
            Statement::IfStatement(statement) => {
                self.collect(&statement.consequent, depth + 1)?;
                if let Some(alternate) = &statement.alternate {
                    self.collect(alternate, depth + 1)?;
                }
                Ok(())
            }
            Statement::ExpressionStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_) => Ok(()),
            _ => Err(blocked("unsupported dynamic script statement")),
        }
    }

    fn variable_expressions(
        &self,
        declaration: &VariableDeclaration<'_>,
        depth: usize,
    ) -> Result<()> {
        for declarator in &declaration.declarations {
            if let Some(expression) = &declarator.init {
                self.expression(expression, depth + 1)?;
            }
        }
        Ok(())
    }

    fn statement(&self, statement: &Statement<'_>, depth: usize) -> Result<()> {
        self.depth(depth)?;
        match statement {
            Statement::VariableDeclaration(declaration) => {
                self.variable_expressions(declaration, depth)
            }
            Statement::BlockStatement(block) => {
                for statement in &block.body {
                    self.statement(statement, depth + 1)?;
                }
                Ok(())
            }
            Statement::ExpressionStatement(statement) => {
                self.expression(&statement.expression, depth + 1)
            }
            Statement::ReturnStatement(statement) => {
                if let Some(expression) = &statement.argument {
                    self.expression(expression, depth + 1)?;
                }
                Ok(())
            }
            Statement::ForStatement(statement) => {
                if let Some(init) = &statement.init {
                    match init {
                        ForStatementInit::VariableDeclaration(declaration) => {
                            self.variable_expressions(declaration, depth + 1)?;
                        }
                        _ => self.expression(
                            init.as_expression().expect("expression for initializer"),
                            depth + 1,
                        )?,
                    }
                }
                if let Some(test) = &statement.test {
                    self.expression(test, depth + 1)?;
                }
                if let Some(update) = &statement.update {
                    self.expression(update, depth + 1)?;
                }
                self.statement(&statement.body, depth + 1)
            }
            Statement::WhileStatement(statement) => {
                self.expression(&statement.test, depth + 1)?;
                self.statement(&statement.body, depth + 1)
            }
            Statement::IfStatement(statement) => {
                self.expression(&statement.test, depth + 1)?;
                self.statement(&statement.consequent, depth + 1)?;
                if let Some(alternate) = &statement.alternate {
                    self.statement(alternate, depth + 1)?;
                }
                Ok(())
            }
            Statement::EmptyStatement(_) => Ok(()),
            Statement::BreakStatement(statement) if statement.label.is_none() => Ok(()),
            Statement::ContinueStatement(statement) if statement.label.is_none() => Ok(()),
            _ => Err(blocked("unsupported dynamic script statement")),
        }
    }

    fn identifier(&self, name: &str) -> Result<()> {
        if self.names.contains(name) || matches!(name, "undefined" | "thisLayer") {
            Ok(())
        } else {
            Err(blocked(format!("unsupported dynamic script global {name}")))
        }
    }

    fn member(&self, object: &Expression<'_>, property: &str, depth: usize) -> Result<()> {
        if !safe_name(property) {
            return Err(blocked("dynamic prototype access is not supported"));
        }
        if let Expression::Identifier(identifier) = object {
            let allowed = match identifier.name.as_str() {
                "engine" => Some(property == "registerAudioBuffers"),
                "thisScene" => Some(property == "createLayer"),
                "thisLayer" => Some(matches!(
                    property,
                    "origin" | "scale" | "angles" | "color" | "alpha" | "visible"
                )),
                "Math" => Some(matches!(
                    property,
                    "abs"
                        | "sign"
                        | "sqrt"
                        | "floor"
                        | "ceil"
                        | "round"
                        | "trunc"
                        | "sin"
                        | "cos"
                        | "tan"
                        | "exp"
                        | "log"
                        | "pow"
                        | "min"
                        | "max"
                        | "PI"
                        | "E"
                )),
                _ => None,
            };
            if let Some(allowed) = allowed {
                return if allowed {
                    Ok(())
                } else {
                    Err(blocked(format!(
                        "unsupported SceneScript host member {property}"
                    )))
                };
            }
        }
        if !matches!(
            property,
            "origin"
                | "scale"
                | "angles"
                | "color"
                | "alpha"
                | "visible"
                | "copy"
                | "x"
                | "y"
                | "z"
                | "push"
                | "length"
                | "average"
                | "hasOwnProperty"
        ) && !self.properties.contains(property)
        {
            return Err(blocked(format!(
                "unsupported SceneScript member {property}"
            )));
        }
        self.expression(object, depth + 1)
    }

    fn computed(
        &self,
        object: &Expression<'_>,
        index: &Expression<'_>,
        depth: usize,
    ) -> Result<()> {
        // Numeric indexing admits arrays and audio buffers, but cannot become
        // a computed route to a host function or JavaScript prototype.
        let numeric = match index {
            Expression::NumericLiteral(number) => {
                number.value.fract() == 0.0
                    && (0.0..MAX_DYNAMIC_LAYERS as f64).contains(&number.value)
            }
            Expression::Identifier(identifier) => {
                self.numeric_names.contains(identifier.name.as_str())
            }
            _ => false,
        };
        if !numeric {
            return Err(blocked("dynamic member names are not supported"));
        }
        self.expression(object, depth + 1)?;
        self.expression(index, depth + 1)
    }

    fn target(&self, target: &AssignmentTarget<'_>, depth: usize) -> Result<()> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                if self.names.contains(identifier.name.as_str()) {
                    Ok(())
                } else {
                    Err(blocked(
                        "dynamic assignments cannot replace a host capability",
                    ))
                }
            }
            AssignmentTarget::StaticMemberExpression(member) if !member.optional => {
                self.member(&member.object, member.property.name.as_str(), depth)
            }
            AssignmentTarget::ComputedMemberExpression(_) => Err(blocked(
                "dynamic indexed writes and audio buffer mutations are unsupported",
            )),
            _ => Err(blocked("unsupported dynamic assignment target")),
        }
    }

    fn expression(&self, expression: &Expression<'_>, depth: usize) -> Result<()> {
        self.depth(depth)?;
        match expression {
            Expression::NumericLiteral(number) if finite(number.value) => Ok(()),
            Expression::BooleanLiteral(_) | Expression::NullLiteral(_) => Ok(()),
            Expression::StringLiteral(text) if text.value.len() <= 4096 => Ok(()),
            Expression::Identifier(identifier) => self.identifier(identifier.name.as_str()),
            Expression::ArrayExpression(array) => {
                if array.elements.len() > MAX_DYNAMIC_LAYERS {
                    return Err(blocked("dynamic array literal exceeds its budget"));
                }
                for element in &array.elements {
                    let expression = element
                        .as_expression()
                        .ok_or_else(|| blocked("array spread and holes are not supported"))?;
                    self.expression(expression, depth + 1)?;
                }
                Ok(())
            }
            Expression::ParenthesizedExpression(expression) => {
                self.expression(&expression.expression, depth + 1)
            }
            Expression::UnaryExpression(unary)
                if matches!(unary.operator.as_str(), "+" | "-" | "!") =>
            {
                self.expression(&unary.argument, depth + 1)
            }
            Expression::BinaryExpression(binary)
                if matches!(
                    binary.operator.as_str(),
                    "+" | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                        | "=="
                        | "!="
                        | "==="
                        | "!=="
                        | "<"
                        | ">"
                        | "<="
                        | ">="
                ) =>
            {
                self.expression(&binary.left, depth + 1)?;
                self.expression(&binary.right, depth + 1)
            }
            Expression::LogicalExpression(logical) => {
                self.expression(&logical.left, depth + 1)?;
                self.expression(&logical.right, depth + 1)
            }
            Expression::ConditionalExpression(conditional) => {
                self.expression(&conditional.test, depth + 1)?;
                self.expression(&conditional.consequent, depth + 1)?;
                self.expression(&conditional.alternate, depth + 1)
            }
            Expression::StaticMemberExpression(member) if !member.optional => {
                self.member(&member.object, member.property.name.as_str(), depth)
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                self.computed(&member.object, &member.expression, depth)
            }
            Expression::AssignmentExpression(assignment) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &assignment.left
                    && self.numeric_names.contains(identifier.name.as_str())
                    && !self.numeric_expression(&assignment.right)
                {
                    return Err(blocked(
                        "numeric array indices cannot acquire dynamic member names",
                    ));
                }
                self.target(&assignment.left, depth + 1)?;
                self.expression(&assignment.right, depth + 1)
            }
            Expression::UpdateExpression(update) => match &update.argument {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                    self.identifier(identifier.name.as_str())
                }
                _ => Err(blocked(
                    "only local scalar increment and decrement are supported",
                )),
            },
            Expression::CallExpression(call) if !call.optional => {
                let Expression::StaticMemberExpression(member) = &call.callee else {
                    return Err(blocked(
                        "only explicit bounded host and value methods may be called",
                    ));
                };
                let method = member.property.name.as_str();
                if !matches!(
                    method,
                    "registerAudioBuffers"
                        | "createLayer"
                        | "copy"
                        | "push"
                        | "hasOwnProperty"
                        | "abs"
                        | "sign"
                        | "sqrt"
                        | "floor"
                        | "ceil"
                        | "round"
                        | "trunc"
                        | "sin"
                        | "cos"
                        | "tan"
                        | "exp"
                        | "log"
                        | "pow"
                        | "min"
                        | "max"
                ) || call.arguments.len() > 8
                {
                    return Err(blocked("unsupported dynamic method call"));
                }
                self.member(&member.object, method, depth + 1)?;
                for argument in &call.arguments {
                    self.expression(
                        argument
                            .as_expression()
                            .ok_or_else(|| blocked("argument spread is unsupported"))?,
                        depth + 1,
                    )?;
                }
                Ok(())
            }
            Expression::NewExpression(constructor)
                if matches!(&constructor.callee, Expression::Identifier(identifier) if identifier.name == "Vec3")
                    && matches!(constructor.arguments.len(), 1 | 3) =>
            {
                for argument in &constructor.arguments {
                    self.expression(
                        argument
                            .as_expression()
                            .ok_or_else(|| blocked("constructor spread is unsupported"))?,
                        depth + 1,
                    )?;
                }
                Ok(())
            }
            _ => Err(blocked(
                "expression is outside the bounded dynamic SceneScript subset",
            )),
        }
    }

    fn numeric_expression(&self, expression: &Expression<'_>) -> bool {
        match expression {
            Expression::NumericLiteral(_) => true,
            Expression::Identifier(identifier) => {
                self.numeric_names.contains(identifier.name.as_str())
            }
            Expression::UnaryExpression(expression) => {
                matches!(expression.operator.as_str(), "+" | "-")
                    && self.numeric_expression(&expression.argument)
            }
            Expression::BinaryExpression(expression) => {
                matches!(
                    expression.operator.as_str(),
                    "+" | "-" | "*" | "/" | "%" | "**"
                ) && self.numeric_expression(&expression.left)
                    && self.numeric_expression(&expression.right)
            }
            Expression::ParenthesizedExpression(expression) => {
                self.numeric_expression(&expression.expression)
            }
            _ => false,
        }
    }
}

// The closure keeps mutable host state out of the author's globals. Its
// accessors copy vectors both ways, matching native value semantics when an
// author reuses one Vec3 while assigning many layers inside a loop.
const HOST: &str = r#"
(() => {
    'use strict';
    const functionPrototype = Function.prototype;
    Object.defineProperty(functionPrototype, 'constructor', {value: undefined, writable: false, configurable: false});
    Object.defineProperty(globalThis, 'eval', {value: undefined, writable: false, configurable: false});
    Object.defineProperty(globalThis, 'Function', {value: undefined, writable: false, configurable: false});
    for (const prototype of [Object.prototype, Array.prototype, Map.prototype, Number.prototype, String.prototype, functionPrototype,
        Float32Array.prototype, Object.getPrototypeOf(Float32Array.prototype)]) Object.freeze(prototype);
    Object.freeze(Math);
    const layers = new Map();
    const created = [];
    let nextHandle = 0;
    let initializing = false;
    let usesAudio = false;
    let registration = false;
    const average = new Float32Array(64);
    Object.preventExtensions(average);
    const audio = Object.freeze({average});
    const finite = v => typeof v === 'number' && Number.isFinite(v) && Math.abs(v) <= 3.4028234663852886e38;
    class Vec3 {
        constructor(x, y, z) {
            if (arguments.length === 1) { y = x; z = x; }
            if (!finite(x) || !finite(y) || !finite(z)) throw new TypeError('invalid Vec3');
            this.x = x; this.y = y; this.z = z;
            Object.preventExtensions(this);
        }
        copy() { return new Vec3(this.x, this.y, this.z); }
    }
    Object.freeze(Vec3.prototype);
    Object.freeze(Vec3);
    const vector = v => {
        if (!v || !finite(v.x) || !finite(v.y) || !finite(v.z)) throw new TypeError('invalid layer vector');
        return [v.x, v.y, v.z];
    };
    const copy = v => new Vec3(v[0], v[1], v[2]);
    const makeLayer = state => {
        const layer = {};
        for (const key of ['origin', 'scale', 'angles', 'color']) {
            Object.defineProperty(layer, key, {
                enumerable: true,
                get: () => copy(state[key]),
                set: value => {
                    const parsed = vector(value);
                    if (key === 'angles' && (parsed[0] !== 0 || parsed[1] !== 0)) throw new TypeError('3D layer rotation unsupported');
                    state[key] = parsed;
                }
            });
        }
        Object.defineProperty(layer, 'visible', {
            enumerable: true, get: () => state.visible,
            set: value => { if (typeof value !== 'boolean') throw new TypeError('invalid layer visibility'); state.visible = value; }
        });
        Object.defineProperty(layer, 'alpha', {
            enumerable: true, get: () => state.alpha,
            set: value => { if (!finite(value) || value < 0 || value > 1) throw new TypeError('invalid layer alpha'); state.alpha = value; }
        });
        Object.preventExtensions(layer);
        layers.set(state.handle, {state, layer});
        nextHandle = Math.max(nextHandle, state.handle + 1);
        return layer;
    };
    const engine = Object.freeze({registerAudioBuffers: count => {
        if (!registration) throw new TypeError('audio registration is only supported at module global scope');
        if (count !== 64) throw new TypeError('only 64 audio bands are supported');
        usesAudio = true;
        return Object.freeze(audio);
    }});
    const thisScene = Object.freeze({createLayer: path => {
        if (!initializing) throw new TypeError('layer creation is only supported during init');
        if (typeof path !== 'string' || path.length === 0 || path.length > 1024 || !path.startsWith('models/') || !path.endsWith('.json')
            || ![...path].every(char => 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_./ -'.includes(char))
            || path.split('/').some(part => part === '' || part === '.' || part === '..')) throw new TypeError('unsafe image model resource path');
        if (layers.size >= 128) throw new RangeError('dynamic layer budget exceeded');
        const state = {handle: nextHandle, origin: [0,0,0], scale: [1,1,1], angles: [0,0,0], color: [1,1,1], alpha: 1, visible: true};
        const layer = makeLayer(state);
        created.push({path, state});
        return layer;
    }});
    Object.defineProperty(globalThis, 'Vec3', {value: Vec3});
    Object.defineProperty(globalThis, 'engine', {value: engine});
    Object.defineProperty(globalThis, 'thisScene', {value: thisScene});
    return Object.freeze({
        load: values => { for (const state of values) makeLayer(state); },
        owner: handle => {
            const item = layers.get(handle);
            if (!item) throw new TypeError('unknown script owner layer');
            globalThis.thisLayer = item.layer;
        },
        initialize: value => { initializing = value; },
        register: value => { registration = value; },
        frame: values => {
            registration = false;
            for (let i=0; i<64; ++i) {
                average[i] = values[i];
            }
        },
        input: (handle, property) => layers.get(handle).layer[property],
        properties: values => {
            for (const key of Object.keys(values)) {
                if (Array.isArray(values[key])) values[key] = copy(values[key]);
            }
            return values;
        },
        output: (handle, property, value) => {
            if (value !== undefined) layers.get(handle).layer[property] = value;
        },
        snapshot: () => JSON.stringify({states: [...layers.values()].map(item => item.state), created, audio: usesAudio})
    });
})()
"#;

#[derive(Deserialize)]
struct Snapshot {
    states: Vec<LayerState>,
    created: Vec<CreatedLayer>,
    audio: bool,
}

struct Vm {
    // Persistent JS values must be destroyed before their context/runtime.
    modules: Vec<Persistent<Object<'static>>>,
    host: Persistent<Object<'static>>,
    context: Context,
    _runtime: Runtime,
    deadline: Rc<Cell<Instant>>,
    bindings: Vec<Binding>,
    base_handles: BTreeSet<usize>,
}

impl Vm {
    fn new(plan: &PreparedScripts, stop: Arc<AtomicBool>) -> Result<(Self, Snapshot)> {
        let base_handles = plan
            .layers
            .iter()
            .map(|layer| layer.handle)
            .collect::<BTreeSet<_>>();
        let runtime = Runtime::new().map_err(|_| blocked("cannot initialize dynamic script VM"))?;
        runtime.set_memory_limit(MEMORY_BYTES);
        runtime.set_max_stack_size(STACK_BYTES);
        let deadline = Rc::new(Cell::new(Instant::now() + INITIALIZATION_LIMIT));
        let callback_deadline = Rc::clone(&deadline);
        runtime.set_interrupt_handler(Some(Box::new(move || {
            stop.load(Ordering::Acquire) || Instant::now() >= callback_deadline.get()
        })));
        // QuickJS needs Eval internally to compile the host and modules. HOST
        // removes author-accessible eval/Function/constructor routes before any
        // author code runs. Date, Proxy, external I/O and loaders are absent.
        // Promise supports module evaluation; author async/await is rejected.
        let context = Context::custom::<(
            intrinsic::Eval,
            intrinsic::Promise,
            intrinsic::Json,
            intrinsic::MapSet,
            intrinsic::TypedArrays,
        )>(&runtime)
        .map_err(|_| blocked("cannot initialize dynamic script context"))?;
        let (host, modules, initialization) = context.with(|ctx| -> Result<_> {
            let host: Object = ctx.eval(HOST).map_err(|_| {
                let exception = ctx.catch();
                let message = exception
                    .as_object()
                    .and_then(|object| object.get::<_, String>("message").ok())
                    .unwrap_or_default();
                blocked(format!(
                    "cannot initialize dynamic host capabilities: {message}"
                ))
            })?;
            let load: Function = host.get("load")?;
            load.call::<_, ()>((ctx.json_parse(serde_json::to_vec(&plan.layers)?)?,))?;
            let owner: Function = host.get("owner")?;
            let initialize: Function = host.get("initialize")?;
            let registration: Function = host.get("register")?;
            let mut modules = Vec::new();
            for (index, binding) in plan.bindings.iter().enumerate() {
                owner.call::<_, ()>((binding.layer,))?;
                let declared = Module::declare(
                    ctx.clone(),
                    format!("lwe-layer-binding-{index}"),
                    binding.source.as_str(),
                )
                .map_err(|_| blocked("cannot compile dynamic script module"))?;
                registration.call::<_, ()>((true,))?;
                let (module, completion) = declared.eval().map_err(|_| {
                    blocked("dynamic script initialization failed or exceeded its limits")
                })?;
                completion.finish::<()>().map_err(|_| {
                    blocked("dynamic script module initialization is not synchronous")
                })?;
                registration.call::<_, ()>((false,))?;
                let init: rquickjs::Value = module.get("init")?;
                if let Some(init) = init.as_function() {
                    initialize.call::<_, ()>((true,))?;
                    let initial: rquickjs::Value = init
                        .call(())
                        .map_err(|_| blocked("dynamic init failed or exceeded its limits"))?;
                    let output: Function = host.get("output")?;
                    output
                        .call::<_, ()>((binding.layer, binding.property.as_str(), initial))
                        .map_err(|_| blocked("dynamic init returned an invalid binding value"))?;
                    initialize.call::<_, ()>((false,))?;
                }
                let apply: rquickjs::Value = module.get("applyUserProperties")?;
                if let Some(apply) = apply.as_function() {
                    let property_values: Function = host.get("properties")?;
                    let snapshot = property_values.call::<_, rquickjs::Value>((
                        ctx.json_parse(serde_json::to_vec(&plan.properties)?)?,
                    ))?;
                    apply.call::<_, ()>((snapshot,)).map_err(|_| {
                        blocked("dynamic property callback failed or exceeded its limits")
                    })?;
                }
                modules.push(Persistent::save(&ctx, module.namespace()?));
            }
            let snapshot = read_snapshot(&host, &base_handles)?;
            Ok((Persistent::save(&ctx, host), modules, snapshot))
        })?;
        Ok((
            Self {
                modules,
                host,
                context,
                _runtime: runtime,
                deadline,
                bindings: plan.bindings.clone(),
                base_handles,
            },
            initialization,
        ))
    }

    fn frame(&mut self, _time: f64, audio: [f32; 64]) -> Result<Vec<LayerState>> {
        self.deadline.set(Instant::now() + FRAME_LIMIT);
        self.context.with(|ctx| -> Result<_> {
            let host = self.host.clone().restore(&ctx)?;
            let frame: Function = host.get("frame")?;
            frame.call::<_, ()>((audio.to_vec(),))?;
            let owner: Function = host.get("owner")?;
            let input: Function = host.get("input")?;
            let output: Function = host.get("output")?;
            for (module, binding) in self.modules.iter().zip(&self.bindings) {
                owner.call::<_, ()>((binding.layer,))?;
                let module = module.clone().restore(&ctx)?;
                let update: Function = module.get("update")?;
                let current =
                    input.call::<_, rquickjs::Value>((binding.layer, binding.property.as_str()))?;
                let value: rquickjs::Value = update
                    .call((current,))
                    .map_err(|_| blocked("dynamic update failed or exceeded its limits"))?;
                output
                    .call::<_, ()>((binding.layer, binding.property.as_str(), value))
                    .map_err(|_| blocked("dynamic update returned an invalid binding value"))?;
            }
            let snapshot = read_snapshot(&host, &self.base_handles)?;
            Ok(snapshot.states)
        })
    }
}

fn read_snapshot(host: &Object<'_>, base_handles: &BTreeSet<usize>) -> Result<Snapshot> {
    let snapshot: Function = host.get("snapshot")?;
    let json: String = snapshot
        .call(())
        .map_err(|_| blocked("cannot read dynamic script snapshot"))?;
    if json.len() > 128 * 1024 {
        return Err(blocked("dynamic snapshot exceeds its byte budget"));
    }
    let snapshot: Snapshot =
        serde_json::from_str(&json).map_err(|_| blocked("invalid dynamic script snapshot"))?;
    if snapshot.states.len() > MAX_DYNAMIC_LAYERS {
        return Err(blocked("dynamic snapshot exceeds its layer budget"));
    }
    let mut state_handles = BTreeSet::new();
    for state in &snapshot.states {
        validate_layer(state)?;
        if !state_handles.insert(state.handle) {
            return Err(blocked("dynamic snapshot repeats a layer handle"));
        }
    }
    let mut created_handles = BTreeSet::new();
    for created in &snapshot.created {
        if !created.path.starts_with("models/")
            || !created.path.ends_with(".json")
            || created.path.len() > 1024
            || created
                .path
                .split('/')
                .any(|part| matches!(part, "" | "." | ".."))
            || !created
                .path
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./ -".contains(&byte))
            || base_handles.contains(&created.state.handle)
            || !created_handles.insert(created.state.handle)
            || !snapshot.states.iter().any(|state| state == &created.state)
        {
            return Err(blocked(
                "dynamic snapshot has an invalid created-layer resource record",
            ));
        }
    }
    if !base_handles.is_subset(&state_handles)
        || state_handles.len() != base_handles.len() + created_handles.len()
        || !created_handles.is_subset(&state_handles)
    {
        return Err(blocked(
            "dynamic snapshot layers do not match their prepared resource records",
        ));
    }
    Ok(snapshot)
}

struct FrameInput {
    time: f64,
    audio: [f32; 64],
}

#[derive(Default)]
struct Mailbox {
    initialized: bool,
    input: Option<FrameInput>,
    output: Option<Vec<LayerState>>,
    error: Option<String>,
    last_submitted: Option<f64>,
}

pub(super) struct ScriptWorker {
    stop: Arc<AtomicBool>,
    mailbox: Arc<Mutex<Mailbox>>,
    thread: Option<JoinHandle<()>>,
}

impl ScriptWorker {
    pub(super) fn spawn(plan: &PreparedScripts) -> Result<Self> {
        let plan = plan.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let mailbox = Arc::new(Mutex::new(Mailbox::default()));
        let worker_stop = Arc::clone(&stop);
        let worker_mailbox = Arc::clone(&mailbox);
        let thread = thread::Builder::new()
            .name("lwe-scene-script".to_owned())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    worker_run(plan, &worker_mailbox, Arc::clone(&worker_stop))
                }))
                .unwrap_or_else(|_| Err(blocked("dynamic script worker stopped unexpectedly")));
                if !worker_stop.load(Ordering::Acquire)
                    && let Err(error) = result
                {
                    worker_mailbox
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .error = Some(format!("{error:#}"));
                }
            })
            .map_err(|_| blocked("cannot start dynamic SceneScript worker"))?;
        Ok(Self {
            stop,
            mailbox,
            thread: Some(thread),
        })
    }

    /// Overwrite the pending input and consume at most one complete snapshot.
    /// Contention returns None, never a wait on author code or a worker lock.
    pub(super) fn poll_frame(
        &self,
        time: f64,
        audio: [f32; 64],
    ) -> Result<Option<Vec<LayerState>>> {
        ensure!(
            time.is_finite()
                && time >= 0.0
                && audio
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            "Invalid dynamic SceneScript frame input"
        );
        let mut mailbox = match self.mailbox.try_lock() {
            Ok(mailbox) => mailbox,
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Poisoned(_)) => {
                return Err(blocked("dynamic script mailbox is unavailable"));
            }
        };
        if let Some(error) = &mailbox.error {
            return Err(blocked(error.clone()));
        }
        if !mailbox.initialized {
            return Ok(None);
        }
        if mailbox.last_submitted != Some(time) {
            mailbox.last_submitted = Some(time);
            mailbox.input = Some(FrameInput { time, audio });
            if let Some(thread) = &self.thread {
                thread.thread().unpark();
            }
        }
        Ok(mailbox.output.take())
    }
}

fn worker_run(
    plan: PreparedScripts,
    mailbox: &Mutex<Mailbox>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let (mut vm, initialization) = Vm::new(&plan, Arc::clone(&stop))?;
    if initialization.created != plan.created || initialization.audio != plan.audio {
        return Err(blocked(
            "dynamic initialization differs from the prepared resource plan",
        ));
    }
    let initial = vm.frame(0.0, [0.0; 64])?;
    if initial != plan.initial {
        return Err(blocked(
            "dynamic initial frame differs from its prepared snapshot",
        ));
    }
    let mut ready = mailbox
        .lock()
        .map_err(|_| blocked("dynamic script mailbox is unavailable"))?;
    ready.output = Some(initial);
    ready.initialized = true;
    ready.last_submitted = Some(0.0);
    drop(ready);
    while !stop.load(Ordering::Acquire) {
        let input = mailbox
            .lock()
            .map_err(|_| blocked("dynamic script mailbox is unavailable"))?
            .input
            .take();
        if let Some(input) = input {
            let output = vm.frame(input.time, input.audio)?;
            mailbox
                .lock()
                .map_err(|_| blocked("dynamic script mailbox is unavailable"))?
                .output = Some(output);
        } else {
            thread::park();
        }
    }
    Ok(())
}

impl Drop for ScriptWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            if thread.join().is_err() {
                tracing::warn!("SceneScript worker stopped unexpectedly");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer() -> LayerState {
        LayerState {
            handle: 7,
            origin: [100.0, 90.0, 0.0],
            scale: [0.0; 3],
            angles: [0.0; 3],
            color: [0.0; 3],
            alpha: 1.0,
            visible: true,
        }
    }

    fn prepare(source: &str) -> Result<PreparedScripts> {
        PreparedScripts::prepare(
            vec![Binding {
                layer: 7,
                property: "visible".to_owned(),
                source: source.to_owned(),
                input: Value::Bool(true),
            }],
            vec![layer()],
            serde_json::json!({"mode":"meter"}),
        )
    }

    // Independent author fixture: do not copy a user-owned package's scripts
    // into the repository. The reused vectors exercise native value copies.
    const AUDIO_BARS: &str = r#"
        const samples = engine.registerAudioBuffers(64);
        let meters = [];
        let anchor;
        export function init() {
            for (let slot = 0; slot < 64; slot++) {
                meters.push(slot === 0 ? thisLayer : thisScene.createLayer('models/meter.json'));
            }
            anchor = thisLayer.origin;
        }
        export function applyUserProperties(settings) {
            for (let item = 0; item < 64; item++) {
                meters[item].visible = settings.mode === 'meter';
            }
        }
        export function update() {
            let position = anchor.copy();
            let stretch = new Vec3(5);
            for (let band = 0; band < 64; band++) {
                position.x += 30;
                stretch.y = samples.average[band] * 50;
                meters[band].origin = position;
                meters[band].scale = stretch;
            }
        }
    "#;

    #[test]
    fn audio_layers_preserve_value_copy_and_zero_scale() {
        let plan = prepare(AUDIO_BARS).unwrap();
        assert!(plan.uses_audio());
        assert_eq!(plan.created_layers().len(), 63);
        assert_eq!(plan.created_layers()[0].state.handle, 8);
        assert_eq!(plan.initial_states()[0].scale, [5.0, 0.0, 5.0]);
        assert_eq!(plan.initial_states()[0].origin, [130.0, 90.0, 0.0]);
        assert_eq!(plan.initial_states()[63].origin, [2020.0, 90.0, 0.0]);
        let (mut vm, _) = Vm::new(&plan, Arc::new(AtomicBool::new(false))).unwrap();
        let output = vm
            .frame(1.0, std::array::from_fn(|i| i as f32 / 64.0))
            .unwrap();
        assert_eq!(output[0].scale, [5.0, 0.0, 5.0]);
        assert_eq!(output[63].scale, [5.0, 49.21875, 5.0]);
        assert_eq!(output[0].origin[0], 130.0);
        assert_eq!(output[63].origin[0], 2020.0);
        assert!(output.iter().all(|state| state.visible));
    }

    #[test]
    fn unsafe_and_unimplemented_features_fail_closed() {
        for source in [
            "export function update(){ engine.launch('x'); }",
            "export function update(){ thisLayer.foo = 1; }",
            "export function update(){ return eval('1'); }",
            "export async function update(){}",
            "import x from 'x'; export function update(){}",
            "export function update(){ thisLayer.origin.constructor('return 1')(); }",
            "export function update(){ thisScene.createLayer('../x'); }",
            "let a=engine.registerAudioBuffers(16); export function update(){}",
            "let a=engine.registerAudioBuffers(64); export function update(){return a.left[0];}",
            "let a=engine.registerAudioBuffers(64); export function update(){a.average[0]=1;}",
            "let a=engine.registerAudioBuffers(64); let k='left'; export function update(){return a[k];}",
            "let a=engine.registerAudioBuffers(64); let k=0; export function update(){k='left';return a[k];}",
            "let i=0; let list=[]; i='__proto__'; let proto=list[i]; proto.push=Math.sin; export function init(){thisScene.createLayer('models/bar.json');} export function update(){}",
            "let key=0; let list=[]; export function init(){let key='__proto__'; let proto=list[key]; proto.push=Math.sin; thisScene.createLayer('models/bar.json');} export function update(){}",
            "let key=0; let list=[]; export function applyUserProperties(key){let proto=list[key];proto.push=Math.sin;} export function update(){}",
            "export function update(){Math.max=Math.abs;}",
            "export function update(){thisLayer=1;}",
            "export function init(){let a=engine.registerAudioBuffers(64);} export function update(){}",
            "export function applyUserProperties(p){let a=engine.registerAudioBuffers(64);} export function update(){}",
        ] {
            let error = prepare(source).unwrap_err();
            assert_eq!(
                super::super::unsupported_feature_code(&error),
                Some("scenescript")
            );
        }
    }

    #[test]
    fn loops_and_creation_budgets_are_enforced() {
        let started = Instant::now();
        assert!(prepare("export function update(){while(true){}}").is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(prepare("export function init(){for(let i=0;i<128;i++){thisScene.createLayer('models/x.json');}} export function update(){}").is_err());
    }

    #[test]
    fn undefined_return_retains_binding_and_boolean_return_applies() {
        let unchanged = prepare("export function update(){}").unwrap();
        assert!(unchanged.initial_states()[0].visible);
        let changed = prepare("export function update(){return false;}").unwrap();
        assert!(!changed.initial_states()[0].visible);
        let initialized =
            prepare("export function init(){return false;} export function update(){}").unwrap();
        assert!(!initialized.initial_states()[0].visible);
    }

    fn wait_snapshot(worker: &ScriptWorker, time: f64, audio: [f32; 64]) -> Vec<LayerState> {
        let started = Instant::now();
        loop {
            if let Some(snapshot) = worker.poll_frame(time, audio).unwrap() {
                return snapshot;
            }
            assert!(started.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn workers_are_independent_and_cancellation_is_bounded() {
        let plan =
            prepare("let n=0; export function update(){n+=1; thisLayer.origin=new Vec3(n);}")
                .unwrap();
        let first = ScriptWorker::spawn(&plan).unwrap();
        let second = ScriptWorker::spawn(&plan).unwrap();
        assert_eq!(wait_snapshot(&first, 0.0, [0.0; 64])[0].origin, [1.0; 3]);
        assert_eq!(wait_snapshot(&second, 0.0, [0.0; 64])[0].origin, [1.0; 3]);
        assert!(first.poll_frame(0.0, [0.0; 64]).unwrap().is_none());
        assert!(first.mailbox.lock().unwrap().input.is_none());
        let started = Instant::now();
        drop(first);
        drop(second);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn contention_never_waits_and_later_faults_are_reported() {
        let plan =
            prepare("let n=0; export function update(){n+=1;if(n>1){while(true){}}}").unwrap();
        let worker = ScriptWorker::spawn(&plan).unwrap();
        let lock = worker.mailbox.lock().unwrap();
        let started = Instant::now();
        assert!(worker.poll_frame(0.0, [0.0; 64]).unwrap().is_none());
        assert!(started.elapsed() < Duration::from_millis(100));
        drop(lock);
        wait_snapshot(&worker, 0.0, [0.0; 64]);
        let started = Instant::now();
        loop {
            if let Err(error) = worker.poll_frame(1.0, [0.0; 64]) {
                assert_eq!(
                    super::super::unsupported_feature_code(&error),
                    Some("scenescript")
                );
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(1));
        }
        let started = Instant::now();
        drop(worker);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn pause_without_submission_keeps_vm_state_and_returns_use_current_value() {
        let plan = prepare("export function update(current){return !current;}").unwrap();
        let (mut first, _) = Vm::new(&plan, Arc::new(AtomicBool::new(false))).unwrap();
        let (mut second, _) = Vm::new(&plan, Arc::new(AtomicBool::new(false))).unwrap();
        assert!(!first.frame(0.0, [0.0; 64]).unwrap()[0].visible);
        assert!(first.frame(1.0, [0.0; 64]).unwrap()[0].visible);
        // An independent paused instance has no callback to advance its state.
        assert!(!second.frame(10.0, [0.0; 64]).unwrap()[0].visible);
    }

    #[test]
    fn snapshot_requires_a_resource_record_for_every_added_handle() {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();
        let mut added = layer();
        added.handle = 8;
        context.with(|ctx| {
            for payload in [
                serde_json::json!({"states":[layer(),added],"created":[],"audio":false}),
                serde_json::json!({"states":[layer()],"created":[{"path":"models/bar.json","state":added}],"audio":false}),
                serde_json::json!({"states":[layer(),layer()],"created":[],"audio":false}),
                serde_json::json!({"states":[layer(),added],"created":[{"path":"models/../x.json","state":added}],"audio":false}),
            ] {
                let host = Object::new(ctx.clone()).unwrap();
                let json = payload.to_string();
                host.set("snapshot", Function::new(ctx.clone(), move || json.clone()).unwrap()).unwrap();
                assert!(read_snapshot(&host, &BTreeSet::from([7])).is_err());
            }
        });
    }
}
