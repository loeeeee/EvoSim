use std::f32::consts::PI;
use std::fmt::{self, Debug};

use bevy::prelude::*;
use rand::prelude::*;
use serde::{Serialize, Deserialize};

use crate::brain::neuron::GenericNN;
use crate::consts::*;

use super::blob_builder::BlobBuilder;
use super::block::PhysiBlockBundle;

/// Generate Blob according to Genotype
/// Wrapper around BlobBuilder
pub struct GenoBlobBuilder<'a> {
    builder: BlobBuilder<'a>,
}

impl<'a> GenoBlobBuilder<'a> {
    pub fn from_commands(commands: Commands<'a, 'a>, nnvec: &'a mut Vec<GenericNN>) -> Self {
        Self {
            builder: BlobBuilder::from_commands(commands, nnvec),
        }
    }

    /// generate blob according to its genotype
    pub fn build(&mut self, geno: &mut BlobGeno, center: [f32; 2]) {
        // Lambda function to use in child extraction
        fn lambda(node: &mut Option<GenericGenoNode>) -> Option<&mut GenoNode> {
            node.as_mut().and_then(|node| match node {
                GenericGenoNode::Parent => None,
                GenericGenoNode::Child(child) => Some(child),
            })
        }

        fn build_node(builder: &mut BlobBuilder, tree: &mut QuadTree<GenericGenoNode>, index: usize) {
            if let Some(Some(_)) = tree.nodes.get_mut(index) {
                let children = tree.children(index);
                // let (top_child, bottom_child, left_child, right_child) = (
                //     tree.nodes.get(children[0]).and_then(lambda),
                //     tree.nodes.get(children[1]).and_then(lambda),
                //     tree.nodes.get(children[2]).and_then(lambda),
                //     tree.nodes.get(children[3]).and_then(lambda),
                // );

                // top
                if let Some(mut node) = tree.nodes.get_mut(children[0]).and_then(lambda) {
                    let nn_id = builder.add_to_top(
                        node.size[0],
                        node.size[1],
                        None,
                        Some(node.joint_limits),
                        (),
                    );

                    // don't overwrite nn_id if it is not None
                    // which means they have already had bounded NN
                    if node.nn_id.is_none() {
                        node.nn_id = nn_id
                    }
                    
                    build_node(builder, tree, children[0]);
                    builder.bottom();
                }

                // bottom
                if let Some(mut node) = tree.nodes.get_mut(children[1]).and_then(lambda) {
                    let nn_id = builder.add_to_bottom(
                        node.size[0],
                        node.size[1],
                        None,
                        Some(node.joint_limits),
                        (),
                    );

                    if node.nn_id.is_none() {
                        node.nn_id = nn_id
                    }

                    build_node(builder, tree, children[1]);
                    builder.top();
                }

                // left
                if let Some(node) = tree.nodes.get_mut(children[2]).and_then(lambda) {
                    let nn_id = builder.add_to_left(
                        node.size[0],
                        node.size[1],
                        None,
                        Some(node.joint_limits),
                        (),
                    );

                    if node.nn_id.is_none() {
                        node.nn_id = nn_id
                    }

                    build_node(builder, tree, children[2]);
                    builder.right();
                }

                // right
                if let Some(node) = tree.nodes.get_mut(children[3]).and_then(lambda) {
                    let nn_id = builder.add_to_right(
                        node.size[0],
                        node.size[1],
                        None,
                        Some(node.joint_limits),
                        (),
                    );

                    if node.nn_id.is_none() {
                        node.nn_id = nn_id
                    }

                    build_node(builder, tree, children[3]);
                    builder.left();
                }
            }
        }

        // save geno to blob
        self.builder.update_geno(geno.clone());

        // create first
        let builder = &mut self.builder;
        geno.assign_nn_id_to_root(
            builder.create_first(
            geno.get_first()
                .unwrap()
                .to_bundle(center)
                .with_color(Color::BLUE),
            (),).unwrap()
        );

        // start recursion
        build_node(&mut self.builder, &mut geno.vec_tree, 0);

        // reset builder
        self.builder.clean();
    }
}

/// The Geno for morphyology of the blob.
/// The Geno is a QuadTree (it can be represented as TernaryTree as well).
/// index 0,1,2,3 means up,down,left,right (one of them can be ParentIndicator)
#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct BlobGeno {
    pub vec_tree: QuadTree<GenericGenoNode>,
}

