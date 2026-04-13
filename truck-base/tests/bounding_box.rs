use truck_base::{bounding_box::*, cgmath64::*};

#[test]
fn test_bbox_non_crossing() {
    let a = BoundingBox::<Point2>::from_iter(&[
        Point2::new(2.0, 1.0),
        Point2::new(3.9999999, 4.999999),
    ]);
    let b = BoundingBox::<Point2>::from_iter(&[Point2::new(0.0, 5.0), Point2::new(3.0, 7.0)]);
    let intersection = a ^ b;
    assert!(intersection.is_empty());
}

#[test]
fn test_bbox_touch_edge() {
    let a = BoundingBox::<Point2>::from_iter(&[Point2::new(2.0, 1.0), Point2::new(3.0, 4.0)]);
    let b = BoundingBox::<Point2>::from_iter(&[Point2::new(3.0, 0.0), Point2::new(4.0, 3.0)]);
    let intersection = a ^ b;
    assert!(!intersection.is_empty());
    assert_eq!(intersection.min(), Point2::new(3.0, 1.0));
    assert_eq!(intersection.max(), Point2::new(3.0, 3.0));
}

#[test]
fn test_bbox_touch_corner() {
    let a = BoundingBox::<Point2>::from_iter(&[Point2::new(2.0, 1.0), Point2::new(3.0, 4.0)]);
    let b = BoundingBox::<Point2>::from_iter(&[Point2::new(3.0, 4.0), Point2::new(4.0, 8.0)]);
    let intersection = a ^ b;
    assert!(!intersection.is_empty());
    assert_eq!(intersection.min(), Point2::new(3.0, 4.0));
    assert_eq!(intersection.max(), Point2::new(3.0, 4.0));
}
