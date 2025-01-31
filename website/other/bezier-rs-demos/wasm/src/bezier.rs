use crate::svg_drawing::*;
use bezier_rs::{ArcStrategy, ArcsOptions, Bezier, ProjectionOptions};
use glam::DVec2;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct CircleSector {
	center: Point,
	radius: f64,
	#[serde(rename = "startAngle")]
	start_angle: f64,
	#[serde(rename = "endAngle")]
	end_angle: f64,
}

#[derive(Serialize, Deserialize)]
struct Point {
	x: f64,
	y: f64,
}

#[wasm_bindgen]
pub enum WasmMaximizeArcs {
	Automatic, // 0
	On,        // 1
	Off,       // 2
}

const SCALE_UNIT_VECTOR_FACTOR: f64 = 50.;

/// Wrapper of the `Bezier` struct to be used in JS.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmBezier(Bezier);

/// Convert a `DVec2` into a `Point`.
fn vec_to_point(p: &DVec2) -> Point {
	Point { x: p.x, y: p.y }
}

/// Convert a bezier to a list of points.
fn bezier_to_points(bezier: Bezier) -> Vec<Point> {
	bezier.get_points().map(|point| Point { x: point.x, y: point.y }).collect()
}

/// Serialize some data and then convert it to a JsValue.
fn to_js_value<T: Serialize>(data: T) -> JsValue {
	JsValue::from_serde(&serde_json::to_string(&data).unwrap()).unwrap()
}

fn convert_wasm_maximize_arcs(wasm_enum_value: WasmMaximizeArcs) -> ArcStrategy {
	match wasm_enum_value {
		WasmMaximizeArcs::Automatic => ArcStrategy::Automatic,
		WasmMaximizeArcs::On => ArcStrategy::FavorLargerArcs,
		WasmMaximizeArcs::Off => ArcStrategy::FavorCorrectness,
	}
}

fn wrap_svg_tag(contents: String) -> String {
	format!("{}{}{}", SVG_OPEN_TAG, contents, SVG_CLOSE_TAG)
}

#[wasm_bindgen]
impl WasmBezier {
	/// Expect js_points to be a list of 2 pairs.
	pub fn new_linear(js_points: &JsValue) -> WasmBezier {
		let points: [DVec2; 2] = js_points.into_serde().unwrap();
		WasmBezier(Bezier::from_linear_dvec2(points[0], points[1]))
	}

	/// Expect js_points to be a list of 3 pairs.
	pub fn new_quadratic(js_points: &JsValue) -> WasmBezier {
		let points: [DVec2; 3] = js_points.into_serde().unwrap();
		WasmBezier(Bezier::from_quadratic_dvec2(points[0], points[1], points[2]))
	}

	/// Expect js_points to be a list of 4 pairs.
	pub fn new_cubic(js_points: &JsValue) -> WasmBezier {
		let points: [DVec2; 4] = js_points.into_serde().unwrap();
		WasmBezier(Bezier::from_cubic_dvec2(points[0], points[1], points[2], points[3]))
	}