impl Default for BlobGeno {
    fn default() -> Self {
        Self {
            vec_tree: QuadTree::<GenericGenoNode>::new(GENO_MAX_DEPTH),
        }
    }
}

impl BlobGeno {
    // TODO: Clean the code. Ugly long function
    /// generate a random GenoType that don't have conflict limbs
    pub fn new_rand() -> BlobGeno {
        // prevent tree-structural block conflict
        let mut occupied_region = Vec::<[f32; 4]>::new();

        fn is_overlapped(
            center: [f32; 2],
            size: [f32; 2],
            occupied_region: &mut Vec<[f32; 4]>,
        ) -> bool {
            let x_min = center[0] - size[0];
            let x_max = center[0] + size[0];
            let y_min = center[1] - size[1];
            let y_max = center[1] + size[1];

            for region in occupied_region.iter() {
                let x_overlap = x_min <= region[1] && x_max >= region[0];
                let y_overlap = y_min <= region[3] && y_max >= region[2];
                if x_overlap && y_overlap {
                    occupied_region.push([x_min, x_max, y_min, y_max]);
                    return true;
                }
            }
            occupied_region.push([x_min, x_max, y_min, y_max]);
            return false;
        }

        /// function to acquire a new rand node
        fn rand_nodes(
            parent: &GenoNode,
            direction: usize,
            occupied_region: &mut Vec<[f32; 4]>,
        ) -> Option<GenericGenoNode> {
            let mut rng = thread_rng();

            let parent_size = parent.size;
            let parent_center = parent.center;

            // set limitation
            // limitation can only avoid block conflict
            // it can not avoid conflict caused by tree structure
            let dx_dy_limits_top_bottom =
                [parent_size[0], DEFAULT_BLOCK_SIZE[0] * RAND_SIZE_SCALER[1]];
            let dx_dy_limits_left_right =
                [DEFAULT_BLOCK_SIZE[0] * RAND_SIZE_SCALER[1], parent_size[1]];

            if rng.gen_bool(RAND_NODE_NOT_NONE) {
                let joint_limits = [rng.gen_range(-PI * 0.9..0.0), rng.gen_range(0.0..PI * 0.9)];
                let mut size = [
                    rng.gen_range(
                        RAND_SIZE_SCALER[0] * DEFAULT_BLOCK_SIZE[0]..dx_dy_limits_top_bottom[0],
                    ),
                    rng.gen_range(
                        RAND_SIZE_SCALER[0] * DEFAULT_BLOCK_SIZE[1]..dx_dy_limits_top_bottom[1],
                    ),
                ];
                if direction == 2 || direction == 3 {
                    size = [
                        rng.gen_range(
                            RAND_SIZE_SCALER[0] * DEFAULT_BLOCK_SIZE[0]..dx_dy_limits_left_right[0],
                        ),
                        rng.gen_range(
                            RAND_SIZE_SCALER[0] * DEFAULT_BLOCK_SIZE[1]..dx_dy_limits_left_right[1],
                        ),
                    ];
                }

                // center
                let mut center = [
                    parent_center[0],
                    parent_center[1] + parent_size[1] + size[1],
                ];
                if direction == 1 {
                    center = [
                        parent_center[0],
                        parent_center[1] - parent_size[1] - size[1],
                    ];
                } else if direction == 2 {
                    center = [
                        parent_center[0] - parent_size[0] - size[0],
                        parent_center[1],
                    ];
                } else if direction == 3 {
                    center = [
                        parent_center[0] + parent_size[0] + size[0],
                        parent_center[1],
                    ]
                }
                if is_overlapped(center, size, occupied_region) {
                    return None;
                } else {
                    return Some(GenericGenoNode::Child(GenoNode {
                        joint_limits,
                        size,
                        center,
                        nn_id: None
                    }));
                }
            };
            return None;
        }

        /// recursive function
        fn build(
            tree: &mut QuadTree<GenericGenoNode>,
            index: usize,
            occupied_region: &mut Vec<[f32; 4]>,
        ) {
            let mut rng = thread_rng();

            let children = tree.children(index);

            // index and children index should in range
            if tree.nodes.get(children[3]).is_none() {
                return;
            }

            // random init four nodes, avoid self-conflict
            if let Some(GenericGenoNode::Child(node)) = tree.nodes[index].clone() {
                for (i, &child) in children.iter().enumerate() {
                    tree.nodes[child] = rand_nodes(&node, i, occupied_region)
                }

                // one parent indicator
                let parent_idx = *children.choose(&mut rng).unwrap();
                tree.nodes[parent_idx] = Some(GenericGenoNode::Parent);

                // keep recursion
                for &i in children.iter() {
                    if i != parent_idx {
                        build(tree, i, occupied_region);
                    }
                }
            }
        }

        // init tree
        let mut bg = BlobGeno::default();
        // root node
        bg.vec_tree.nodes[0] = Some(GenericGenoNode::Child(GenoNode::default()));
        build(&mut bg.vec_tree, 0, &mut occupied_region);
        bg
    }

