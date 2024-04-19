use std::collections::{HashMap, HashSet};

use constraints::GridConstraints;
use grid::Direction;
use grid::{Color, Coord, Grid, Rule};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use z3::Params;

use crate::constraints::print_solved_grid;
use crate::grid::GridPattern;

mod constraints;
pub mod grid;

fn solve_simple(grid: &Grid) {
    match try_solve_grid(grid, 1000000) {
        GridSolveResult::Solved(solved) => {
            println!("Solved: \n{:?}", solved);
        }
        GridSolveResult::Unsolvable => {
            println!("Unsolvable");
        }
        GridSolveResult::Unknown => {
            println!("Unknown");
        }
    }
}

fn solve_underconstrained(grid: &mut Grid) {
    let mut unfillable = HashSet::new();

    let mut timeout = 1;
    loop {
        println!("Begin parallel solve with timeout: {}", timeout);
        let solved = par_solve_grid(&grid, &unfillable, timeout);
        if solved.is_empty() {
            break;
        }
        let any_filled = solved.iter().any(|(_, result)| match result {
            SolveResult::Definitely(_) => true,
            _ => false,
        });
        for (coord, result) in solved {
            match result {
                SolveResult::Definitely(color) => {
                    println!("Definitely: {:?} -> {:?}", coord, color);
                    grid.set_color(coord.i as usize, coord.j as usize, color);
                }
                SolveResult::Unfillable => {
                    println!("Unfillable: {:?}", coord);
                    unfillable.insert(coord);
                }
                SolveResult::Unknown => {}
            }
        }
        if !any_filled {
            timeout *= 2;
        }
    }
    println!("Final grid: \n{:?}", grid);
}

fn main() {
    let mut grid = Grid::new(8, 8);
    grid.add_rule(Rule::ConnectAll(Color::Dark));
    grid.add_rule(Rule::ConnectAll(Color::Light));
    grid.color_light(0, 2);
    grid.color_light(0, 3);
    grid.color_dark(0, 5);
    grid.add_rule(Rule::BanPattern(GridPattern::square2x2(
        Color::Dark,
        Color::Dark,
        Color::Dark,
        Color::Dark,
    )));
    grid.add_rule(Rule::BanPattern(GridPattern::square2x2(
        Color::Light,
        Color::Light,
        Color::Light,
        Color::Light,
    )));
    // grid.add_rule(Rule::BanPattern(GridPattern::square2x2(
    //     Color::Light,
    //     Color::Dark,
    //     Color::Dark,
    //     Color::Light,
    // )));
    solve_underconstrained(&mut grid);

    // let mut grid = Grid::new(4, 4);
    // grid.add_rule(Rule::DartNumbers);
    // grid.add_rule(Rule::VisibleCellCount);
    // grid.visible_count(0, 2, 7);
    // grid.color_dark(0, 2);
    // grid.dart_number(3, 0, Direction::Right, 2, Color::Dark);
    // solve_simple(&grid);

    // let mut grid = Grid::new(17, 17);
    // for i in 0..17 {
    //     grid.remove_square(1, i);
    //     grid.remove_square(i, 1);
    // }
    // grid.remove_square(0, 0);

    // for i in 2..17 {
    //     grid.dart_number(0, i, Direction::Down, 3, Color::Light);
    //     grid.dart_number(i, 0, Direction::Right, 3, Color::Light);
    // }

    // for (r, c, n) in [
    //     (2, 9, 2),
    //     (3, 5, 4),
    //     (3, 13, 9),
    //     (5, 3, 5),
    //     (5, 15, 9),
    //     (9, 2, 7),
    //     (9, 9, 6),
    //     (9, 16, 7),
    //     (13, 3, 8),
    //     (13, 15, 5),
    //     (15, 5, 6),
    //     (15, 13, 7),
    //     (16, 9, 5),
    // ] {
    //     grid.visible_count(r, c, n);
    //     grid.color_light(r, c);
    // }

    // grid.add_rule(Rule::BanPattern(GridPattern {
    //     pattern: vec![
    //         (Coord { i: 0, j: 0 }, Color::Dark),
    //         (Coord { i: 0, j: 1 }, Color::Dark),
    //     ],
    // }));
    // grid.add_rule(Rule::BanPattern(GridPattern {
    //     pattern: vec![
    //         (Coord { i: 0, j: 0 }, Color::Dark),
    //         (Coord { i: 1, j: 1 }, Color::Dark),
    //     ],
    // }));
    // grid.add_rule(Rule::VisibleCellCount);
    // grid.add_rule(Rule::DartNumbers);

    // solve_simple(&grid);

    // let mut grid = Grid::new(5, 12);
    // grid.add_rule(Rule::RegionAreaEqualsNumber);
    // for (r, c, n) in [
    //     (0, 2, 4),
    //     (0, 5, 3),
    //     (0, 8, 4),
    //     (0, 11, 6),
    //     (2, 1, 4),
    //     (2, 4, 3),
    //     (2, 7, 4),
    //     (2, 10, 6),
    //     (4, 0, 4),
    //     (4, 3, 3),
    //     (4, 6, 4),
    //     (4, 9, 6),
    // ] {
    //     grid.set_area_number(r, c, n);
    //     grid.color_light(r, c);
    // }
    // grid.color_light(1, 3);
    // grid.color_light(3, 8);
    // solve_simple(&grid);

    // let mut grid = Grid::new(9, 9);
    // grid.add_rule(Rule::RegionAreaEqualsNumber);
    // for (r, c, n) in [
    //     (1, 4, 5),
    //     (2, 2, 1),
    //     (4, 1, 4),
    //     (6, 2, 19),
    //     (7, 4, 1),
    //     (6, 6, 3),
    //     (4, 7, 3),
    //     (2, 6, 4),
    // ] {
    //     grid.set_area_number(r, c, n);
    //     grid.color_light(r, c);
    // }
    // grid.add_rule(Rule::BanPattern(GridPattern {
    //     pattern: vec![
    //         (Coord { i: 0, j: 0 }, Color::Dark),
    //         (Coord { i: 0, j: 1 }, Color::Dark),
    //         (Coord { i: 1, j: 0 }, Color::Dark),
    //         (Coord { i: 1, j: 1 }, Color::Dark),
    //     ],
    // }));
    // grid.add_rule(Rule::ConnectAll(Color::Dark));
    // grid.add_rule(Rule::ExactlyOneNumberPerRegion(Color::Light));
    // solve_simple(&grid);
}