	fn draw_bezier_through_points(bezier: Bezier, through_point: DVec2) -> String {
		let mut bezier_string = String::new();
		bezier.to_svg(
			&mut bezier_string,
			CURVE_ATTRIBUTES.to_string(),
			ANCHOR_ATTRIBUTES.to_string(),
			HANDLE_ATTRIBUTES.to_string().replace(GRAY, RED),
			HANDLE_LINE_ATTRIBUTES.to_string().replace(GRAY, RED),
		);
		let through_point_circle = format!(r#"<circle cx="{}" cy="{}" {}/>"#, through_point.x, through_point.y, ANCHOR_ATTRIBUTES.to_string());

		wrap_svg_tag(format!("{bezier_string}{through_point_circle}"))
	}

	pub fn quadratic_through_points(js_points: &JsValue, t: f64) -> String {
		let points: [DVec2; 3] = js_points.into_serde().unwrap();
		let bezier = Bezier::quadratic_through_points(points[0], points[1], points[2], Some(t));
		WasmBezier::draw_bezier_through_points(bezier, points[1])
	}

	pub fn cubic_through_points(js_points: &JsValue, t: f64, midpoint_separation: f64) -> String {
		let points: [DVec2; 3] = js_points.into_serde().unwrap();
		let bezier = Bezier::cubic_through_points(points[0], points[1], points[2], Some(t), Some(midpoint_separation));
		WasmBezier::draw_bezier_through_points(bezier, points[1])
	}

	pub fn set_start(&mut self, x: f64, y: f64) {
		self.0.set_start(DVec2::new(x, y));
	}

	pub fn set_end(&mut self, x: f64, y: f64) {
		self.0.set_end(DVec2::new(x, y));
	}

	pub fn set_handle_start(&mut self, x: f64, y: f64) {
		self.0.set_handle_start(DVec2::new(x, y));
	}

	pub fn set_handle_end(&mut self, x: f64, y: f64) {
		self.0.set_handle_end(DVec2::new(x, y));
	}

	/// The wrapped return type is `Vec<Point>`.
	pub fn get_points(&self) -> JsValue {
		let points: Vec<Point> = self.0.get_points().map(|point| vec_to_point(&point)).collect();
		to_js_value(points)
	}

	fn get_bezier_path(&self) -> String {
		let mut bezier = String::new();
		self.0.to_svg(
			&mut bezier,
			CURVE_ATTRIBUTES.to_string(),
			ANCHOR_ATTRIBUTES.to_string(),
			HANDLE_ATTRIBUTES.to_string(),
			HANDLE_LINE_ATTRIBUTES.to_string(),
		);
		bezier
	}

	pub fn to_svg(&self) -> String {
		wrap_svg_tag(self.get_bezier_path())
	}

	pub fn length(&self) -> String {
		let bezier = self.get_bezier_path();
		wrap_svg_tag(format!("{bezier}{}", draw_text(format!("Length: {:.2}", self.0.length(None)), TEXT_OFFSET_X, TEXT_OFFSET_Y, BLACK)))
	}

	/// The wrapped return type is `Point`.
	pub fn evaluate_value(&self, t: f64) -> JsValue {
		let point: Point = vec_to_point(&self.0.evaluate(t));
		to_js_value(point)
	}

	pub fn evaluate(&self, t: f64) -> String {
		let bezier = self.get_bezier_path();
		let point = &self.0.evaluate(t);
		let content = format!("{bezier}{}", draw_circle(point.x, point.y, 4., RED, 1.5, WHITE));
		wrap_svg_tag(content)
	}

	pub fn compute_lookup_table(&self, steps: usize) -> String {
		let bezier = self.get_bezier_path();
		let table_values: Vec<Point> = self.0.compute_lookup_table(Some(steps)).iter().map(vec_to_point).collect();
		let circles: String = table_values
			.iter()
			.map(|point| draw_circle(point.x, point.y, 3., RED, 1.5, WHITE))
			.fold("".to_string(), |acc, circle| acc + &circle);
		let content = format!("{bezier}{circles}");
		wrap_svg_tag(content)
	}

	pub fn derivative(&self) -> String {
		let bezier = self.get_bezier_path();
		let derivative = self.0.derivative();
		if derivative.is_none() {
			return bezier;
		}

		let mut derivative_svg_path = String::new();
		derivative.unwrap().to_svg(
			&mut derivative_svg_path,
			CURVE_ATTRIBUTES.to_string().replace(BLACK, RED),
			ANCHOR_ATTRIBUTES.to_string().replace(BLACK, RED),
			HANDLE_ATTRIBUTES.to_string().replace(GRAY, RED),
			HANDLE_LINE_ATTRIBUTES.to_string().replace(GRAY, RED),
		);
		let content = format!("{bezier}{derivative_svg_path}");
		wrap_svg_tag(content)
	}

	/// The wrapped return type is `Point`.
	pub fn tangent(&self, t: f64) -> String {
		let bezier = self.get_bezier_path();

		let tangent_point = self.0.tangent(t);
		let intersection_point = self.0.evaluate(t);
		let tangent_end = intersection_point + tangent_point * SCALE_UNIT_VECTOR_FACTOR;

		let content = format!(
			"{bezier}{}{}{}",
			draw_circle(intersection_point.x, intersection_point.y, 3., RED, 1., WHITE),
			draw_line(intersection_point.x, intersection_point.y, tangent_end.x, tangent_end.y, RED, 1.),
			draw_circle(tangent_end.x, tangent_end.y, 3., RED, 1., WHITE),
		);
		wrap_svg_tag(content)
	}

	pub fn normal(&self, t: f64) -> String {
		let bezier = self.get_bezier_path();

		let normal_point = self.0.normal(t);
		let intersection_point = self.0.evaluate(t);
		let normal_end = intersection_point + normal_point * SCALE_UNIT_VECTOR_FACTOR;

		let content = format!(
			"{bezier}{}{}{}",
			draw_line(intersection_point.x, intersection_point.y, normal_end.x, normal_end.y, RED, 1.),
			draw_circle(intersection_point.x, intersection_point.y, 3., RED, 1., WHITE),
			draw_circle(normal_end.x, normal_end.y, 3., RED, 1., WHITE),
		);
		wrap_svg_tag(content)
	}

	pub fn curvature(&self, t: f64) -> String {
		let bezier = self.get_bezier_path();
		let radius = 1. / self.0.curvature(t);
		let normal_point = self.0.normal(t);
		let intersection_point = self.0.evaluate(t);

		let curvature_center = intersection_point + normal_point * radius;

		let content = format!(
			"{bezier}{}{}{}{}",
			draw_circle(curvature_center.x, curvature_center.y, radius.abs(), RED, 1., NONE),
			draw_line(intersection_point.x, intersection_point.y, curvature_center.x, curvature_center.y, RED, 1.),
			draw_circle(intersection_point.x, intersection_point.y, 3., RED, 1., WHITE),
			draw_circle(curvature_center.x, curvature_center.y, 3., RED, 1., WHITE),
		);
		wrap_svg_tag(content)
	}

	pub fn split(&self, t: f64) -> String {
		let beziers: [Bezier; 2] = self.0.split(t);

		let mut original_bezier_svg = String::new();
		self.0.to_svg(
			&mut original_bezier_svg,
			CURVE_ATTRIBUTES.to_string().replace(BLACK, WHITE),
			ANCHOR_ATTRIBUTES.to_string().replace(BLACK, WHITE),
			HANDLE_ATTRIBUTES.to_string(),
			HANDLE_LINE_ATTRIBUTES.to_string(),
		);

		let mut bezier_svg_1 = String::new();
		beziers[0].to_svg(
			&mut bezier_svg_1,
			CURVE_ATTRIBUTES.to_string().replace(BLACK, ORANGE),
			ANCHOR_ATTRIBUTES.to_string().replace(BLACK, ORANGE),
			HANDLE_ATTRIBUTES.to_string().replace(GRAY, ORANGE),
			HANDLE_LINE_ATTRIBUTES.to_string().replace(GRAY, ORANGE),
		);

		let mut bezier_svg_2 = String::new();
		beziers[1].to_svg(
			&mut bezier_svg_2,
			CURVE_ATTRIBUTES.to_string().replace(BLACK, RED),
			ANCHOR_ATTRIBUTES.to_string().replace(BLACK, RED),
			HANDLE_ATTRIBUTES.to_string().replace(GRAY, RED),
			HANDLE_LINE_ATTRIBUTES.to_string().replace(GRAY, RED),
		);

		wrap_svg_tag(format!("{original_bezier_svg}{bezier_svg_1}{bezier_svg_2}"))
	}

	pub fn trim(&self, t1: f64, t2: f64) -> String {
		let trimmed_bezier = self.0.trim(t1, t2);

		let mut trimmed_bezier_svg = String::new();
		trimmed_bezier.to_svg(
			&mut trimmed_bezier_svg,
			CURVE_ATTRIBUTES.to_string().replace(BLACK, RED),
			ANCHOR_ATTRIBUTES.to_string().replace(BLACK, RED),
			HANDLE_ATTRIBUTES.to_string().replace(GRAY, RED),
			HANDLE_LINE_ATTRIBUTES.to_string().replace(GRAY, RED),
		);

		wrap_svg_tag(format!("{}{trimmed_bezier_svg}", self.get_bezier_path()))
	}

	pub fn project(&self, x: f64, y: f64) -> String {
		let projected_t_value = self.0.project(DVec2::new(x, y), ProjectionOptions::default());
		let projected_point = self.0.evaluate(projected_t_value);

		let bezier = self.get_bezier_path();
		let content = format!("{bezier}{}", draw_line(projected_point.x, projected_point.y, x, y, RED, 1.),);
		wrap_svg_tag(content)
	}

	pub fn local_extrema(&self) -> String {
		let local_extrema: [Vec<f64>; 2] = self.0.local_extrema();

		let bezier = self.get_bezier_path();
		let circles: String = local_extrema
			.iter()
			.zip([RED, GREEN])
			.flat_map(|(t_value_list, color)| {
				t_value_list.iter().map(|&t_value| {
					let point = self.0.evaluate(t_value);
					draw_circle(point.x, point.y, 3., color, 1.5, WHITE)
				})
			})
			.fold("".to_string(), |acc, circle| acc + &circle);

		let content = format!(
			"{bezier}{circles}{}{}",
			draw_text("X extrema".to_string(), TEXT_OFFSET_X, TEXT_OFFSET_Y - 20., RED),
			draw_text("Y extrema".to_string(), TEXT_OFFSET_X, TEXT_OFFSET_Y, GREEN),
		);
		wrap_svg_tag(content)
	}

	pub fn bounding_box(&self) -> String {
		let [bbox_min_corner, bbox_max_corner] = self.0.bounding_box();

		let bezier = self.get_bezier_path();
		let content = format!(
			"{bezier}<rect x={} y ={} width=\"{}\" height=\"{}\" style=\"fill:{NONE};stroke:{RED};stroke-width:1\" />",
			bbox_min_corner.x,
			bbox_min_corner.y,
			bbox_max_corner.x - bbox_min_corner.x,
			bbox_max_corner.y - bbox_min_corner.y,
		);
		wrap_svg_tag(content)
	}

	pub fn inflections(&self) -> String {
		let inflections: Vec<f64> = self.0.inflections();

		let bezier = self.get_bezier_path();
		let circles: String = inflections
			.iter()
			.map(|&t_value| {
				let point = self.0.evaluate(t_value);
				draw_circle(point.x, point.y, 3., RED, 1.5, WHITE)
			})
			.fold("".to_string(), |acc, circle| acc + &circle);
		let content = format!("{bezier}{circles}");
		wrap_svg_tag(content)
	}

	/// The wrapped return type is `Vec<Vec<Point>>`.
	pub fn de_casteljau_points(&self, t: f64) -> JsValue {
		let points: Vec<Vec<Point>> = self
			.0
			.de_casteljau_points(t)
			.iter()
			.map(|level| level.iter().map(|&point| Point { x: point.x, y: point.y }).collect::<Vec<Point>>())
			.collect();
		to_js_value(points)
	}

	pub fn rotate(&self, angle: f64) -> WasmBezier {
		WasmBezier(self.0.rotate(angle))
	}

	fn intersect(&self, curve: &Bezier, error: Option<f64>) -> Vec<f64> {
		self.0.intersections(curve, error)
	}

	pub fn intersect_line_segment(&self, js_points: &JsValue) -> Vec<f64> {
		let points: [DVec2; 2] = js_points.into_serde().unwrap();
		let line = Bezier::from_linear_dvec2(points[0], points[1]);
		self.intersect(&line, None)
	}

	pub fn intersect_quadratic_segment(&self, js_points: &JsValue, error: f64) -> Vec<f64> {
		let points: [DVec2; 3] = js_points.into_serde().unwrap();
		let quadratic = Bezier::from_quadratic_dvec2(points[0], points[1], points[2]);
		self.intersect(&quadratic, Some(error))
	}

	pub fn intersect_cubic_segment(&self, js_points: &JsValue, error: f64) -> Vec<f64> {
		let points: [DVec2; 4] = js_points.into_serde().unwrap();
		let cubic = Bezier::from_cubic_dvec2(points[0], points[1], points[2], points[3]);
		self.intersect(&cubic, Some(error))
	}

	/// The wrapped return type is `Vec<[f64; 2]>`.
	pub fn intersect_self(&self, error: f64) -> JsValue {
		let points: Vec<[f64; 2]> = self.0.self_intersections(Some(error));
		to_js_value(points)
	}

	pub fn reduce(&self) -> String {
		let empty_string = String::new();
		let original_curve_svg = self.get_bezier_path();
		let bezier_curves_svg: String = self
			.0
			.reduce(None)
			.iter()
			.enumerate()
			.map(|(idx, bezier_curve)| {
				let mut curve_svg = String::new();
				bezier_curve.to_svg(
					&mut curve_svg,
					CURVE_ATTRIBUTES.to_string().replace(BLACK, &format!("hsl({}, 100%, 50%)", (40 * idx))),
					empty_string.clone(),
					empty_string.clone(),
					empty_string.clone(),
				);
				curve_svg
			})
			.fold(original_curve_svg, |acc, item| format!("{acc}{item}"));
		wrap_svg_tag(bezier_curves_svg)
	}

	pub fn offset(&self, distance: f64) -> String {
		let empty_string = String::new();
		let original_curve_svg = self.get_bezier_path();
		let bezier_curves_svg = self
			.0
			.offset(distance)
			.iter()
			.enumerate()
			.map(|(idx, bezier_curve)| {
				let mut curve_svg = String::new();
				bezier_curve.to_svg(
					&mut curve_svg,
					CURVE_ATTRIBUTES.to_string().replace(BLACK, &format!("hsl({}, 100%, 50%)", (40 * idx))),
					empty_string.clone(),
					empty_string.clone(),
					empty_string.clone(),
				);
				curve_svg
			})
			.fold(original_curve_svg, |acc, item| format!("{acc}{item}"));
		wrap_svg_tag(bezier_curves_svg)
	}

	/// The wrapped return type is `Vec<CircleSector>`.
	pub fn arcs(&self, error: f64, max_iterations: usize, maximize_arcs: WasmMaximizeArcs) -> JsValue {
		let strategy = convert_wasm_maximize_arcs(maximize_arcs);
		let options = ArcsOptions { error, max_iterations, strategy };
		let circle_sectors: Vec<CircleSector> = self
			.0
			.arcs(options)
			.iter()
			.map(|sector| CircleSector {
				center: Point {
					x: sector.center.x,
					y: sector.center.y,
				},
				radius: sector.radius,
				start_angle: sector.start_angle,
				end_angle: sector.end_angle,
			})
			.collect();
		to_js_value(circle_sectors)
	}
}