    pub fn get_first(&self) -> Option<&GenoNode> {
        self.vec_tree.nodes[0].as_ref().and_then(|node| match node {
            GenericGenoNode::Parent => None,
            GenericGenoNode::Child(child) => Some(child),
        })
    }

    /// The genotype is valid or not.
    /// 
    /// Not valid means self-conflit limbs
    pub fn is_valid(&self) -> bool {

        fn is_overlapped(
            center: [f32; 2],
            size: [f32; 2],
            occupied_region: &mut Vec<[f32; 4]>,
        ) -> bool {
            let x_min = center[0] - size[0];
            let x_max = center[0] + size[0];
            let y_min = center[1] - size[1];
            let y_max = center[1] + size[1];

            // println!("{},{},{},{}",x_min,x_max,y_min,y_max);

            for region in occupied_region.iter() {
                let x_overlap = x_min < region[1] - POSITION_EPSILON && x_max - POSITION_EPSILON > region[0];
                let y_overlap = y_min < region[3] - POSITION_EPSILON && y_max - POSITION_EPSILON > region[2];
                if x_overlap && y_overlap {
                    occupied_region.push([x_min, x_max, y_min, y_max]);
                    return true;
                }
            }
            occupied_region.push([x_min, x_max, y_min, y_max]);
            return false;
        }

        /// recursively add to `occupied_region`
        fn check (
            tree: &QuadTree<GenericGenoNode>,
            mut occupied_region: &mut Vec<[f32; 4]>,
            idx: usize
        ) -> bool {
            // println!("is_valid checking {}", idx);
            // println!("occupied_region {:?}", occupied_region);
            if let Some(Some(GenericGenoNode::Child(cur))) = tree.nodes.get(idx) {
                if !is_overlapped(cur.center, cur.size, &mut occupied_region) {
                    tree.children(idx).iter().all(|&i| check(tree, occupied_region, i))
                } else {
                    // println!("not valid {}", idx);
                    false
                }
            } else {
                true
            }
        }

        let mut occupied_region: Vec<[f32; 4]> = Vec::new();
        check(&self.vec_tree, &mut occupied_region, 0)

    }


    /// all nodes don't have child, used for mutate to lose limb
    /// 
    /// can not return root, can not return parent indicator
    pub fn leaf_nodes(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for i in 1..self.vec_tree.nodes.len() {
            if let Some(GenericGenoNode::Parent) = self.vec_tree.nodes[i] {
                continue; // Skip if the node is of type GenericGenoNode::Parent
            }
            if self.vec_tree.nodes[i].is_some() && self.vec_tree.children(i).iter().all(
                |&child_idx| 
                child_idx >= self.vec_tree.nodes.len() || 
                self.vec_tree.nodes[child_idx].is_none() || 
                matches!(
                    self.vec_tree.nodes[child_idx], 
                    Some(GenericGenoNode::Parent)
                )
            ) {
                result.push(i);
            }
        }
        result
    }

    pub fn assign_nn_id_to_root(&mut self, id: usize) {
        if let Some(Some(GenericGenoNode::Child(node))) = self.vec_tree.nodes.get_mut(0) {
            if node.nn_id.is_none() {
                node.nn_id = Some(id);
            }
        } else {
            panic!()
        }
    }
}

