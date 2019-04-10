use crate::mathvec::{Scalar, Vec2d};
use crate::particles::{Particle};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Span {
    plus_corner : Vec2d,
    minus_corner : Vec2d,
}

impl Span {
    fn empty() -> Span {
        Span { plus_corner : Vec2d::zero(), minus_corner : Vec2d::zero() }
    }

    pub fn new(plus_corner : Vec2d, minus_corner : Vec2d) -> Span {
        debug_assert!( 
            (plus_corner == Vec2d::zero() && minus_corner == Vec2d::zero()) ||
            (plus_corner.x > minus_corner.x && plus_corner.y > minus_corner.y)
        );
        Span { plus_corner, minus_corner }
    }
    pub fn midpoint(self) -> Vec2d {
        (self.minus_corner + self.plus_corner)/2.0
    }
    pub fn subspans(self) -> [Span ; 4] {
        let mut retval = [Span::empty() ; 4] ;
        let mid = self.midpoint();
        let x_min = self.minus_corner.x;
        let x_mid = mid.x;
        let x_max = self.plus_corner.x;

        let y_min = self.minus_corner.y;
        let y_mid = mid.y;
        let y_max = self.plus_corner.y;
        
        let nn = Span::new(Vec2d::new(x_mid, y_mid), Vec2d::new(x_min, y_min));
        retval[0] = nn;

        let np = Span::new(Vec2d::new(x_mid, y_max), Vec2d::new(x_min, y_mid));
        retval[1] = np;

        let pn = Span::new(Vec2d::new(x_max, y_mid), Vec2d::new(x_mid, y_min));
        retval[2] = pn;

        let pp = Span::new(Vec2d::new(x_max, y_max), Vec2d::new(x_mid, y_mid));
        retval[3] = pp;

        retval
    }
    pub fn subspan_idx_for(self, part : Particle) -> (usize, usize) {
        let mid = self.midpoint();
        let x_idx = if { part.pos.x < mid.x } { 0 } else { 1 };
        let y_idx = if { part.pos.y < mid.y } { 0 } else { 1 };
        (x_idx, y_idx)
    }

    pub fn volume(self) -> Scalar {
        let shifted_spans = self.plus_corner - self.minus_corner;
        shifted_spans.x * shifted_spans.y 
    }

}


type NodeIndex = u32;

#[derive(Copy, Clone)]
pub struct TreeNode {
    span : Span,
    data : NodeType,
} 

#[derive(Copy, Clone)]
enum NodeType {
    Empty,
    Leaf(Particle),
    Branch {
        masspos : Vec2d,
        mass : Scalar,
        subnodes : [NodeIndex ; 4],
    }
}

