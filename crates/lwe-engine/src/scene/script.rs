//! Pure scalar SceneScript bindings, evaluated on the preparation worker.
//!
//! Oxc parses the unmodified ECMAScript module. The accepted AST consists of
//! scalar module variables, one `applyUserProperties` callback and a pure
//! `update` return expression. QuickJS supplies the language semantics; this is
//! not an expression translator. No interpreter or callbacks reach the renderer.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, BindingPatternKind, Declaration, Expression, Function,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rquickjs::{Context, Function as JsFunction, Module, Object, Runtime};

use super::unsupported;

pub(super) const MAX_SOURCE_BYTES: usize = 16 * 1024;
const MAX_DELIMITERS: usize = 256;
const MAX_VARIABLES: usize = 32;
const MAX_AST_DEPTH: usize = 32;
const MEMORY_BYTES: usize = 4 * 1024 * 1024;
const STACK_BYTES: usize = 256 * 1024;
const TIME_LIMIT: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy)]
pub(super) enum ScalarProperty {
    Number(f64),
    Bool(bool),
}

pub(super) type PropertySnapshot = BTreeMap<String, ScalarProperty>;

fn blocked(detail: &'static str) -> anyhow::Error {
    unsupported("scenescript", detail)
}

/// Validate the original module before executing it in a fresh bounded VM.
///
/// Imports, host APIs, time, random values, loops, nested functions, dynamic
/// property access, object construction and mutations from `update` remain
/// unsupported. The update expression therefore has the same result every
/// frame for this fixed validated property snapshot. Reading update's current
/// value is rejected: a feedback expression cannot be folded at preparation.
/// Property assignments cannot read old module state, and their only admitted
/// conditions check the presence of a known control in the complete snapshot.
pub(super) fn evaluate(source: &str, input: f64, properties: &PropertySnapshot) -> Result<f64> {
    validate(source, properties)?;
    if !finite(input) {
        return Err(blocked("script binding requires a finite scalar input"));
    }
    let runtime = Runtime::new().map_err(|_| blocked("cannot initialize scalar script runtime"))?;
    runtime.set_memory_limit(MEMORY_BYTES);
    runtime.set_max_stack_size(STACK_BYTES);
    let deadline = Instant::now() + TIME_LIMIT;
    runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)));
    let context =
        Context::full(&runtime).map_err(|_| blocked("cannot initialize scalar script context"))?;
    let result = context.with(|ctx| -> rquickjs::Result<Option<f64>> {
        let module = Module::declare(ctx.clone(), "lwe-scalar-binding", source)?;
        let (module, completion) = module.eval()?;
        completion.finish::<()>()?;
        let snapshot = Object::new(ctx.clone())?;
        for (name, property) in properties {
            // Every name read by the admitted AST has already been checked;
            // no author-supplied functions, accessors or prototypes are passed.
            if !safe_identifier(name) {
                continue;
            }
            match property {
                ScalarProperty::Number(value) => snapshot.set(name.as_str(), *value)?,
                ScalarProperty::Bool(value) => snapshot.set(name.as_str(), *value)?,
            }
        }
        let apply: JsFunction = module.get("applyUserProperties")?;
        apply.call::<_, ()>((snapshot,))?;
        let update: JsFunction = module.get("update")?;
        let output: rquickjs::Value = update.call((input,))?;
        Ok(output.as_number())
    });
    result
        .map_err(|_| blocked("scalar script evaluation failed or exceeded its limits"))?
        .filter(|output| finite(*output))
        .ok_or_else(|| blocked("scalar script must return one finite number"))
}

fn finite(value: f64) -> bool {
    value.is_finite() && (value as f32).is_finite()
}

