use crate::grid::{Color, Coord, PreparedGrid, PreparedRule};
use z3::{
    ast::{self, Ast},
    Solver,
};

pub struct SquareVariables<'ctx> {
    id: ast::Int<'ctx>,

    pub color: ast::Bool<'ctx>,
    region_leader: ast::Int<'ctx>,
    region_rank: ast::Int<'ctx>,
    region_size: ast::Int<'ctx>,
    is_leader: ast::Bool<'ctx>,

    left_visible: ast::Int<'ctx>,
    right_visible: ast::Int<'ctx>,
    top_visible: ast::Int<'ctx>,
    bottom_visible: ast::Int<'ctx>,
    visible_total: ast::Int<'ctx>,
}

pub struct AuxVariables<'ctx> {
    dark_leader: ast::Int<'ctx>,
    light_leader: ast::Int<'ctx>,
    zero: ast::Int<'ctx>,
    one: ast::Int<'ctx>,
}

impl<'ctx> SquareVariables<'ctx> {
    pub fn new(id: usize, ctx: &'ctx z3::Context) -> SquareVariables<'ctx> {
        let color = ast::Bool::new_const(ctx, format!("color_{}", id));
        let id = ast::Int::from_u64(ctx, id as u64);
        let region_leader = ast::Int::new_const(ctx, format!("region_leader_{}", id));
        let region_rank = ast::Int::new_const(ctx, format!("region_rank_{}", id));
        let region_size = ast::Int::new_const(ctx, format!("region_size_{}", id));
        let is_leader = region_leader._eq(&id);

        let left_visible = ast::Int::new_const(ctx, format!("left_visible_{}", id));
        let right_visible = ast::Int::new_const(ctx, format!("right_visible_{}", id));
        let top_visible = ast::Int::new_const(ctx, format!("top_visible_{}", id));
        let bottom_visible = ast::Int::new_const(ctx, format!("bottom_visible_{}", id));
        let visible_total = ast::Int::add(
            ctx,
            &[
                &left_visible,
                &right_visible,
                &top_visible,
                &bottom_visible,
                &ast::Int::from_u64(ctx, 1),
            ],
        );

        SquareVariables {
            id,
            color,
            region_leader,
            region_rank,
            region_size,
            is_leader,

            left_visible,
            right_visible,
            top_visible,
            bottom_visible,
            visible_total,
        }
    }
}

pub struct GridConstraints<'ctx> {
    pub squares: Vec<SquareVariables<'ctx>>,
    pub aux: AuxVariables<'ctx>,
    pub basic_constraints: Vec<ast::Bool<'ctx>>,
    pub rule_constraints: Vec<ast::Bool<'ctx>>,
}

impl Color {
    pub fn to_bool<'ctx>(&self, color_var: &ast::Bool<'ctx>) -> ast::Bool<'ctx> {
        match self {
            Color::Light => color_var.clone(),
            Color::Dark => color_var.not(),
        }
    }
}

