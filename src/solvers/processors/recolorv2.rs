use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    rc::Rc,
};

use derivative::Derivative;
use nabo::KDTree;

use crate::{
    block::{BlockId, Rect},
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Cost, Move, MoveType},
    painting::Painting,
    solvers::Processor,
};

pub struct Recolorv2;

fn recolor(
    canvas: &mut Canvas,
    initial_moves: &[AppliedMove],
    planned_creation_colors: HashMap<BlockId, Color>,
) -> Vec<AppliedMove> {
    // step 4  Rebuild the coloring history.
    // step 4.1 make a color free move history
    let color_free_moves: Vec<_> = initial_moves
        .iter()
        .filter(|&mov| match mov.mov {
            Move::Color(..) => false,
            _ => true,
        })
        .collect();

    // step 4.2 go back to the initial canvas
    for am in initial_moves.iter().rev() {
        am.clone().undo(canvas);
    }

    // step 4.3, build the new moves by inserting coloring as color free moves are applied
    let mut new_moves: Vec<AppliedMove> = vec![];
    for free_move in color_free_moves {
        let applied_move = free_move.mov.clone().apply(canvas).unwrap();
        new_moves.push(applied_move.clone());
        for created_block in applied_move.created_blocks() {
            if let Some(color) = planned_creation_colors.get(&created_block) {
                new_moves.push(Move::Color(created_block, *color).apply(canvas).unwrap());
            }
        }
    }
    new_moves
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
struct ColorCluster {
    #[derivative(Debug = "ignore")]
    pixels: Vec<Color>,
    best_color: Color,
    /// the cost of coloring this cluster
    coloring_cost: Cost,
    /// the error cost
    error_cost: Cost,
    /// a reference cost
    reference_cost: Cost,
}

impl ColorCluster {
    fn improvement(&self) -> i64 {
        return self.reference_cost.0 as i64 - self.cost().0 as i64;
    }

    fn cost(&self) -> Cost {
        self.coloring_cost + self.error_cost
    }

    fn new(region: Rect, pixels: Vec<Color>, canvas: &Canvas, painting: &Painting) -> Self {
        const EPS: f32 = 0.2;
        const MAX_ITERATIONS: u32 = 1000;
        let color_options = [
            Color::gmedian(&pixels, EPS, MAX_ITERATIONS),
            Color::pmedian(&pixels, EPS, MAX_ITERATIONS),
        ];

        let best_color = *color_options
            .iter()
            .min_by_key(|c| painting.calculate_score_rect(&region, **c) as i64)
            .unwrap();

        let coloring_cost = canvas.compute_cost(MoveType::Color, region.area());
        let error_cost = Cost::from_block_cost(painting.calculate_score_rect(&region, best_color));
        ColorCluster {
            pixels,
            best_color,
            coloring_cost,
            error_cost,
            reference_cost: coloring_cost + error_cost,
        }
    }
}

#[derive(Debug)]
struct ColorTree {
    root: BlockId,
    region: Option<Rect>,
    children: Vec<Rc<ColorTree>>,
    /// A group of pixel this block is responsible for coloring
    cluster: Option<ColorCluster>,
}

impl ColorTree {
    fn find_clusters(root: &Rc<ColorTree>, res: &mut Vec<Rc<ColorTree>>) {
        for child in &root.children {
            ColorTree::find_clusters(child, res);
        }
        if let Some(_) = root.cluster {
            res.push(root.clone())
        }
    }
}

#[derive(Debug)]
struct ColorForest {
    trees: HashMap<BlockId, Rc<ColorTree>>,
    roots: Vec<Rc<ColorTree>>,
}

impl ColorForest {
    fn get_colors(&self) -> HashMap<BlockId, Color> {
        self.trees
            .iter()
            .filter_map(|(block_id, tree)| {
                if let Some(ref cluster) = tree.as_ref().cluster {
                    Some((block_id.clone(), cluster.best_color))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// The move history is used to recover the span of all nodes
fn build_initial_forest(
    move_history: &Vec<AppliedMove>,
    canvas: &Canvas,
    painting: &Painting,
) -> ColorForest {
    let mut color_trees: HashMap<BlockId, Rc<ColorTree>> =
        HashMap::with_capacity(canvas.blocks_count());

    for block in canvas.blocks_iter() {
        let pixels = painting.get_pixels(&block.r);
        let tree = Rc::new(ColorTree {
            root: block.id.clone(),
            region: Some(block.r),
            children: vec![],
            cluster: Some(ColorCluster::new(block.r, pixels, canvas, painting)),
        });
        assert!(!color_trees.contains_key(&block.id));
        color_trees.insert(block.id.clone(), tree.clone());

        // rebuild the parent tree
        let mut tree = tree;
        let mut parents: Vec<BlockId> = block.id.rev_parents(false).collect();
        parents.reverse();
        for parent in parents {
            // find the entry for the next parent up the chain
            let entry = color_trees.entry(parent.clone());
            match entry {
                Occupied(mut entry) => {
                    // if there's already an entry, add a child
                    let parent = entry.get_mut();
                    {
                        let parent = unsafe { Rc::get_mut_unchecked(parent) };
                        parent.children.push(tree.clone());
                    }
                    // stop exploring the parent chain
                    break;
                }
                Vacant(entry) => {
                    // otherwise, create the entry and add the child
                    let res = Rc::new(ColorTree {
                        root: parent,
                        children: vec![tree.clone()],
                        cluster: None,
                        region: None, // computed later, very anoying
                    });
                    tree = res.clone();
                    entry.insert(res);
                }
            }
        }
    }

    let mut live_roots: Vec<Rc<ColorTree>> = vec![];
    for integer_root in 0..canvas.get_roots_count() {
        let root = BlockId::new_root(integer_root);
        if let Some(tree) = color_trees.get(&root) {
            live_roots.push(tree.clone());
        }
    }

    // find the region spanned by nodes in the color trees
    let mut canvas = canvas.clone();
    for undo_move in move_history.iter().rev() {
        undo_move.undo.clone().apply(&mut canvas);
    }

    for initial_block in canvas.blocks_iter() {
        color_trees
            .entry(initial_block.id.clone())
            .and_modify(|entry| {
                unsafe { Rc::get_mut_unchecked(entry) }.region = Some(initial_block.r);
            });
    }

    for mov in move_history.iter() {
        let info = mov.mov.clone().apply(&mut canvas).unwrap();
        for created_block in info.created_blocks() {
            let block = canvas.get_block(&created_block).unwrap();
            color_trees.entry(created_block).and_modify(|entry| {
                unsafe { Rc::get_mut_unchecked(entry) }.region = Some(block.r);
            });
        }
    }

    ColorForest {
        trees: color_trees,
        roots: live_roots,
    }
}

struct WorkCluster {
    subclusters: Vec<Rc<ColorTree>>,
    cluster: ColorCluster,
}

impl WorkCluster {
    fn new(tree: &Rc<ColorTree>) -> WorkCluster {
        WorkCluster {
            subclusters: vec![tree.clone()],
            cluster: tree.cluster.as_ref().unwrap().clone(),
        }
    }

    fn merge_with(
        &self,
        other: &WorkCluster,
        region: Rect,
        canvas: &Canvas,
        painting: &Painting,
    ) -> WorkCluster {
        let mut subclusters = self.subclusters.clone();
        subclusters.extend_from_slice(&other.subclusters);
        let mut pixels = self.cluster.pixels.clone();
        pixels.extend_from_slice(&other.cluster.pixels);
        let mut cluster = ColorCluster::new(region, pixels, canvas, painting);
        // when combining clusters, our reference point is what we had before starting work
        cluster.reference_cost = self.cluster.reference_cost + other.cluster.reference_cost;
        WorkCluster {
            subclusters,
            cluster,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl nabo::Point<f32> for Color {
    fn set(&mut self, i: u32, value: nabo::NotNan<f32>) {
        self.0[i as usize] = value.into_inner() as u8;
    }

    fn get(&self, i: u32) -> nabo::NotNan<f32> {
        nabo::NotNan::new(self.0[i as usize] as f32).unwrap()
    }

    const DIM: u32 = 4;

    const DIM_BIT_COUNT: u32 = 32 - Self::DIM.leading_zeros();

    const DIM_MASK: u32 = (1 << Self::DIM_BIT_COUNT) - 1;

    const MAX_NODE_COUNT: u32 = ((1u64 << (32 - Self::DIM_BIT_COUNT)) - 1) as u32;
}

fn optimize_tree(tree: &mut Rc<ColorTree>, canvas: &Canvas, painting: &Painting) {
    let cur_tree = unsafe { Rc::get_mut_unchecked(tree) };
    assert!(
        cur_tree.region.is_some(),
        "no region for tree {}",
        cur_tree.root
    );
    if cur_tree.children.is_empty() {
        return;
    }

    // step 1: optimize all children
    for child in &mut cur_tree.children {
        optimize_tree(child, canvas, painting);
    }

    // step 2: build a list of color clusters in the subtree

    // step 3: and look for possible merges.
    //  we're comparing the cost of creating individual clusters and coloring them to the cost
    //  of the combined cluster. here's how we work:
    //   - for each cluster, we find its nearest neighbor
    //     - if the cost of the merged cluster is lower than the sum of the two, keep it and start over
    //     - otherwise, go to the next cluster

    // step 4: pick the cluster with the most cost improvement.
    //         the cost improvement is the difference between the initial cost and the new cost

    let mut clusters = vec![];
    for child in &cur_tree.children {
        ColorTree::find_clusters(child, &mut clusters);
    }

    // compute work clusters
    let mut clusters: Vec<WorkCluster> = clusters.iter().map(|c| WorkCluster::new(c)).collect();
    loop {
        let cloud: Vec<_> = clusters.iter().map(|c| c.cluster.best_color).collect();
        let kd_tree = KDTree::new(&cloud);
        let mut best_distance = f32::INFINITY;
        let mut best_pair: Option<(u32, u32)> = None;
        for cluster_i in 0..clusters.len() {
            let cluster = kd_tree.knn(2, &clusters[cluster_i].cluster.best_color);
            let cluster_a = &cluster[0];
            let cluster_b = &cluster[1];
            let cluster_a_color: na::Point4<f32> =
                clusters[cluster_a.index as usize].cluster.best_color.into();
            let cluster_b_color: na::Point4<f32> =
                clusters[cluster_b.index as usize].cluster.best_color.into();
            let dist = (cluster_a_color - cluster_b_color).norm();
            if dist < best_distance {
                best_distance = dist;
                best_pair = Some((cluster_a.index, cluster_b.index))
            }
        }
        let (mut cluster_a_i, mut cluster_b_i) = best_pair.unwrap();
        if cluster_a_i > cluster_b_i {
            (cluster_a_i, cluster_b_i) = (cluster_b_i, cluster_a_i);
        }

        // skip if not worth it
        let cluster_a = &clusters[cluster_a_i as usize];
        let cluster_b = &clusters[cluster_b_i as usize];
        let merged_cluster =
            cluster_a.merge_with(cluster_b, cur_tree.region.unwrap(), canvas, painting);
        if (cluster_a.cluster.cost() + cluster_b.cluster.cost()).0 < merged_cluster.cluster.cost().0
        {
            break;
        }

        // update the cluster list
        clusters.remove(cluster_b_i as usize);
        clusters.remove(cluster_a_i as usize);
        clusters.push(merged_cluster);

        if clusters.len() == 1 {
            break;
        }
    }

    // at this point, we have a list of possible clusters and need to pick one
    let best_cluster = clusters
        .iter_mut()
        .max_by_key(|cluster| cluster.cluster.improvement())
        .unwrap();

    // if the best cluster is no good, don't do anything
    if best_cluster.cluster.improvement() <= 0 {
        return;
    }

    // set the ref cluster of all subclusters to none
    for subcluster in &mut best_cluster.subclusters {
        let mut subcluster = unsafe { Rc::get_mut_unchecked(subcluster) };
        subcluster.cluster = None;
    }
    // attach the best cluster to our node
    cur_tree.cluster = Some(best_cluster.cluster.clone());
}

fn optimize_forest(forest: &mut ColorForest, canvas: &Canvas, painting: &Painting) {
    for root in &mut forest.roots {
        optimize_tree(root, canvas, painting);
    }
}

impl Processor for Recolorv2 {
    fn name(&self) -> &str {
        "recolorv2"
    }

    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    ) {
        let mut forest = build_initial_forest(applied_moves, canvas, painting);
        optimize_forest(&mut forest, canvas, painting);
        let planned_creation_colors = forest.get_colors();
        *applied_moves = recolor(canvas, applied_moves, planned_creation_colors);
    }
}