fn validate(source: &str, properties: &PropertySnapshot) -> Result<()> {
    // The delimiter count is a conservative allocation/recursion guard, not a
    // syntax parser. Counting comments and strings as well can only reject an
    // over-budget source; it cannot admit unparsed syntax.
    if source.is_empty()
        || source.len() > MAX_SOURCE_BYTES
        || source
            .bytes()
            .filter(|byte| b"{}()[]".contains(byte))
            .count()
            > MAX_DELIMITERS
    {
        return Err(blocked(
            "scalar script source exceeds the preparation limit",
        ));
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if !parsed.errors.is_empty() || parsed.panicked {
        return Err(blocked("invalid scalar script module"));
    }
    let mut variables = BTreeSet::new();
    let mut callbacks = BTreeMap::new();
    for statement in &parsed.program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                if !matches!(declaration.kind.as_str(), "let" | "const") {
                    return Err(blocked("unsupported scalar script declaration"));
                }
                for declaration in &declaration.declarations {
                    let name = binding_name(&declaration.id)?;
                    if !safe_identifier(name)
                        || matches!(name, "Math" | "update" | "applyUserProperties")
                        || !variables.insert(name.to_owned())
                        || variables.len() > MAX_VARIABLES
                    {
                        return Err(blocked("invalid scalar script variable"));
                    }
                    literal(declaration.init.as_ref().ok_or_else(|| {
                        blocked("scalar script variables require literal initializers")
                    })?)?;
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if export.source.is_some() || !export.specifiers.is_empty() {
                    return Err(blocked("script imports and re-exports are not supported"));
                }
                let Some(Declaration::FunctionDeclaration(function)) = &export.declaration else {
                    return Err(blocked("only scalar script callbacks may be exported"));
                };
                let name = function
                    .id
                    .as_ref()
                    .map(|id| id.name.as_str())
                    .ok_or_else(|| blocked("script callback requires a name"))?;
                if !matches!(name, "update" | "applyUserProperties")
                    || callbacks.insert(name, function.as_ref()).is_some()
                {
                    return Err(blocked("unsupported or duplicate scalar script callback"));
                }
            }
            _ => return Err(blocked("unsupported scalar script module statement")),
        }
    }
    if callbacks.len() != 2 {
        return Err(blocked(
            "scalar scripts require property and update callbacks",
        ));
    }
    for (name, function) in callbacks {
        let parameter = callback_parameter(function)?;
        if variables.contains(parameter)
            || matches!(parameter, "Math" | "update" | "applyUserProperties")
        {
            return Err(blocked("script callback parameter shadows a binding"));
        }
        let body = function.body.as_ref().expect("callback body validated");
        let validation = Validation {
            variables: &variables,
            properties,
            parameter,
        };
        if name == "update" {
            let [Statement::ReturnStatement(statement)] = body.statements.as_slice() else {
                return Err(blocked("update must contain only one pure scalar return"));
            };
            validation.expression(
                statement
                    .argument
                    .as_ref()
                    .ok_or_else(|| blocked("update requires a scalar return expression"))?,
                false,
                0,
            )?;
        } else {
            for statement in &body.statements {
                validation.property_statement(statement, 0)?;
            }
        }
    }
    Ok(())
}

fn binding_name<'a>(binding: &'a BindingPattern<'_>) -> Result<&'a str> {
    match &binding.kind {
        BindingPatternKind::BindingIdentifier(identifier) => Ok(identifier.name.as_str()),
        _ => Err(blocked("destructuring script bindings are not supported")),
    }
}

fn callback_parameter<'a>(function: &'a Function<'_>) -> Result<&'a str> {
    if function.r#async
        || function.generator
        || function.params.rest.is_some()
        || function.body.is_none()
    {
        return Err(blocked(
            "async and generator script callbacks are not supported",
        ));
    }
    let [parameter] = function.params.items.as_slice() else {
        return Err(blocked(
            "script callbacks require one scalar or property parameter",
        ));
    };
    let name = binding_name(&parameter.pattern)?;
    if !safe_identifier(name) {
        return Err(blocked("invalid script callback parameter"));
    }
    Ok(name)
}