impl TreeNode {
    pub fn new(span : Span) -> TreeNode {
        TreeNode {
            span,
            data : NodeType::Empty,
        }
    }
    pub fn masspos(&self) -> Vec2d {
        match self.data {
            NodeType::Empty => Vec2d::zero(),
            NodeType::Leaf(p) => p.mass * p.pos,
            NodeType::Branch{masspos, ..} => masspos
        }
    }
    pub fn mass(&self) -> Scalar {
        match self.data {
            NodeType::Empty => 0.0,
            NodeType::Leaf(p) => p.mass,
            NodeType::Branch{mass, ..} => mass
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct MassTreeBuilder {
    node_capacity : usize, 
    span : Span,
}

impl MassTreeBuilder {

    pub fn new() -> Self {
        MassTreeBuilder::default()
    }

    pub fn with_span(self, span : Span) -> Self {
        MassTreeBuilder {
            span : span, 
            ..self
        }
    }

    pub fn with_particle_capacity(self, cap : usize) -> Self {
        let node_cap = ( (cap as f64) * (cap as f64).log(4.0) ) as usize + 1; 
        self.with_node_capacity(node_cap)
    }

    pub fn with_node_capacity(self, cap : usize) -> Self {
        MassTreeBuilder {
            node_capacity : cap, 
            ..self
        }
    }

    pub fn build(self) -> MassTree {
        let mut nodes = Vec::with_capacity(self.node_capacity);
        nodes.push(TreeNode {
            span : self.span,
            data : NodeType::Empty,
        });
        MassTree {
            nodes
        }
    }
}

pub struct MassTree {
    nodes : Vec<TreeNode>,
}

impl Default for MassTree {
    fn default() -> Self {
        MassTreeBuilder::default().build()
    }
}

impl MassTree {
    pub fn builder() -> MassTreeBuilder {
        MassTreeBuilder::new()
    }
    fn get_node(&self, idx : NodeIndex) -> &TreeNode {
        &self.nodes[idx as usize]
    }
    fn get_node_mut(&mut self, idx : NodeIndex) -> &mut TreeNode {
        &mut self.nodes[idx as usize]
    }
    fn leaf_to_branch(&mut self, parent_idx : NodeIndex) {
        let (span, part) = match self.get_node(parent_idx) {
            TreeNode{span, data : NodeType::Leaf(p)} => (*span, *p),
            _ => {
                return;
            }
        };

        let start_len = self.nodes.len();
        let mut new_children = [0 ; 4];

        let spans = span.subspans();
        for x_idx in 0..2 {
            for y_idx in 0..2 {
                let offset = (x_idx * 2 + y_idx) as usize;
                self.nodes.push(TreeNode::new(spans[offset]));
                debug_assert_eq!(self.nodes.len(), start_len + offset+1);
                debug_assert_eq!( ((spans[offset].plus_corner.x - span.plus_corner.x).abs() < Vec2d::EPSILON), x_idx == 1);
                debug_assert_eq!( ((spans[offset].plus_corner.y - span.plus_corner.y).abs() < Vec2d::EPSILON), y_idx == 1);
                new_children[offset] = (start_len + offset) as NodeIndex;
            }
        }

        let new_data = {
            let (x_idx, y_idx) = span.subspan_idx_for(part);
            self.get_node_mut(new_children[x_idx * 2 + y_idx]).data = NodeType::Leaf(part);
            
            NodeType::Branch {
                mass : part.mass,
                masspos : part.mass * part.pos,
                subnodes : new_children
            }
        };

        self.get_node_mut(parent_idx).data = new_data;
    }
    pub fn add_particle(&mut self, particle : Particle) {
        debug_assert!(self.nodes.get(0).map_or(false, |n| n.span.volume() > 0.0001));
        let mut cur_idx : NodeIndex = 0;
        loop {
            let cur_node = self.get_node_mut(cur_idx);
            match cur_node.data {
                NodeType::Empty => {
                    cur_node.data = NodeType::Leaf(particle);
                    return;
                },
                NodeType::Leaf(p) => {
                    if p == particle {
                        return;
                    }
                    self.leaf_to_branch(cur_idx);
                },
                NodeType::Branch{subnodes, ref mut mass, ref mut masspos} => {
                    *mass += particle.mass;
                    *masspos += particle.mass * particle.pos;
                    let (x_idx, y_idx) = cur_node.span.subspan_idx_for(particle);
                    cur_idx = subnodes[x_idx * 2 + y_idx];
                }
            }
        }
    }
    pub fn calculate_forces(&self, arg : Particle, G : Scalar) -> Vec2d {
        let mut retval = Vec2d::zero();
        let mut to_calc : Vec<NodeIndex> = Vec::with_capacity(4);
        to_calc.push(0);
        while let Some(cur_idx) = to_calc.pop() {
            let cur_node = self.get_node(cur_idx);
            match cur_node {
                TreeNode{span, data : NodeType::Branch{ masspos, mass, subnodes }} => {
                    let pt = (*masspos)/(*mass);
                    let diff = pt - arg.pos;
                    let d = diff.mag_squared();
                    let diffw = span.plus_corner.x - span.minus_corner.x;
                    if diffw * diffw >= d * 0.8 {
                        to_calc.extend_from_slice(subnodes);
                    }
                    else {
                        let force = G * arg.mass * mass/d; 
                        let d = d.sqrt();
                        if d > 0.5 {
                            retval += diff * force /d;
                        }
                    }
                },
                TreeNode{data : NodeType::Leaf(part), ..} => {
                    let diff = part.pos - arg.pos; 
                    let d = diff.mag_squared();
                    let force = G * arg.mass * part.mass/d;
                    let d = d.sqrt();
                    if d > 0.5 {
                        retval += diff * force/d; 
                    }
                },
                _ => {},
            }
        }
        retval
    }
}