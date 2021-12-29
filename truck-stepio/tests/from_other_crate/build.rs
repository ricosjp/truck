espr_derive::inline_express!(
  "
SCHEMA ap04x;

TYPE LABEL = STRING;
END_TYPE;

ENTITY representation_item;
  name : LABEL;
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

ENTITY bounded_surface
   SUPERTYPE OF (ONEOF(b_spline_surface
    (*, rectangular_trimmed_surface,
                       curve_bounded_surface, rectangular_composite_surface*)
    ))
   SUBTYPE OF (surface);
END_ENTITY;

TYPE b_spline_surface_form = ENUMERATION OF
   (plane_surf,
    cylindrical_surf,
    conical_surf,
    spherical_surf,
    toroidal_surf,
    surf_of_revolution,
    ruled_surf,
    generalised_cone,
    quadric_surf,
    surf_of_linear_extrusion,
    unspecified);
END_TYPE;

ENTITY b_spline_surface
   SUPERTYPE OF (ONEOF(b_spline_surface_with_knots, uniform_surface,
                       quasi_uniform_surface, bezier_surface)
                         ANDOR rational_b_spline_surface)
   SUBTYPE OF (bounded_surface);
   u_degree             : INTEGER;
   v_degree             : INTEGER;
   control_points_list  : LIST [2:?] OF LIST [2:?] OF cartesian_point;
   surface_form         : b_spline_surface_form;
   u_closed             : LOGICAL;
   v_closed             : LOGICAL;
   self_intersect       : LOGICAL;
 DERIVE
   u_upper             : INTEGER := SIZEOF(control_points_list) - 1;
   v_upper             : INTEGER := SIZEOF(control_points_list[1]) - 1;
   control_points      : ARRAY [0:u_upper] OF ARRAY [0:v_upper] OF 
                         cartesian_point 
                       := make_array_of_array(control_points_list,
                                              0,u_upper,0,v_upper);
 WHERE
   WR1: ('GEOMETRY_SCHEMA.UNIFORM_SURFACE' IN TYPEOF(SELF)) OR
        ('GEOMETRY_SCHEMA.QUASI_UNIFORM_SURFACE' IN TYPEOF(SELF)) OR
        ('GEOMETRY_SCHEMA.BEZIER_SURFACE' IN TYPEOF(SELF)) OR
        ('GEOMETRY_SCHEMA.B_SPLINE_SURFACE_WITH_KNOTS' IN TYPEOF(SELF));
END_ENTITY;

ENTITY b_spline_surface_with_knots
   SUBTYPE OF (b_spline_surface);
   u_multiplicities  : LIST [2:?] OF INTEGER;
   v_multiplicities  : LIST [2:?] OF INTEGER;
   u_knots           : LIST [2:?] OF parameter_value;
   v_knots           : LIST [2:?] OF parameter_value;
   knot_spec         : knot_type;
 DERIVE
   knot_u_upper      : INTEGER := SIZEOF(u_knots);
   knot_v_upper      : INTEGER := SIZEOF(v_knots);
 WHERE
    WR1: constraints_param_b_spline(SELF\\b_spline_surface.u_degree,
                   knot_u_upper, SELF\\b_spline_surface.u_upper,
                               u_multiplicities, u_knots);
    WR2: constraints_param_b_spline(SELF\\b_spline_surface.v_degree,
                   knot_v_upper, SELF\\b_spline_surface.v_upper,
                               v_multiplicities, v_knots);
    WR3: SIZEOF(u_multiplicities) = knot_u_upper;
    WR4: SIZEOF(v_multiplicities) = knot_v_upper;
END_ENTITY;

ENTITY uniform_surface
   SUBTYPE OF (b_spline_surface);
END_ENTITY;

ENTITY quasi_uniform_surface
   SUBTYPE OF (b_spline_surface);
END_ENTITY;

ENTITY bezier_surface
   SUBTYPE OF (b_spline_surface);
END_ENTITY;

ENTITY rational_b_spline_surface
   SUBTYPE OF (b_spline_surface);
   weights_data : LIST [2:?] OF LIST [2:?] OF REAL;
                                
 DERIVE
   weights       : ARRAY [0:u_upper] OF
                      ARRAY [0:v_upper] OF REAL
                 := make_array_of_array(weights_data,0,u_upper,0,v_upper);
 WHERE
   WR1: (SIZEOF(weights_data) =
                          SIZEOF(SELF\\b_spline_surface.control_points_list))
           AND (SIZEOF(weights_data[1]) =
                          SIZEOF(SELF\\b_spline_surface.control_points_list[1]));
   WR2: surface_weights_positive(SELF);
END_ENTITY;

ENTITY topological_representation_item
   SUPERTYPE OF (ONEOF(vertex, edge, face_bound, face, vertex_shell,
                   wire_shell, connected_edge_set, connected_face_set,
                    (loop ANDOR path)))
   SUBTYPE OF (representation_item);
END_ENTITY;

ENTITY vertex
   SUBTYPE OF (topological_representation_item);
END_ENTITY;

ENTITY vertex_point
    SUBTYPE OF(vertex,geometric_representation_item);
      vertex_geometry : point;
END_ENTITY;

ENTITY edge
   SUPERTYPE OF(ONEOF(edge_curve, oriented_edge))
   SUBTYPE OF (topological_representation_item);
   edge_start : vertex;
   edge_end   : vertex;
END_ENTITY;

ENTITY edge_curve
   SUBTYPE OF(edge,geometric_representation_item);
   edge_geometry : curve;
   same_sense    : BOOLEAN;
END_ENTITY;

ENTITY oriented_edge
   SUBTYPE OF (edge);
   edge_element : edge;
   orientation  : BOOLEAN;
 DERIVE
   SELF\\edge.edge_start : vertex := boolean_choose (SELF.orientation,
                                            SELF.edge_element.edge_start,
                                            SELF.edge_element.edge_end);
   SELF\\edge.edge_end   : vertex := boolean_choose (SELF.orientation,
                                            SELF.edge_element.edge_end,
                                            SELF.edge_element.edge_start);
 WHERE
   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_EDGE' IN TYPEOF (SELF.edge_element));
END_ENTITY;

ENTITY path
   SUPERTYPE OF (ONEOF(open_path, edge_loop, oriented_path))
   SUBTYPE OF (topological_representation_item);
   edge_list  : LIST [1:?] OF UNIQUE oriented_edge;
 WHERE
   WR1: path_head_to_tail(SELF);
END_ENTITY;

ENTITY oriented_path
   SUBTYPE OF (path);
   path_element : path;
   orientation  : BOOLEAN;
 DERIVE
   SELF\\path.edge_list : LIST [1:?] OF UNIQUE oriented_edge
                           := conditional_reverse(SELF.orientation,
                                         SELF.path_element.edge_list);
 WHERE
   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_PATH' IN TYPEOF (SELF.path_element));
END_ENTITY;

ENTITY open_path
   SUBTYPE OF (path);
 DERIVE
   ne : INTEGER := SIZEOF(SELF\\path.edge_list);
 WHERE
   WR1: (SELF\\path.edge_list[1].edge_element.edge_start) :<>:
                       (SELF\\path.edge_list[ne].edge_element.edge_end);
END_ENTITY;

ENTITY loop
   SUPERTYPE OF (ONEOF(vertex_loop, edge_loop, poly_loop))
   SUBTYPE OF (topological_representation_item);
END_ENTITY;

ENTITY vertex_loop
   SUBTYPE OF (loop);
   loop_vertex : vertex;
END_ENTITY;

ENTITY edge_loop
   SUBTYPE OF (loop,path);
 DERIVE
   ne : INTEGER := SIZEOF(SELF\\path.edge_list);
 WHERE
   WR1: (SELF\\path.edge_list[1].edge_start) :=:
        (SELF\\path.edge_list[ne].edge_end);
END_ENTITY;

ENTITY poly_loop
   SUBTYPE OF (loop,geometric_representation_item);
   polygon : LIST [3:?] OF UNIQUE cartesian_point;
END_ENTITY;

ENTITY face_bound
   SUBTYPE OF(topological_representation_item);
   bound       :  loop;
   orientation :  BOOLEAN;
END_ENTITY;

ENTITY face_outer_bound
    SUBTYPE OF (face_bound);
END_ENTITY;

ENTITY face
   SUPERTYPE OF(ONEOF(face_surface, subface, oriented_face))
   SUBTYPE OF (topological_representation_item);
   bounds : SET[1:?] OF face_bound;
 WHERE
   WR1: NOT (mixed_loop_type_set(list_to_set(list_face_loops(SELF))));
   WR2: SIZEOF(QUERY(temp <* bounds | 'TOPOLOGY_SCHEMA.FACE_OUTER_BOUND' IN
                                               TYPEOF(temp))) <= 1;
END_ENTITY;

ENTITY face_surface
   SUBTYPE OF(face,geometric_representation_item);
   face_geometry :  surface;
   same_sense    :  BOOLEAN;
END_ENTITY;

ENTITY oriented_face
   SUBTYPE OF (face);
   face_element : face;
   orientation  : BOOLEAN;
 DERIVE
   SELF\\face.bounds : SET[1:?] OF face_bound
          := conditional_reverse(SELF.orientation,SELF.face_element.bounds);
 WHERE
   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_FACE' IN TYPEOF (SELF.face_element));
END_ENTITY;

ENTITY subface
   SUBTYPE OF (face);
   parent_face   :  face;
 WHERE
   WR1: NOT (mixed_loop_type_set(list_to_set(list_face_loops(SELF)) +
              list_to_set(list_face_loops(parent_face))));
END_ENTITY;

ENTITY connected_face_set
   SUPERTYPE OF (ONEOF (closed_shell, open_shell))
   SUBTYPE OF (topological_representation_item);
   cfs_faces : SET [1:?] OF face;
END_ENTITY;

ENTITY vertex_shell
   SUBTYPE OF (topological_representation_item);
   vertex_shell_extent : vertex_loop;
END_ENTITY;

ENTITY wire_shell
   SUBTYPE OF (topological_representation_item);
   wire_shell_extent : SET [1:?] OF loop;
 WHERE
   WR1: NOT mixed_loop_type_set(wire_shell_extent);
END_ENTITY;

ENTITY open_shell
   SUBTYPE OF (connected_face_set);
END_ENTITY;

ENTITY oriented_open_shell
   SUBTYPE OF (open_shell);
   open_shell_element : open_shell;
   orientation        : BOOLEAN;
 DERIVE
   SELF\\connected_face_set.cfs_faces : SET [1:?] OF face
                                      := conditional_reverse(SELF.orientation,
                                           SELF.open_shell_element.cfs_faces);
 WHERE
   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_OPEN_SHELL' 
                IN TYPEOF (SELF.open_shell_element));
END_ENTITY;

ENTITY closed_shell
   SUBTYPE OF (connected_face_set);
END_ENTITY;

ENTITY oriented_closed_shell
   SUBTYPE OF (closed_shell);
   closed_shell_element : closed_shell;
   orientation          : BOOLEAN;
 DERIVE
   SELF\\connected_face_set.cfs_faces : SET [1:?] OF face
                                       := conditional_reverse(SELF.orientation,
                                          SELF.closed_shell_element.cfs_faces);
 WHERE
   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_CLOSED_SHELL' 
                IN TYPEOF (SELF.closed_shell_element));
END_ENTITY;

ENTITY connected_edge_set
   SUBTYPE OF (topological_representation_item);
   ces_edges : SET [1:?] OF edge;
END_ENTITY;

END_SCHEMA;

"
);

truck_stepio::parse_primitives!(ap04x, __parse_primitives);
truck_stepio::impl_curve!(ap04x, __impl_curve);
truck_stepio::impl_surface!(ap04x, __impl_surface);
