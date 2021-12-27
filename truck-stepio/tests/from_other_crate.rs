espr_derive::inline_express!("
SCHEMA ap04x;

TYPE label = STRING; 
END_TYPE;

ENTITY representation_item;
  name : label;
WHERE
  WR1: SIZEOF(using_representations(SELF)) > 0;
END_ENTITY;

ENTITY geometric_representation_item
  SUPERTYPE OF (ONEOF(point, direction, vector, placement))
  SUBTYPE OF (representation_item);
DERIVE
  dim : dimension_count := dimension_of(SELF);
WHERE
  WR1: SIZEOF (QUERY (using_rep <* using_representations (SELF) |
       NOT ('GEOMETRY_SCHEMA.GEOMETRIC_REPRESENTATION_CONTEXT' IN
       TYPEOF (using_rep.context_of_items)))) = 0;
END_ENTITY;

TYPE length_measure = REAL;
END_TYPE;

TYPE plane_angle_measure = REAL;
END_TYPE;

TYPE parameter_value = REAL;
END_TYPE;

TYPE positive_length_measure = length_measure;
WHERE
  WR1: SELF > 0;
END_TYPE;

ENTITY point
   SUPERTYPE OF (ONEOF(cartesian_point))
   SUBTYPE OF (geometric_representation_item);
END_ENTITY;

ENTITY cartesian_point
  SUPERTYPE OF (ONEOF(cylindrical_point, spherical_point, polar_point))
  SUBTYPE OF (point);
   coordinates  : LIST [1:3] OF length_measure;
END_ENTITY;

ENTITY cylindrical_point
  SUBTYPE OF (cartesian_point);
    r     : length_measure;
    theta : plane_angle_measure;
    z     : length_measure;
  DERIVE
    SELF\\cartesian_point.coordinates : LIST [1:3] OF length_measure :=
                    [r*cos(theta), r*sin(theta), z];
  WHERE
   WR1: r >= 0.0;
END_ENTITY;

ENTITY spherical_point
  SUBTYPE OF (cartesian_point);
    r     : length_measure;
    theta : plane_angle_measure;
    phi   : plane_angle_measure;
  DERIVE
    SELF\\cartesian_point.coordinates : LIST [1:3] OF length_measure :=
      [r*sin(theta)*cos(phi), r*sin(theta)*sin(phi), r*cos(theta)];
  WHERE
   WR1: r >= 0.0;
END_ENTITY;

ENTITY polar_point
  SUBTYPE OF (cartesian_point);
    r     : length_measure;
    theta : plane_angle_measure;
  DERIVE
    SELF\\cartesian_point.coordinates : LIST [1:3] OF length_measure :=
                  [r*cos(theta), r*sin(theta)];
  WHERE
   WR1: r >= 0.0;
END_ENTITY;

ENTITY direction
   SUBTYPE OF (geometric_representation_item);
   direction_ratios : LIST [2:3] OF REAL;
 WHERE
   WR1: SIZEOF(QUERY(tmp <* direction_ratios | tmp <> 0.0)) > 0;
END_ENTITY;

ENTITY vector
   SUBTYPE OF (geometric_representation_item);
   orientation : direction;
   magnitude   : length_measure;
 WHERE
   WR1 : magnitude >= 0.0;
END_ENTITY;

ENTITY placement
   SUPERTYPE OF (ONEOF(axis1_placement,axis2_placement_2d,axis2_placement_3d))
   SUBTYPE OF (geometric_representation_item);
   location : cartesian_point;
END_ENTITY;

ENTITY axis1_placement
 SUBTYPE OF (placement);
   axis     : OPTIONAL direction;
 DERIVE
   z : direction := NVL(normalise(axis), dummy_gri ||
                                 direction([0.0,0.0,1.0]));
 WHERE
   WR1: SELF\\geometric_representation_item.dim  = 3;
END_ENTITY;

TYPE axis2_placement = SELECT 
   (axis2_placement_2d, 
    axis2_placement_3d);
END_TYPE;

ENTITY axis2_placement_2d
   SUBTYPE OF (placement);
   ref_direction : OPTIONAL direction;
 DERIVE
   p             : LIST [2:2] OF direction := build_2axes(ref_direction);
 WHERE
   WR1: SELF\\geometric_representation_item.dim = 2;
END_ENTITY;

ENTITY axis2_placement_3d
   SUBTYPE OF (placement);
   axis          : OPTIONAL direction;
   ref_direction : OPTIONAL direction;
 DERIVE
   p             : LIST [3:3] OF direction := build_axes(axis,ref_direction);
 WHERE
   WR1: SELF\\placement.location.dim = 3;
   WR2: (NOT (EXISTS (axis))) OR (axis.dim = 3);
   WR3: (NOT (EXISTS (ref_direction))) OR (ref_direction.dim = 3);
   WR4: (NOT (EXISTS (axis))) OR (NOT (EXISTS (ref_direction))) OR
          (cross_product(axis,ref_direction).magnitude > 0.0);
END_ENTITY;

ENTITY curve
   SUPERTYPE OF (ONEOF(line, conic
    (*, pcurve, surface_curve, offset_curve_2d, offset_curve_3d, curve_replica*)
        ))
   SUBTYPE OF (geometric_representation_item);
END_ENTITY;

ENTITY line
   SUBTYPE OF (curve);
   pnt : cartesian_point;
   dir : vector;
 WHERE
   WR1: dir.dim  = pnt.dim;
END_ENTITY;

ENTITY conic
   SUPERTYPE OF (ONEOF(circle, ellipse, hyperbola, parabola))
   SUBTYPE OF (curve);
   position: axis2_placement;
END_ENTITY;

ENTITY circle
   SUBTYPE OF (conic);
   radius   : positive_length_measure;
END_ENTITY;

ENTITY ellipse
   SUBTYPE OF (conic);
   semi_axis_1 : positive_length_measure;
   semi_axis_2 : positive_length_measure;
END_ENTITY;

ENTITY hyperbola
   SUBTYPE OF (conic);
   semi_axis      : positive_length_measure;
   semi_imag_axis : positive_length_measure;
END_ENTITY;

ENTITY parabola
   SUBTYPE OF (conic);
   focal_dist : length_measure;
 WHERE
   WR1: focal_dist <> 0.0;
END_ENTITY;

ENTITY bounded_curve
   SUPERTYPE OF (ONEOF(polyline, b_spline_curve
    (*, trimmed_curve, bounded_pcurve, bounded_surface_curve, composite_curve*)
        ))
   SUBTYPE OF (curve);
END_ENTITY;

ENTITY polyline
   SUBTYPE OF (bounded_curve);
   points : LIST [2:?] OF cartesian_point;
END_ENTITY;

TYPE b_spline_curve_form = ENUMERATION OF
   (polyline_form,
    circular_arc,
    elliptic_arc,
    parabolic_arc,
    hyperbolic_arc,
    unspecified);
END_TYPE;

TYPE knot_type = ENUMERATION OF 
   (uniform_knots,
    unspecified,
    quasi_uniform_knots,
    piecewise_bezier_knots);
END_TYPE;

ENTITY b_spline_curve
   SUPERTYPE OF (ONEOF(uniform_curve, b_spline_curve_with_knots,
                       quasi_uniform_curve, bezier_curve)
                         ANDOR rational_b_spline_curve)
   SUBTYPE OF (bounded_curve);
   degree               : INTEGER;
   control_points_list  : LIST [2:?] OF cartesian_point;
   curve_form           : b_spline_curve_form;
   closed_curve         : LOGICAL;
   self_intersect       : LOGICAL;
 DERIVE
   upper_index_on_control_points  : INTEGER 
                                  := (SIZEOF(control_points_list) - 1);
   control_points       : ARRAY [0:upper_index_on_control_points]
                                                         OF cartesian_point 
                                  := list_to_array(control_points_list,0,
                                             upper_index_on_control_points);
 WHERE
   WR1: ('GEOMETRY_SCHEMA.UNIFORM_CURVE' IN TYPEOF(self)) OR
        ('GEOMETRY_SCHEMA.QUASI_UNIFORM_CURVE' IN TYPEOF(self)) OR
        ('GEOMETRY_SCHEMA.BEZIER_CURVE' IN TYPEOF(self)) OR
        ('GEOMETRY_SCHEMA.B_SPLINE_CURVE_WITH_KNOTS' IN TYPEOF(self));
END_ENTITY;

ENTITY b_spline_curve_with_knots
   SUBTYPE OF (b_spline_curve);
   knot_multiplicities  : LIST [2:?] OF INTEGER;
   knots                : LIST [2:?] OF parameter_value;
   knot_spec            : knot_type;
 DERIVE
   upper_index_on_knots : INTEGER := SIZEOF(knots);
 WHERE
   WR1: constraints_param_b_spline(degree, upper_index_on_knots,
                               upper_index_on_control_points,
                               knot_multiplicities, knots);
    WR2: SIZEOF(knot_multiplicities) = upper_index_on_knots;
END_ENTITY;

ENTITY uniform_curve
   SUBTYPE OF (b_spline_curve);
END_ENTITY;

ENTITY quasi_uniform_curve
   SUBTYPE OF (b_spline_curve);
END_ENTITY;

ENTITY bezier_curve
   SUBTYPE OF (b_spline_curve);
END_ENTITY;

ENTITY rational_b_spline_curve
   SUBTYPE OF (b_spline_curve);
   weights_data : LIST [2:?] OF REAL;
                                
 DERIVE
   weights              : ARRAY [0:upper_index_on_control_points] OF REAL
                                  := list_to_array(weights_data,0,
                                         upper_index_on_control_points);
 WHERE
   WR1:  SIZEOF(weights_data) = SIZEOF(SELF\\b_spline_curve.control_points_list);
   WR2:  curve_weights_positive(SELF);
END_ENTITY;

ENTITY surface
   SUPERTYPE OF (ONEOF(elementary_surface, swept_surface
    (*, bounded_surface, offset_surface, surface_replica*)))
   SUBTYPE OF (geometric_representation_item);
END_ENTITY;

ENTITY elementary_surface
   SUPERTYPE OF (ONEOF(plane, cylindrical_surface, conical_surface,
                       spherical_surface, toroidal_surface))
   SUBTYPE OF (surface);
   position : axis2_placement_3d;
END_ENTITY;

ENTITY plane
 SUBTYPE OF (elementary_surface);
END_ENTITY;

ENTITY
 cylindrical_surface
   SUBTYPE OF (elementary_surface);
   radius : positive_length_measure;
END_ENTITY;

ENTITY
 conical_surface
   SUBTYPE OF (elementary_surface);
   radius     : length_measure;
   semi_angle : plane_angle_measure;
 WHERE
   WR1: radius >= 0.0;
END_ENTITY;

ENTITY spherical_surface
   SUBTYPE OF (elementary_surface);
   radius   : positive_length_measure;
END_ENTITY;

ENTITY toroidal_surface
   SUBTYPE OF (elementary_surface);
   major_radius : positive_length_measure;
   minor_radius : positive_length_measure;
END_ENTITY;

ENTITY degenerate_toroidal_surface
   SUBTYPE OF (toroidal_surface);
   select_outer : BOOLEAN;
 WHERE
  WR1: major_radius <   minor_radius;
END_ENTITY;

ENTITY swept_surface
   SUPERTYPE OF (ONEOF(surface_of_linear_extrusion, surface_of_revolution))
   SUBTYPE OF (surface);
   swept_curve : curve;
END_ENTITY;

ENTITY surface_of_linear_extrusion
   SUBTYPE OF (swept_surface);
   extrusion_axis      : vector;
END_ENTITY;

ENTITY surface_of_revolution
  SUBTYPE OF (swept_surface);
  axis_position       : axis1_placement;
DERIVE
  axis_line : line := representation_item('')||
                    geometric_representation_item()|| curve()||
                    line(axis_position.location, representation_item('')||
                    geometric_representation_item()||
                    vector(axis_position.z, 1.0));
END_ENTITY;

END_SCHEMA;
");

truck_stepio::parse_primitives!(ap04x, __parse_primitives);
truck_stepio::impl_curve!(ap04x, __impl_curve);
truck_stepio::impl_surface!(ap04x, __impl_surface);