fn safe_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && !matches!(
            name,
            "__proto__" | "prototype" | "constructor" | "hasOwnProperty"
        )
}

fn literal(expression: &Expression<'_>) -> Result<()> {
    match expression {
        Expression::NumericLiteral(number) if finite(number.value) => Ok(()),
        Expression::BooleanLiteral(_) => Ok(()),
        Expression::UnaryExpression(unary) if matches!(unary.operator.as_str(), "+" | "-") => {
            match &unary.argument {
                Expression::NumericLiteral(number) if finite(number.value) => Ok(()),
                _ => Err(blocked("script initializers must be scalar literals")),
            }
        }
        _ => Err(blocked("script initializers must be scalar literals")),
    }
}

struct Validation<'a> {
    variables: &'a BTreeSet<String>,
    properties: &'a PropertySnapshot,
    parameter: &'a str,
}

impl Validation<'_> {
    fn depth(&self, depth: usize) -> Result<()> {
        if depth > MAX_AST_DEPTH {
            Err(blocked("scalar script AST exceeds the preparation limit"))
        } else {
            Ok(())
        }
    }

    fn property_name(&self, name: &str) -> Result<()> {
        if safe_identifier(name) && self.properties.contains_key(name) {
            Ok(())
        } else {
            Err(blocked(
                "script property must name a declared scalar control",
            ))
        }
    }

    fn property_statement(&self, statement: &Statement<'_>, depth: usize) -> Result<()> {
        self.depth(depth)?;
        match statement {
            Statement::BlockStatement(block) => {
                for statement in &block.body {
                    self.property_statement(statement, depth + 1)?;
                }
                Ok(())
            }
            Statement::IfStatement(statement) => {
                // Declared controls are always present in the full snapshot.
                // This exact test cannot leave a previous callback's scalar
                // state behind when a property value changes.
                let Expression::CallExpression(test) = &statement.test else {
                    return Err(blocked(
                        "property conditions require a declared-control presence check",
                    ));
                };
                let Expression::StaticMemberExpression(member) = &test.callee else {
                    return Err(blocked(
                        "property conditions require a declared-control presence check",
                    ));
                };
                if member.property.name != "hasOwnProperty"
                    || !matches!(&member.object, Expression::Identifier(object) if object.name == self.parameter)
                {
                    return Err(blocked(
                        "property conditions require a declared-control presence check",
                    ));
                }
                self.expression(&statement.test, true, depth + 1)?;
                self.property_statement(&statement.consequent, depth + 1)?;
                if let Some(alternate) = &statement.alternate {
                    self.property_statement(alternate, depth + 1)?;
                }
                Ok(())
            }
            Statement::ExpressionStatement(statement) => {
                let Expression::AssignmentExpression(assignment) = &statement.expression else {
                    return Err(blocked(
                        "property callbacks may only assign scalar variables",
                    ));
                };
                let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
                    return Err(blocked("script property mutation is not supported"));
                };
                if assignment.operator.as_str() != "="
                    || !self.variables.contains(target.name.as_str())
                {
                    return Err(blocked(
                        "property callbacks may only assign declared variables",
                    ));
                }
                self.expression(&assignment.right, true, depth + 1)
            }
            _ => Err(blocked("unsupported scalar property callback statement")),
        }
    }

    fn expression(&self, expression: &Expression<'_>, property: bool, depth: usize) -> Result<()> {
        self.depth(depth)?;
        match expression {
            Expression::NumericLiteral(number) if finite(number.value) => Ok(()),
            Expression::BooleanLiteral(_) => Ok(()),
            Expression::Identifier(identifier)
                if !property && self.variables.contains(identifier.name.as_str()) =>
            {
                Ok(())
            }
            Expression::ParenthesizedExpression(expression) => {
                self.expression(&expression.expression, property, depth + 1)
            }
            Expression::UnaryExpression(unary)
                if matches!(unary.operator.as_str(), "+" | "-" | "!") =>
            {
                self.expression(&unary.argument, property, depth + 1)
            }
            Expression::BinaryExpression(binary)
                if matches!(
                    binary.operator.as_str(),
                    "+" | "-" | "*" | "/" | "%" | "**" | "===" | "!==" | "<" | ">" | "<=" | ">="
                ) =>
            {
                self.expression(&binary.left, property, depth + 1)?;
                self.expression(&binary.right, property, depth + 1)
            }
            Expression::LogicalExpression(logical) => {
                self.expression(&logical.left, property, depth + 1)?;
                self.expression(&logical.right, property, depth + 1)
            }
            Expression::ConditionalExpression(conditional) => {
                self.expression(&conditional.test, property, depth + 1)?;
                self.expression(&conditional.consequent, property, depth + 1)?;
                self.expression(&conditional.alternate, property, depth + 1)
            }
            Expression::StaticMemberExpression(member)
                if property
                    && !member.optional
                    && matches!(&member.object, Expression::Identifier(identifier) if identifier.name == self.parameter) =>
            {
                self.property_name(member.property.name.as_str())
            }
            Expression::CallExpression(call) if !call.optional => {
                let Expression::StaticMemberExpression(member) = &call.callee else {
                    return Err(blocked(
                        "only bounded Math and property-presence calls are supported",
                    ));
                };
                let Expression::Identifier(object) = &member.object else {
                    return Err(blocked(
                        "nested and dynamic script member calls are not supported",
                    ));
                };
                let method = member.property.name.as_str();
                if property
                    && object.name == self.parameter
                    && method == "hasOwnProperty"
                    && !member.optional
                    && call.arguments.len() == 1
                {
                    let Some(Expression::StringLiteral(name)) = call.arguments[0].as_expression()
                    else {
                        return Err(blocked("property-presence checks require a literal name"));
                    };
                    return self.property_name(name.value.as_str());
                }
                let arity = call.arguments.len();
                let allowed = match method {
                    "abs" | "sign" | "sqrt" | "floor" | "ceil" | "round" | "trunc" | "sin"
                    | "cos" | "tan" | "exp" | "log" => arity == 1,
                    "pow" => arity == 2,
                    "min" | "max" => (1..=8).contains(&arity),
                    _ => false,
                };
                if object.name != "Math" || member.optional || !allowed {
                    return Err(blocked(
                        "script call is outside the pure scalar Math subset",
                    ));
                }
                for argument in &call.arguments {
                    self.expression(
                        argument
                            .as_expression()
                            .ok_or_else(|| blocked("spread script arguments are not supported"))?,
                        property,
                        depth + 1,
                    )?;
                }
                Ok(())
            }
            _ => Err(blocked(
                "script expression is outside the pure scalar subset",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::unsupported_feature_code;

    const PURE: &str = "let amount = 0; export function applyUserProperties(p) { if (p.hasOwnProperty('gain')) { amount = p.gain; } } export function update(current) { return Math.sqrt(Math.abs(amount)) * Math.sign(amount) + 1; }";

    #[test]
    fn vm_executes_unmodified_module_and_snapshot_changes() {
        assert_eq!(
            evaluate(
                PURE,
                1.0,
                &BTreeMap::from([("gain".into(), ScalarProperty::Number(9.0))])
            )
            .unwrap(),
            4.0
        );
        assert_eq!(
            evaluate(
                PURE,
                1.0,
                &BTreeMap::from([("gain".into(), ScalarProperty::Number(-16.0))])
            )
            .unwrap(),
            -3.0
        );
    }

    #[test]
    fn boolean_controls_and_scalar_conditionals_use_real_js_semantics() {
        let source = "let enabled = false; export function applyUserProperties(p) { enabled = p.enabled; } export function update(v) { return enabled ? 6 : -3; }";
        for (enabled, expected) in [(true, 6.0), (false, -3.0)] {
            assert_eq!(
                evaluate(
                    source,
                    3.0,
                    &BTreeMap::from([("enabled".into(), ScalarProperty::Bool(enabled))])
                )
                .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn broad_modules_host_apis_mutation_and_non_scalar_returns_fail_closed() {
        for expression in [
            "Date.now()",
            "Math.random()",
            "engine.runtime",
            "globalThis",
            "v.constructor('return 1')()",
            "amount++",
            "(amount = 2)",
            "v * 2",
            "p.gain",
            "{}",
            "[]",
            "true",
            "1/0",
            "Math.sqrt(-1)",
            "import('x')",
            "(function(){return 1})()",
        ] {
            let source = format!(
                "let amount = 0; export function applyUserProperties(p) {{ amount = p.gain; }} export function update(v) {{ return {expression}; }}"
            );
            let error = evaluate(
                &source,
                1.0,
                &BTreeMap::from([("gain".into(), ScalarProperty::Number(2.0))]),
            )
            .unwrap_err();
            assert_eq!(
                unsupported_feature_code(&error),
                Some("scenescript"),
                "{expression}"
            );
        }
        for source in [
            "import x from 'x'; export function applyUserProperties(p) {} export function update(v) { return 1; }",
            "export function init() {} export function applyUserProperties(p) {} export function update(v) { return 1; }",
            "export function applyUserProperties(p) { while (true) {} } export function update(v) { return 1; }",
            "export function applyUserProperties(p) { p.gain = 1; } export function update(v) { return 1; }",
            "let amount = Date.now(); export function applyUserProperties(p) {} export function update(v) { return 1; }",
            "let amount=0; export function applyUserProperties(p) { amount=p['gain']; } export function update(v) { return amount; }",
            "let amount=0; export function applyUserProperties(p) { amount=amount+1; } export function update(v) { return amount; }",
            "let amount=0; export function applyUserProperties(p) { if(p.gain>1) {amount=p.gain;} } export function update(v) { return amount; }",
            "export async function applyUserProperties(p) {} export function update(v) { return 1; }",
            "export function applyUserProperties(p) {} export function update(v) { v++; return v; }",
        ] {
            assert!(
                evaluate(source, 1.0, &PropertySnapshot::new()).is_err(),
                "{source}"
            );
        }
    }

    #[test]
    fn undeclared_controls_and_source_resource_limits_are_rejected() {
        assert!(evaluate(PURE, 1.0, &PropertySnapshot::new()).is_err());
        assert!(
            evaluate(
                &" ".repeat(MAX_SOURCE_BYTES + 1),
                1.0,
                &PropertySnapshot::new()
            )
            .is_err()
        );
        let nested = format!(
            "export function applyUserProperties(p) {{}} export function update(v) {{ return {}1{}; }}",
            "(".repeat(MAX_DELIMITERS),
            ")".repeat(MAX_DELIMITERS)
        );
        assert!(evaluate(&nested, 1.0, &PropertySnapshot::new()).is_err());
    }

    #[test]
    #[ignore = "requires a private original-source scalar binding fixture"]
    fn original_scalar_bindings_match_independent_numeric_expectations() {
        let path = std::env::var("LWE_SCENE_SCALAR_SCRIPT_FIXTURE")
            .expect("LWE_SCENE_SCALAR_SCRIPT_FIXTURE must name the private fixture");
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() <= 256 * 1024);
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert!(!cases.is_empty() && cases.len() <= 128);
        for case in cases {
            let properties = case["properties"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        if let Some(value) = value.as_bool() {
                            ScalarProperty::Bool(value)
                        } else {
                            ScalarProperty::Number(value.as_f64().unwrap())
                        },
                    )
                })
                .collect();
            let output = evaluate(
                case["source"].as_str().unwrap(),
                case["input"].as_f64().unwrap(),
                &properties,
            )
            .unwrap();
            let expected = case["expected"].as_f64().unwrap();
            assert!(
                (output - expected).abs() <= 1e-12,
                "{}: {output} != {expected}",
                case["name"]
            );
        }
    }
}