impl<'ctx> GridConstraints<'ctx> {
    pub fn new(grid: &PreparedGrid, ctx: &'ctx z3::Context) -> GridConstraints<'ctx> {
        let squares = grid
            .squares
            .iter()
            .map(|square| SquareVariables::new(square.index.0, ctx))
            .collect::<Vec<_>>();
        let aux = AuxVariables {
            dark_leader: ast::Int::new_const(ctx, "dark_leader"),
            light_leader: ast::Int::new_const(ctx, "light_leader"),
            zero: ast::Int::from_u64(ctx, 0),
            one: ast::Int::from_u64(ctx, 1),
        };
        let mut constraints = GridConstraints {
            squares,
            aux,
            basic_constraints: Vec::new(),
            rule_constraints: Vec::new(),
        };
        constraints.add_basic_constraints_for_variables(grid, ctx);
        for rule in &grid.rules {
            constraints.add_constraints_for_rule(rule, grid, ctx);
        }
        constraints
    }

    fn add_basic_constraints_for_variables(&mut self, grid: &PreparedGrid, ctx: &'ctx z3::Context) {
        for square in &grid.squares {
            let square_vars = &self.squares[square.index.0];
            let is_leader = square_vars.is_leader.clone();
            // Rank is at least 0.
            self.basic_constraints
                .push(square_vars.region_rank.ge(&self.aux.zero));
            // Rank == 0 is equivalent to being region leader.
            self.basic_constraints
                .push(is_leader._eq(&square_vars.region_rank._eq(&self.aux.zero)));
            // ID is >= region leader.
            self.basic_constraints
                .push(square_vars.id.ge(&square_vars.region_leader));
            if let Some(right) = square.right {
                // Neighbor color being the same is equivalent to their region leaders being the same.
                let right_vars = &self.squares[right.0];
                let color_same = square_vars.color._eq(&right_vars.color);
                self.basic_constraints.push(
                    color_same._eq(&square_vars.region_leader._eq(&right_vars.region_leader)),
                );
                // Region rank differs by exactly one.
                self.basic_constraints
                    .push(color_same.implies(&ast::Bool::or(
                        ctx,
                        &[
                            &square_vars.region_rank._eq(&ast::Int::add(
                                ctx,
                                &[&right_vars.region_rank, &self.aux.one],
                            )),
                            &right_vars.region_rank._eq(&ast::Int::add(
                                ctx,
                                &[&square_vars.region_rank, &self.aux.one],
                            )),
                        ],
                    )));
            }
            if let Some(below) = square.below {
                // Neighbor color being the same is equivalent to their region leaders being the same.
                let below_vars = &self.squares[below.0];
                let color_same = square_vars.color._eq(&below_vars.color);
                self.basic_constraints.push(
                    color_same._eq(&square_vars.region_leader._eq(&below_vars.region_leader)),
                );
                // Region rank differs by exactly one.
                self.basic_constraints
                    .push(color_same.implies(&ast::Bool::or(
                        ctx,
                        &[
                            &square_vars.region_rank._eq(&ast::Int::add(
                                ctx,
                                &[&below_vars.region_rank, &self.aux.one],
                            )),
                            &below_vars.region_rank._eq(&ast::Int::add(
                                ctx,
                                &[&square_vars.region_rank, &self.aux.one],
                            )),
                        ],
                    )));
            }
            let neighbors = [square.left, square.right, square.above, square.below]
                .into_iter()
                .filter_map(|x| x)
                .collect::<Vec<_>>();

            // Either rank is zero, or there's at least one neighbor with same color and rank - 1.
            let mut rank_cases = Vec::new();
            rank_cases.push(square_vars.region_rank._eq(&self.aux.zero));
            for neighbor in &neighbors {
                let neighbor_vars = &self.squares[neighbor.0];
                rank_cases.push(ast::Bool::and(
                    ctx,
                    &[
                        &square_vars.color._eq(&neighbor_vars.color),
                        &square_vars.region_rank._eq(&ast::Int::add(
                            ctx,
                            &[&neighbor_vars.region_rank, &self.aux.one],
                        )),
                    ],
                ));
            }
            self.basic_constraints
                .push(ast::Bool::or(ctx, &rank_cases.iter().collect::<Vec<_>>()));

            // Region size is the number of cells that share the same region leader.
            let mut region_size_components = Vec::new();
            for other in &grid.squares {
                let other_vars = &self.squares[other.index.0];
                region_size_components.push(
                    square_vars
                        .region_leader
                        ._eq(&other_vars.region_leader)
                        .ite(&self.aux.one, &self.aux.zero),
                );
            }
            self.basic_constraints
                .push(square_vars.region_size._eq(&ast::Int::add(
                    ctx,
                    &region_size_components.iter().collect::<Vec<_>>(),
                )));

            // A square's top_visible is 1 plus its above neighbor's top_visible if they have the same color,
            // otherwise zero.
            if let Some(above) = square.above {
                let above_vars = &self.squares[above.0];
                self.basic_constraints
                    .push(square_vars.color._eq(&above_vars.color).ite(
                        &square_vars.top_visible._eq(&ast::Int::add(
                            ctx,
                            &[&above_vars.top_visible, &self.aux.one],
                        )),
                        &square_vars.top_visible._eq(&self.aux.zero),
                    ));
            } else {
                self.basic_constraints
                    .push(square_vars.top_visible._eq(&self.aux.zero));
            }

            // A square's bottom_visible is 1 plus its below neighbor's bottom_visible if they have the same color,
            // otherwise zero.
            if let Some(below) = square.below {
                let below_vars = &self.squares[below.0];
                self.basic_constraints
                    .push(square_vars.color._eq(&below_vars.color).ite(
                        &square_vars.bottom_visible._eq(&ast::Int::add(
                            ctx,
                            &[&below_vars.bottom_visible, &self.aux.one],
                        )),
                        &square_vars.bottom_visible._eq(&self.aux.zero),
                    ));
            } else {
                self.basic_constraints
                    .push(square_vars.bottom_visible._eq(&self.aux.zero));
            }

            // A square's left_visible is 1 plus its left neighbor's left_visible if they have the same color,
            // otherwise zero.
            if let Some(left) = square.left {
                let left_vars = &self.squares[left.0];
                self.basic_constraints
                    .push(square_vars.color._eq(&left_vars.color).ite(
                        &square_vars.left_visible._eq(&ast::Int::add(
                            ctx,
                            &[&left_vars.left_visible, &self.aux.one],
                        )),
                        &square_vars.left_visible._eq(&self.aux.zero),
                    ));
            } else {
                self.basic_constraints
                    .push(square_vars.left_visible._eq(&self.aux.zero));
            }

            // A square's right_visible is 1 plus its right neighbor's right_visible if they have the same color,
            // otherwise zero.
            if let Some(right) = square.right {
                let right_vars = &self.squares[right.0];
                self.basic_constraints
                    .push(square_vars.color._eq(&right_vars.color).ite(
                        &square_vars.right_visible._eq(&ast::Int::add(
                            ctx,
                            &[&right_vars.right_visible, &self.aux.one],
                        )),
                        &square_vars.right_visible._eq(&self.aux.zero),
                    ));
            } else {
                self.basic_constraints
                    .push(square_vars.right_visible._eq(&self.aux.zero));
            }
        }
    }

    fn add_constraints_for_rule(
        &mut self,
        rule: &PreparedRule,
        grid: &PreparedGrid,
        ctx: &'ctx z3::Context,
    ) {
        match rule {
            PreparedRule::SquareIsColor(index, color) => {
                let square = &self.squares[index.0];
                self.rule_constraints.push(color.to_bool(&square.color));
            }
            PreparedRule::SquaresAreSameColor(a, b) => {
                let square_a = &self.squares[a.0];
                let square_b = &self.squares[b.0];
                self.rule_constraints
                    .push(square_a.color._eq(&square_b.color));
            }
            PreparedRule::BanPattern(grid_pattern) => {
                for i in 0..grid.size.i {
                    'outer: for j in 0..grid.size.j {
                        let offset = grid_pattern.offset(Coord { i, j });
                        let mut and_terms = Vec::new();
                        for (coord, color) in &offset.pattern {
                            if let Some(index) = grid.square_indexes.get(coord) {
                                let square = &self.squares[index.0];
                                and_terms.push(color.to_bool(&square.color));
                            } else {
                                continue 'outer;
                            }
                        }
                        let and = ast::Bool::and(ctx, &and_terms.iter().collect::<Vec<_>>());
                        self.rule_constraints.push(and.not());
                    }
                }
            }
            PreparedRule::ConnectAll(color) => {
                let leader = match color {
                    Color::Dark => &self.aux.dark_leader,
                    Color::Light => &self.aux.light_leader,
                };
                for square in &self.squares {
                    self.rule_constraints.push(
                        color
                            .to_bool(&square.color)
                            .implies(&square.region_leader._eq(leader)),
                    );
                }
            }
            PreparedRule::RegionFixedSize(color, size) => {
                let size_int = ast::Int::from_u64(ctx, *size as u64);
                for square in &self.squares {
                    self.rule_constraints.push(
                        color
                            .to_bool(&square.color)
                            .implies(&square.region_size._eq(&size_int)),
                    );
                }
            }
            PreparedRule::ExactlyOneNumberPerRegion(color, numbered_squares) => {
                self.rule_constraints.push(ast::Int::distinct(
                    ctx,
                    &numbered_squares
                        .iter()
                        .map(|index| &self.squares[index.0].region_leader)
                        .collect::<Vec<_>>(),
                ));
                for square in &self.squares {
                    let mut or_terms = Vec::new();
                    for index in numbered_squares {
                        let other = &self.squares[index.0];
                        or_terms.push(square.region_leader._eq(&other.region_leader));
                    }
                    self.rule_constraints.push(
                        color
                            .to_bool(&square.color)
                            .implies(&ast::Bool::or(ctx, &or_terms.iter().collect::<Vec<_>>())),
                    );
                }
            }
            PreparedRule::RegionAreaEqualsNumber(index, number) => {
                let number_int = ast::Int::from_u64(ctx, *number as u64);
                self.rule_constraints
                    .push(self.squares[index.0].region_size._eq(&number_int));
            }
            PreparedRule::VisibleCellCount(index, number) => {
                let number_int = ast::Int::from_u64(ctx, *number as u64);
                self.rule_constraints
                    .push(self.squares[index.0].visible_total._eq(&number_int));
            }
            PreparedRule::RegionAreaEqualsEither(index, a, b) => {
                let a_int = ast::Int::from_u64(ctx, *a as u64);
                let b_int = ast::Int::from_u64(ctx, *b as u64);
                self.rule_constraints.push(ast::Bool::or(
                    ctx,
                    &[
                        &self.squares[index.0].region_size._eq(&a_int),
                        &self.squares[index.0].region_size._eq(&b_int),
                    ],
                ));
            }
            PreparedRule::VisibleCellCountEither(index, a, b) => {
                let a_int = ast::Int::from_u64(ctx, *a as u64);
                let b_int = ast::Int::from_u64(ctx, *b as u64);
                self.rule_constraints.push(ast::Bool::or(
                    ctx,
                    &[
                        &self.squares[index.0].visible_total._eq(&a_int),
                        &self.squares[index.0].visible_total._eq(&b_int),
                    ],
                ));
            }
            PreparedRule::ColorCountInSet(count, color, set) => {
                let mut components = Vec::new();
                for index in set {
                    components.push(
                        color
                            .to_bool(&self.squares[index.0].color)
                            .ite(&self.aux.one, &self.aux.zero),
                    );
                }
                self.rule_constraints.push(
                    ast::Int::add(ctx, &components.iter().collect::<Vec<_>>())
                        ._eq(&ast::Int::from_u64(ctx, *count as u64)),
                );
            }
            PreparedRule::RegionsHaveDifferentShapes(_) => todo!(),
        }
    }