/// GenericGenoNode is the Node in the BlobGeno QuadTree.
/// Representing morphyology of each block inside blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericGenoNode {
    /// parent indicator
    Parent,
    Child(GenoNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenoNode {
    pub joint_limits: [f32; 2],
    pub size: [f32; 2],
    pub center: [f32; 2],
    pub nn_id: Option<usize>,
}

impl Default for GenoNode {
    fn default() -> Self {
        Self {
            joint_limits: [-PI, PI],
            size: DEFAULT_BLOCK_SIZE,
            center: [0.0, 0.0],
            nn_id: None
        }
    }
}

impl GenoNode {
    pub fn from_nn_id(nn_id: usize) -> Self {
        Self {
            joint_limits: [-PI, PI],
            size: DEFAULT_BLOCK_SIZE,
            center: [0.0, 0.0],
            nn_id: Some(nn_id)
        }
    }
    /// generate `PhysiBlockBundle` from GenoNode
    fn to_bundle(&self, center: [f32; 2]) -> PhysiBlockBundle {
        PhysiBlockBundle::from_xy_dx_dy(center[0], center[1], self.size[0], self.size[1])
    }
}

/// QuadTree, Helper struct
#[derive(Clone, Serialize, Deserialize)]
pub struct QuadTree<T> {
    pub nodes: Vec<Option<T>>,
    pub max_depth: u32,
}

impl<T> QuadTree<T> {
    pub fn new(max_depth: u32) -> Self {
        let capacity = usize::pow(4, max_depth)+1;
        let nodes = (0..capacity).map(|_| None).collect();
        Self { max_depth, nodes }
    }

    pub fn parent(&self, index: usize) -> Option<usize> {
        if index == 0 {
            None
        } else {
            Some((index - 1) / 4)
        }
    }

    pub fn children(&self, index: usize) -> [usize; 4] {
        let base = 4 * index;
        [base + 1, base + 2, base + 3, base + 4]
    }

    pub fn depth(&self, index: usize) -> u32 {
        (index as f64).log(4.0).floor() as u32
    }

    pub fn is_leaf(&self, index: usize) -> bool {
        let children_indices = self.children(index);
        children_indices.iter().all(|&child_index| {
            child_index >= self.nodes.len() || self.nodes[child_index].is_none()
        })
    }

    pub fn clean_subtree(&mut self, index: usize) {
        self.nodes[index] = None;
        let child_indices = self.children(index);

        // For each child, if the child exists, clean it recursively
        for &child_index in &child_indices {
            if child_index < self.nodes.len() && self.nodes[child_index].is_some() {
                self.clean_subtree(child_index);
            }
        }
    }

    pub fn clean_subtree_without_self(&mut self, index: usize) {
        let child_indices = self.children(index);

        // For each child, if the child exists, clean it recursively
        for &child_index in &child_indices {
            if child_index < self.nodes.len() && self.nodes[child_index].is_some() {
                self.clean_subtree(child_index);
            }
        }
    }

    /// all nodes have at least one `none` child, using for mutate to gain limb
    pub fn branch_nodes(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for i in 0..self.nodes.len() {
            if self.nodes[i].is_some() 
                && self.depth(i) < self.max_depth - 1 // Ensure the node is not at the last layer
                && self.children(i).iter().any(
                    |&child_idx| 
                    child_idx >= self.nodes.len() || self.nodes[child_idx].is_none()
                ) {
                result.push(i);
            }
        }
        result
    }
}

impl<T: Debug> Debug for QuadTree<T> {
    /// tree structure debug info
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_node<T: Debug>(
            tree: &QuadTree<T>,
            index: usize,
            indent: &str,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            match tree.nodes.get(index) {
                None | Some(None) => Ok(()), // skip empty nodes
                Some(Some(node)) => {
                    writeln!(f, "{}- Node {}: {:?}", indent, index, node)?;
                    let children = tree.children(index);
                    for &child_index in &children {
                        print_node(tree, child_index, &format!("{}  ", indent), f)?;
                    }
                    Ok(())
                }
            }
        }

        writeln!(f, "QuadTree {{")?;
        print_node(self, 0, "  ", f)?;
        writeln!(f, "}}")
    }
}


#[cfg(test)]
mod builder_validation_test {
    use super::*;

    #[test]
    fn test_geno_builder_validation() {
        for _ in 0..100 {
            let geno = BlobGeno::new_rand();
            assert!(geno.is_valid());
        }
    }
}