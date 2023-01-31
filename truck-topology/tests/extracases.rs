use truck_topology::{shell::ShellCondition, *};

// Cases where truck version <= 0.4.0 fails
#[test]
fn singular_vertices_in_certain_boundaries() {
    let v = Vertex::news([(); 12]);
    let edge = [
        Edge::new(&v[0], &v[1], ()),
        Edge::new(&v[1], &v[2], ()),
        Edge::new(&v[0], &v[7], ()),
        Edge::new(&v[1], &v[8], ()),
        Edge::new(&v[2], &v[9], ()),
        Edge::new(&v[3], &v[4], ()),
        Edge::new(&v[4], &v[3], ()),
        Edge::new(&v[5], &v[6], ()),
        Edge::new(&v[6], &v[5], ()),
        Edge::new(&v[7], &v[8], ()),
        Edge::new(&v[8], &v[9], ()),
        Edge::new(&v[7], &v[10], ()),
        Edge::new(&v[9], &v[11], ()),
        Edge::new(&v[10], &v[11], ()),
    ];
    let shell: Shell<(), (), ()> = vec![
        Face::new(
            vec![
                vec![
                    edge[9].inverse(),
                    edge[2].inverse(),
                    edge[0].clone(),
                    edge[3].clone(),
                ]
                .into(),
                vec![edge[5].clone(), edge[6].clone()].into(),
            ],
            (),
        ),
        Face::new(
            vec![
                vec![
                    edge[3].inverse(),
                    edge[1].clone(),
                    edge[4].clone(),
                    edge[10].inverse(),
                ]
                .into(),
                vec![edge[7].clone(), edge[8].clone()].into(),
            ],
            (),
        ),
        Face::new(
            vec![vec![
                edge[10].clone(),
                edge[12].clone(),
                edge[13].inverse(),
                edge[11].inverse(),
                edge[9].clone(),
            ]
            .into()],
            (),
        ),
    ]
    .into();
    assert!(matches!(shell.shell_condition(), ShellCondition::Oriented));
    assert!(shell.singular_vertices().is_empty());
}
