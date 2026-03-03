use super::*;

#[test]
fn extract_boundaries0() {
    let faces = Faces::from_iter(&[[0, 1, 4].as_ref(), &[1, 2, 3, 4], &[0, 4, 5, 6, 7]]);
    let boundaries = faces.extract_boundaries();
    assert_eq!(boundaries.len(), 1);
    for a in boundaries[0].windows(2) {
        assert!(a[0] + 1 == a[1] || (a[0], a[1]) == (7, 0));
    }
}

#[test]
fn extract_boundaries1() {
    let faces = Faces::from_iter(&[
        [0, 1, 5, 4].as_ref(),
        &[1, 2, 6, 5],
        &[2, 3, 7, 6],
        &[3, 0, 4, 7],
    ]);
    let boundaries = faces.extract_boundaries();
    assert_eq!(boundaries.len(), 2);
    println!("{boundaries:?}");
    for a in boundaries[0].windows(2) {
        if a[0] < 4 {
            assert!(a[0] + 1 == a[1] || a[1] + 3 == a[0]);
        } else {
            assert!(a[0] == a[1] + 1 || a[1] == a[0] + 3);
        }
    }
    for a in boundaries[1].windows(2) {
        if a[0] < 4 {
            assert!(a[0] + 1 == a[1] || a[1] + 3 == a[0]);
        } else {
            assert!(a[0] == a[1] + 1 || a[1] == a[0] + 3);
        }
    }
    assert!(boundaries[0][0] > 3 || boundaries[1][0] > 3);
    assert!(boundaries[0][0] < 4 || boundaries[1][0] < 4);
}

#[test]
fn shell_condition() {
    let faces = Faces::from_iter(&[[0, 1, 4].as_ref(), &[1, 2, 3, 0], &[0, 4, 5, 6, 1]]);
    assert_eq!(faces.shell_condition(), ShellCondition::Irregular);
    let faces = Faces::from_iter(&[
        [0, 1, 5, 4].as_ref(),
        &[1, 2, 6, 5],
        &[6, 7, 3, 2],
        &[3, 0, 4, 7],
    ]);
    assert_eq!(faces.shell_condition(), ShellCondition::Regular);
    let faces = Faces::from_iter(&[
        [0, 1, 5, 4].as_ref(),
        &[1, 2, 6, 5],
        &[2, 3, 7, 6],
        &[3, 0, 4, 7],
    ]);
    assert_eq!(faces.shell_condition(), ShellCondition::Oriented);
    let faces = Faces::from_iter(&[
        [0, 1, 5, 4].as_ref(),
        &[1, 2, 6, 5],
        &[2, 3, 7, 6],
        &[3, 0, 4, 7],
        &[1, 0, 3],
        &[1, 3, 2],
        &[5, 6, 7],
        &[4, 5, 7],
    ]);
    assert_eq!(faces.shell_condition(), ShellCondition::Closed);
}