enum GridSolveResult {
    Solved(Grid),
    Unsolvable,
    Unknown,
}

fn try_solve_grid(grid: &Grid, timeout: u32) -> GridSolveResult {
    println!("Trying to solve: \n{:?}", grid);
    let prepared = grid.prepare();
    let config = z3::Config::new();
    let ctx = z3::Context::new(&config);
    let constraints = GridConstraints::new(&prepared, &ctx);
    let solver = z3::Solver::new(&ctx);
    let mut params = Params::new(&ctx);
    params.set_u32("timeout", timeout * 1000);
    solver.set_params(&params);
    constraints.assert(&solver);

    match solver.check() {
        z3::SatResult::Unsat => GridSolveResult::Unsolvable,
        z3::SatResult::Unknown => GridSolveResult::Unknown,
        z3::SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let mut grid = grid.clone();
            for (coord, index) in &prepared.square_indexes {
                let color = model
                    .eval(&constraints.squares[index.0].color, false)
                    .unwrap();
                let color = match color.as_bool() {
                    Some(true) => Color::Light,
                    Some(false) => Color::Dark,
                    None => panic!("Model did not evaluate color for cell"),
                };
                grid.set_color(coord.i as usize, coord.j as usize, color);
            }
            GridSolveResult::Solved(grid)
        }
    }
}

enum SolveResult {
    Definitely(Color),
    Unfillable,
    Unknown,
}

fn par_solve_grid(
    grid: &Grid,
    unfillable: &HashSet<Coord>,
    timeout: u32,
) -> Vec<(Coord, SolveResult)> {
    let unfilled_squares = grid
        .squares()
        .filter(|(coord, square)| !unfillable.contains(coord) && square.color.is_none());
    let grid_with_squares_filled = unfilled_squares
        .flat_map(|(coord, _)| {
            let mut grid0 = grid.clone();
            grid0.color_light(coord.i as usize, coord.j as usize);
            let mut grid1 = grid.clone();
            grid1.color_dark(coord.i as usize, coord.j as usize);
            [(coord, Color::Light, grid0), (coord, Color::Dark, grid1)].into_iter()
        })
        .collect::<Vec<_>>();
    let result = grid_with_squares_filled
        .into_par_iter()
        .map(|(coord, color, grid)| {
            let result = try_solve_grid(&grid, timeout);
            (coord, color, result)
        })
        .collect::<Vec<_>>();

    let mut coord_solves = HashMap::new();
    let mut coord_result = HashMap::new();
    for (coord, color, result) in result {
        coord_result.entry(coord).or_insert(SolveResult::Unknown);
        match result {
            GridSolveResult::Solved(grid) => {
                let entry = coord_solves.entry(coord).or_insert(0);
                *entry += 1;
                if entry == &2 {
                    coord_result.insert(coord, SolveResult::Unfillable);
                }
            }
            GridSolveResult::Unsolvable => {
                coord_result.insert(coord, SolveResult::Definitely(color.opposite()));
            }
            _ => {}
        }
    }
    coord_result.into_iter().collect()
}
