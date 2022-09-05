use std::fmt::Display;

use rand::random;

use crate::{
    block::{Block, Rect},
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Move, Orientation},
    painting::Painting,
};

use super::Solver;

#[derive(Debug)]
pub struct Genetic {
    initial_population: u32,
}

impl Genetic {
    pub fn new() -> Self {
        Genetic {
            initial_population: 100,
        }
    }
}

#[derive(Debug, Clone)]
struct Individual {
    chromosome: (Canvas, Vec<AppliedMove>),
    fitness: u64,
    id: String,
    desired: Painting,
}

impl Display for Individual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id={} fitness={}", self.id, self.fitness)
    }
}

impl Individual {
    fn mate(&self, another: &Individual) -> Individual {
        let mut applied = vec![];
        let my_moves = &self.chromosome.1;
        let another_moves = &another.chromosome.1;
        let mut state = self.chromosome.0.clone();
        // println!("me = {:?}", my_moves);
        // println!("another = {:?}", another_moves);

        for mov in my_moves {
            // println!("{:?}", mov);
            if let Ok(applied_mv) = mov.clone().mov.apply(&mut state) {
                applied.push(applied_mv);
            }
        }

        for mov in another_moves {
            // println!("{:?}", mov);
            if let Ok(applied_mv) = mov.clone().mov.apply(&mut state) {
                applied.push(applied_mv);
            }
        }

        let id = format!("{}+{}", self.id, another.id);
        let fitness = state.render().calculate_score(&self.desired).0;

        // println!("id={id} fitness={fitness}");

        Individual {
            chromosome: (state.clone(), applied),
            fitness,
            id,
            desired: self.desired.clone(),
        }
    }
}

fn generate_population(
    initial_state: &Canvas,
    count: usize,
    painting: &Painting,
) -> Vec<Individual> {
    let mut population = Vec::with_capacity(count);

    for i in 0..count {
        population.push(generate_individual(&mut initial_state.clone(), i, painting));
    }

    population
}

fn generate_individual(state: &mut Canvas, id: usize, painting: &Painting) -> Individual {
    let mut moves = vec![];
    let mut applied_moves = vec![];

    for b in state.blocks_iter() {
        let mov = get_move_for_block(&state, b);
        moves.push(mov);
    }

    for mov in moves {
        if let Some(mov) = mov {
            if let Ok(applied) = mov.apply(state) {
                //.expect("Can't apply move");
                // println!("applied {:?}", applied);
                applied_moves.push(applied);
            }
        }
    }

    let fitness = state.render().calculate_score(painting).0;
    Individual {
        chromosome: (state.clone(), applied_moves),
        fitness,
        id: id.to_string(),
        desired: painting.clone(),
    }
}

fn get_move_for_block(state: &Canvas, block: &Block) -> Option<Move> {
    let selector = random::<u32>() % 6;
    match selector {
        0 => {
            let r = random::<u8>();
            let g = random::<u8>();
            let b = random::<u8>();
            let a = random::<u8>();
            Some(Move::Color(block.id.clone(), Color::new(r, g, b, a)))
        }
        1 => {
            let r = random::<u32>() % block.r.width();
            let cut_at = block.r.top_right.x - r;

            if cut_at > block.r.bottom_left.x && cut_at < block.r.top_right.x {
                Some(Move::LineCut(
                    block.id.clone(),
                    Orientation::Vertical,
                    cut_at,
                ))
            } else {
                None
            }
        }
        2 => {
            let r = random::<u32>() % block.r.height();
            let cut_at = block.r.top_right.y - r;

            if cut_at > block.r.bottom_left.y && cut_at < block.r.top_right.y {
                Some(Move::LineCut(
                    block.id.clone(),
                    Orientation::Horizontal,
                    cut_at,
                ))
            } else {
                None
            }
        }
        3 => {
            let r_x = random::<u32>() % block.r.width();
            let r_y = random::<u32>() % block.r.height();

            let cut_at_x = block.r.top_right.x - r_x;
            let cut_at_y = block.r.top_right.y - r_y;

            if cut_at_x > block.r.bottom_left.x
                && cut_at_x < block.r.top_right.x
                && cut_at_y > block.r.bottom_left.y
                && cut_at_y < block.r.top_right.y
            {
                Some(Move::PointCut(block.id.clone(), cut_at_x, cut_at_y))
            } else {
                None
            }
        }
        4 => state
            .blocks_iter()
            .find(|bl| bl.area() == block.area())
            .map(|bl| Move::Swap(block.id.clone(), bl.id.clone())),
        5 => state
            .blocks_iter()
            .find(|bl| is_mergeable(bl, block))
            .map(|bl| Move::Merge(block.id.clone(), bl.id.clone())),
        _ => panic!("move selector is not yet implemented"),
    }
}

fn is_mergeable(a: &Block, b: &Block) -> bool {
    let Block {
        r:
            Rect {
                bottom_left: a_bottom_left,
                top_right: a_top_right,
            },
        ..
    } = a;
    let Block {
        r:
            Rect {
                bottom_left: b_bottom_left,
                top_right: b_top_right,
            },
        ..
    } = b;

    let vertical = (a_bottom_left.y == b_top_right.y || a_top_right.y == b_bottom_left.y)
        && a_bottom_left.x == b_bottom_left.x
        && a_top_right.x == b_top_right.x;

    let horizontal = (b_top_right.x == a_bottom_left.x || a_top_right.x == b_bottom_left.x)
        && a_bottom_left.y == b_bottom_left.y
        && a_top_right.y == b_top_right.y;

    vertical || horizontal
}

impl Solver for Genetic {
    fn name(&self) -> &str {
        "genetic"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut population =
            generate_population(&canvas, self.initial_population as usize, painting);

        let mut best = population[0].clone();

        for _ in 0..20 {
            let (current_best, new_population) = breed(&mut population);
            population = new_population;

            // println!("new_population={}", population.len());
            // println!("best_score={}", current_best.fitness);

            best = current_best;
        }

        best.chromosome.1
    }
}

fn breed(population: &mut Vec<Individual>) -> (Individual, Vec<Individual>) {
    population.sort_by_key(|ind| ind.fitness);
    let best = population[0].clone();

    let mut new_generation = vec![];
    let elitist_num = (population.len() as f32 * 0.1) as u32;

    for i in 0..=elitist_num as usize {
        new_generation.push(population[i].clone())
    }

    let ordinary_num = population.len() as u32 - elitist_num - 1;
    for _i in 0..ordinary_num as u32 {
        let individ_1_i = elitist_num + rand::random::<u32>() % ordinary_num;
        let individ_2_i = elitist_num + rand::random::<u32>() % ordinary_num;

        let individ_1 = &population[individ_1_i as usize];
        let individ_2 = &population[individ_2_i as usize];

        let new_individ = individ_1.mate(individ_2);
        new_generation.push(new_individ);
    }

    (best, new_generation.clone())
}
