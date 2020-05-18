use std::vec::Vec;
use crate::errors::Error;
use crate::{Result, Shell, Solid};
use crate::shell::ShellCondition;

impl Solid {
    /// create the shell whose boundaries is boundary.
    /// # Panic
    /// All boundary must be non-empty, connected, and closed.
    #[inline(always)]
    pub fn new(boundaries: Vec<Shell>) -> Solid {
        for shell in &boundaries {
            if shell.is_empty() {
                panic!("{}", Error::EmptyShell);
            } else if !shell.is_connected() {
                panic!("{}", Error::NotConnectedShell);
            } else if shell.shell_condition() != ShellCondition::Closed {
                panic!("{}", Error::NotClosedShell);
            }
        }
        Solid { boundaries: boundaries }
    }
    
    /// create the shell whose boundaries is boundary.
    /// # Failure
    /// All boundary must be non-empty, connected, and closed.
    #[inline(always)]
    pub fn try_new(boundaries: Vec<Shell>) -> Result<Solid> {
        for shell in &boundaries {
            if shell.is_empty() {
                return Err(Error::EmptyShell);
            } else if !shell.is_connected() {
                return Err(Error::NotConnectedShell);
            } else if shell.shell_condition() != ShellCondition::Closed {
                return Err(Error::NotClosedShell);
            }
        }
        Ok(Solid { boundaries: boundaries })
    }
    
    /// create the shell whose boundaries is boundary.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether all boundary is non-empty, connected, and closed.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(boundaries: Vec<Shell>) -> Solid { Solid { boundaries: boundaries } }

    /// get the reference of boundary shells
    #[inline(always)]
    pub fn boundaries(&self) -> &Vec<Shell> { &self.boundaries }
}

#[test]
fn cube() {
    use crate::*;
    let v = Vertex::news(8);
    let edge = [
        Edge::new(v[0], v[1]), // 0
        Edge::new(v[1], v[2]), // 1
        Edge::new(v[2], v[3]), // 2
        Edge::new(v[3], v[0]), // 3
        Edge::new(v[0], v[4]), // 4
        Edge::new(v[1], v[5]), // 5
        Edge::new(v[2], v[6]), // 6
        Edge::new(v[3], v[7]), // 7
        Edge::new(v[4], v[5]), // 8
        Edge::new(v[5], v[6]), // 9
        Edge::new(v[6], v[7]), // 10
        Edge::new(v[7], v[4]), // 11
    ];

    let mut wire0 = Wire::new();
    wire0.push_back(edge[0]);
    wire0.push_back(edge[1]);
    wire0.push_back(edge[2]);
    wire0.push_back(edge[3]);
    let face0 = Face::new(wire0);

    let mut wire1 = Wire::new();
    wire1.push_back(edge[4]);
    wire1.push_back(edge[8]);
    wire1.push_back(edge[5].inverse());
    wire1.push_back(edge[0].inverse());
    let face1 = Face::new(wire1);

    let mut wire2 = Wire::new();
    wire2.push_back(edge[5]);
    wire2.push_back(edge[9]);
    wire2.push_back(edge[6].inverse());
    wire2.push_back(edge[1].inverse());
    let face2 = Face::new(wire2);

    let mut wire3 = Wire::new();
    wire3.push_back(edge[6]);
    wire3.push_back(edge[10]);
    wire3.push_back(edge[7].inverse());
    wire3.push_back(edge[2].inverse());
    let face3 = Face::new(wire3);
    
    let mut wire4 = Wire::new();
    wire4.push_back(edge[7]);
    wire4.push_back(edge[11]);
    wire4.push_back(edge[4].inverse());
    wire4.push_back(edge[3].inverse());
    let face4 = Face::new(wire4);

    let mut wire5 = Wire::new();
    wire5.push_back(edge[11].inverse());
    wire5.push_back(edge[10].inverse());
    wire5.push_back(edge[9].inverse());
    wire5.push_back(edge[8].inverse());
    let face5 = Face::new(wire5);

    let mut shell = Shell::new();
     
    shell.push(face0);
    shell.push(face5);
    
    assert!(!shell.is_connected());
    
    shell.push(face1);
    
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    
    shell.push(face2);
    shell.push(face3);
    shell.push(face4);

    Solid::new(vec![shell]);
}

