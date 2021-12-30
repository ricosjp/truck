mod build {
	#[path = "../config_control_design.rs"]
	pub mod config_control_design;
	truck_stepio::parse_primitives!(config_control_design, __parse_primitives);
	truck_stepio::impl_curve!(config_control_design, __impl_curve);
	truck_stepio::impl_surface!(config_control_design, __impl_surface);
}
mod parse;