    pub fn assert(&self, solver: &Solver<'ctx>) {
        for constraint in &self.basic_constraints {
            solver.assert(constraint);
        }
        for constraint in &self.rule_constraints {
            solver.assert(constraint);
        }
    }
}

pub enum PrintKind {
    Color,
    RegionSize,
    RegionLeader,
    RegionRank,
    VisibleTotal,
}

impl PrintKind {
    pub fn column_width(&self) -> usize {
        match self {
            PrintKind::Color => 2,
            PrintKind::RegionSize => 3,
            PrintKind::RegionLeader => 4,
            PrintKind::RegionRank => 3,
            PrintKind::VisibleTotal => 3,
        }
    }

    pub fn print_square<'ctx>(
        &self,
        square: &SquareVariables<'ctx>,
        model: &z3::Model<'ctx>,
    ) -> String {
        let mut res = match self {
            PrintKind::Color => {
                if model.eval(&square.color, false).unwrap().as_bool().unwrap() {
                    "□".to_string()
                } else {
                    "■".to_string()
                }
            }
            PrintKind::RegionSize => model
                .eval(&square.region_size, false)
                .unwrap()
                .as_i64()
                .unwrap()
                .to_string(),
            PrintKind::RegionLeader => model
                .eval(&square.region_leader, false)
                .unwrap()
                .as_i64()
                .unwrap()
                .to_string(),
            PrintKind::RegionRank => model
                .eval(&square.region_rank, false)
                .unwrap()
                .as_i64()
                .unwrap()
                .to_string(),
            PrintKind::VisibleTotal => model
                .eval(&square.visible_total, false)
                .unwrap()
                .as_i64()
                .unwrap()
                .to_string(),
        };
        while res.len() < self.column_width() {
            res.push(' ');
        }
        res
    }
}

pub fn print_solved_grid(
    grid: &PreparedGrid,
    constraints: &GridConstraints<'_>,
    model: &z3::Model<'_>,
    kind: PrintKind,
) -> String {
    let mut res = String::new();
    for i in 0..grid.size.i {
        for j in 0..grid.size.j {
            if let Some(index) = grid.square_indexes.get(&Coord { i, j }) {
                let square = &constraints.squares[index.0];
                res.push_str(&kind.print_square(square, model));
            } else {
                res.push_str(&" ".to_string().repeat(kind.column_width()));
            }
        }
        res.push('\n');
    }
    res
}
