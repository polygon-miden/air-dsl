use super::{AirIR, Impl, Scope};

mod boundary_constraints;
use boundary_constraints::add_fn_get_assertions;

mod transition_constraints;
use transition_constraints::add_fn_evaluate_transition;

// HELPERS TO GENERATE AN IMPLEMENTATION OF THE WINTERFELL AIR TRAIT
// ================================================================================================

/// Updates the provided scope with a new Air struct and Winterfell Air trait implementation
/// which are equivalent the provided AirIR.
pub(super) fn add_air(scope: &mut Scope, ir: &AirIR) {
    let name = ir.air_name();

    // add the Air struct and its base implementation.
    add_air_struct(scope, name);

    // add Winterfell Air trait implementation for the provided AirIR.
    add_air_trait(scope, ir, name);
}

/// Updates the provided scope with a custom Air struct.
fn add_air_struct(scope: &mut Scope, name: &str) {
    // define the custom Air struct.
    scope
        .new_struct(name)
        .vis("pub")
        .field("context", "AirContext<Felt>");

    // add the custom Air implementation block
    let base_impl = scope.new_impl(name);
    // add a simple method to get the last step.
    base_impl
        .new_fn("last_step")
        .vis("pub")
        .ret("usize")
        .line("self.trace_length() - self.context().num_transition_exemptions()");
}

/// Updates the provided scope with the custom Air struct and an Air trait implementation based on
/// the provided AirIR.
fn add_air_trait(scope: &mut Scope, ir: &AirIR, name: &str) {
    // add the implementation block for the Air trait.
    let air_impl = scope
        .new_impl(name)
        .impl_trait("Air")
        .associate_type("BaseField", "Felt");

    // add default function "context".
    let fn_context = air_impl
        .new_fn("context")
        .arg_ref_self()
        .ret("&AirContext<Felt>");
    fn_context.line("&self.context");

    // add the method implementations required by the AIR trait.
    add_fn_new(air_impl, ir);
    add_fn_get_assertions(air_impl, ir);
    add_fn_evaluate_transition(air_impl, ir);
}

/// Adds an implementation of the "new" method to the referenced Air implementation based on the
/// data in the provided AirIR.
fn add_fn_new(impl_ref: &mut Impl, ir: &AirIR) {
    // define the function.
    let new = impl_ref
        .new_fn("new")
        .arg("trace_info", "TraceInfo")
        .arg("options", "WinterProofOptions")
        .ret("Self");

    // define the transition constraint degrees of the main trace `main_degrees`.
    let mut main_degrees: Vec<String> = Vec::new();
    for degree in ir.main_degrees().iter() {
        main_degrees.push(format!("TransitionConstraintDegree::new({})", degree));
    }
    new.line(format!(
        "let main_degrees = vec![{}];",
        main_degrees.join(",")
    ));

    // define the transition constraint degrees of the aux trace `aux_degrees`.
    new.line("let aux_degrees = Vec::new();");

    // define the number of main trace boundary constraints `num_main_assertions`.
    new.line(format!(
        "let num_main_assertions = {};",
        ir.num_main_assertions()
    ));

    // define the number of aux trace boundary constraints `num_aux_assertions`.
    new.line("let num_aux_assertions = 0;");

    // define the context.
    let context = "
let context = AirContext::new_multi_segment(
    trace_info,
    main_degrees,
    aux_degrees,
    num_main_assertions,
    num_aux_assertions,
    options,
)
.set_num_transition_exemptions(2);";

    new.line(context);

    // return initialized Self.
    new.line("Self { context }");
}