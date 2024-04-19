use std::{collections::HashMap, fmt::Debug};

#[derive(Clone)]
pub struct Grid {
    size: Coord,
    squares: Vec<Vec<Square>>,
    rules: Vec<Rule>,
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..self.size.i {
            for y in 0..self.size.j {
                let square = &self.squares[x as usize][y as usize];
                if square.exists {
                    match self.squares[x as usize][y as usize].color {
                        Some(Color::Light) => write!(f, "□ ")?,
                        Some(Color::Dark) => write!(f, "■ ")?,
                        None => write!(f, "_ ")?,
                    }
                } else {
                    write!(f, "  ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    pub fn square(&self, coord: Coord) -> Option<&Square> {
        if coord.i < 0
            || coord.i >= self.size.i
            || coord.j < 0
            || coord.j >= self.size.j
            || !self.squares[coord.i as usize][coord.j as usize].exists
        {
            None
        } else {
            Some(&self.squares[coord.i as usize][coord.j as usize])
        }
    }

    pub fn squares(&self) -> impl Iterator<Item = (Coord, Square)> + '_ {
        (0..self.size.i).flat_map(move |x| {
            (0..self.size.j).filter_map(move |y| {
                if self.squares[x as usize][y as usize].exists {
                    Some((Coord { i: x, j: y }, self.squares[x as usize][y as usize]))
                } else {
                    None
                }
            })
        })
    }

    pub fn new(rows: usize, cols: usize) -> Grid {
        Grid {
            size: Coord {
                i: rows as isize,
                j: cols as isize,
            },
            squares: vec![
                vec![
                    Square {
                        exists: true,
                        merge_with_right: false,
                        merge_with_bottom: false,
                        color: None,
                        area_number: None,
                        visible_count: None,
                        dart_number: None,
                    };
                    cols
                ];
                rows
            ],
            rules: Vec::new(),
        }
    }

    pub fn remove_square(&mut self, row: usize, col: usize) {
        self.squares[row][col].exists = false;
    }

    pub fn set_area_number(&mut self, row: usize, col: usize, number: usize) {
        self.squares[row][col].area_number = Some(number);
    }

    pub fn color_light(&mut self, row: usize, col: usize) {
        self.squares[row][col].color = Some(Color::Light);
    }

    pub fn color_dark(&mut self, row: usize, col: usize) {
        self.squares[row][col].color = Some(Color::Dark);
    }

    pub fn set_color(&mut self, row: usize, col: usize, color: Color) {
        self.squares[row][col].color = Some(color);
    }

    pub fn join_right(&mut self, row: usize, col: usize) {
        self.squares[row][col].merge_with_right = true;
    }

    pub fn join_bottom(&mut self, row: usize, col: usize) {
        self.squares[row][col].merge_with_bottom = true;
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn dart_number(
        &mut self,
        row: usize,
        col: usize,
        direction: Direction,
        number: usize,
        color: Color,
    ) {
        self.squares[row][col].dart_number = Some((direction, number));
        self.squares[row][col].color = Some(color);
    }

    pub fn visible_count(&mut self, row: usize, col: usize, count: usize) {
        self.squares[row][col].visible_count = Some(count);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Square {
    pub exists: bool,
    pub merge_with_right: bool,
    pub merge_with_bottom: bool,
    pub color: Option<Color>,
    pub area_number: Option<usize>,
    pub visible_count: Option<usize>,
    pub dart_number: Option<(Direction, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SquareIndex(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    Light,
    Dark,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::Light => Color::Dark,
            Color::Dark => Color::Light,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Coord {
    pub i: isize,
    pub j: isize,
}

impl Coord {
    fn right(self, size: Coord) -> Option<Coord> {
        if self.j < size.j - 1 {
            Some(Coord {
                i: self.i,
                j: self.j + 1,
            })
        } else {
            None
        }
    }

    fn below(self, size: Coord) -> Option<Coord> {
        if self.i < size.i - 1 {
            Some(Coord {
                i: self.i + 1,
                j: self.j,
            })
        } else {
            None
        }
    }

    fn left(self) -> Option<Coord> {
        if self.j > 0 {
            Some(Coord {
                i: self.i,
                j: self.j - 1,
            })
        } else {
            None
        }
    }

    fn above(self) -> Option<Coord> {
        if self.i > 0 {
            Some(Coord {
                i: self.i - 1,
                j: self.j,
            })
        } else {
            None
        }
    }

    fn neighbor(self, direction: Direction, size: Coord) -> Option<Coord> {
        match direction {
            Direction::Up => self.above(),
            Direction::Down => self.below(size),
            Direction::Left => self.left(),
            Direction::Right => self.right(size),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GridPattern {
    pub pattern: Vec<(Coord, Color)>,
}

impl GridPattern {
    pub fn square2x2(a: Color, b: Color, c: Color, d: Color) -> GridPattern {
        GridPattern {
            pattern: vec![
                (Coord { i: 0, j: 0 }, a),
                (Coord { i: 0, j: 1 }, b),
                (Coord { i: 1, j: 0 }, c),
                (Coord { i: 1, j: 1 }, d),
            ],
        }
    }

    fn canonicalize(self) -> GridPattern {
        let Self { mut pattern } = self;
        pattern.sort_by_key(|&(coord, _)| coord);
        let min_x = pattern.iter().map(|&(coord, _)| coord.i).min().unwrap();
        let min_y = pattern.iter().map(|&(coord, _)| coord.j).min().unwrap();
        for (coord, _) in &mut pattern {
            coord.i -= min_x;
            coord.j -= min_y;
        }
        GridPattern { pattern }
    }

    pub fn rotate(&self) -> GridPattern {
        let mut pattern = self.pattern.clone();
        for (coord, _) in &mut pattern {
            let x = coord.i;
            coord.i = -coord.j;
            coord.j = x;
        }
        GridPattern { pattern }.canonicalize()
    }

    pub fn reflect(&self) -> GridPattern {
        let mut pattern = self.pattern.clone();
        for (coord, _) in &mut pattern {
            coord.i = -coord.i;
        }
        GridPattern { pattern }.canonicalize()
    }

    pub fn all_rotations_and_reflections(&self) -> Vec<GridPattern> {
        let mut result = Vec::new();
        let mut current = self.clone();
        for _ in 0..4 {
            if !result.contains(&current) {
                result.push(current.clone());
            }
            let reflected = current.reflect();
            if !result.contains(&reflected) {
                result.push(reflected);
            }
            current = current.rotate();
        }
        result
    }

    pub fn offset(&self, by: Coord) -> GridPattern {
        let mut pattern = self.pattern.clone();
        for (coord, _) in &mut pattern {
            coord.i += by.i;
            coord.j += by.j;
        }
        GridPattern { pattern }
    }
}

impl Debug for GridPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut min_x = std::isize::MAX;
        let mut min_y = std::isize::MAX;
        let mut max_x = std::isize::MIN;
        let mut max_y = std::isize::MIN;
        for (coord, _) in &self.pattern {
            min_x = min_x.min(coord.i);
            min_y = min_y.min(coord.j);
            max_x = max_x.max(coord.i);
            max_y = max_y.max(coord.j);
        }
        writeln!(f)?;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let mut found = false;
                for (coord, color) in &self.pattern {
                    if coord.i == x && coord.j == y {
                        write!(
                            f,
                            "{}",
                            match color {
                                Color::Light => "□ ",
                                Color::Dark => "■ ",
                            }
                        )?;
                        found = true;
                    }
                }
                if !found {
                    write!(f, "  ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    BanPattern(GridPattern),
    ConnectAll(Color),
    RegionAreaEqualsNumber,
    RegionFixedSize(Color, usize),
    ExactlyOneNumberPerRegion(Color),
    VisibleCellCount,
    RegionsHaveDifferentShapes(Color),
    NumbersAreOffByOne,
    DartNumbers,
    // TODO: lotus, galaxy, letters
}

#[derive(Debug)]
pub enum PreparedRule {
    SquareIsColor(SquareIndex, Color),
    SquaresAreSameColor(SquareIndex, SquareIndex),
    BanPattern(GridPattern),
    ConnectAll(Color),
    RegionFixedSize(Color, usize),
    ExactlyOneNumberPerRegion(Color, Vec<SquareIndex>),
    RegionAreaEqualsNumber(SquareIndex, usize),
    VisibleCellCount(SquareIndex, usize),
    RegionAreaEqualsEither(SquareIndex, usize, usize),
    VisibleCellCountEither(SquareIndex, usize, usize),
    RegionsHaveDifferentShapes(Color),
    ColorCountInSet(usize, Color, Vec<SquareIndex>),
}

#[derive(Debug)]
pub struct PreparedSquare {
    pub index: SquareIndex,

    pub left: Option<SquareIndex>,
    pub right: Option<SquareIndex>,
    pub above: Option<SquareIndex>,
    pub below: Option<SquareIndex>,
}

#[derive(Debug)]
pub struct PreparedGrid {
    pub size: Coord,
    pub square_indexes: HashMap<Coord, SquareIndex>,
    pub squares: Vec<PreparedSquare>,
    pub rules: Vec<PreparedRule>,
}

impl Grid {
    pub fn prepare_rule(
        &self,
        rule: &Rule,
        square_indexes: &HashMap<Coord, SquareIndex>,
        off_by_one: bool,
        prepared: &mut Vec<PreparedRule>,
    ) {
        match rule {
            Rule::BanPattern(pattern) => {
                for pattern in pattern.all_rotations_and_reflections() {
                    prepared.push(PreparedRule::BanPattern(pattern));
                }
            }
            Rule::ConnectAll(color) => {
                prepared.push(PreparedRule::ConnectAll(*color));
            }
            Rule::RegionAreaEqualsNumber => {
                for (coord, square) in self.squares() {
                    let index = square_indexes[&coord];
                    if let Some(area_number) = square.area_number {
                        if off_by_one {
                            prepared.push(PreparedRule::RegionAreaEqualsEither(
                                index,
                                area_number - 1,
                                area_number + 1,
                            ));
                        } else {
                            prepared.push(PreparedRule::RegionAreaEqualsNumber(index, area_number));
                        }
                    }
                }
            }
            Rule::RegionFixedSize(color, size) => {
                prepared.push(PreparedRule::RegionFixedSize(*color, *size));
            }
            Rule::ExactlyOneNumberPerRegion(color) => {
                let mut squares_with_number = Vec::new();
                for (coord, square) in self.squares() {
                    let index = square_indexes[&coord];
                    if square.area_number.is_some() {
                        squares_with_number.push(index);
                    }
                }
                prepared.push(PreparedRule::ExactlyOneNumberPerRegion(
                    *color,
                    squares_with_number,
                ));
            }
            Rule::VisibleCellCount => {
                for (coord, square) in self.squares() {
                    let index = square_indexes[&coord];
                    if let Some(visible_count) = square.visible_count {
                        if off_by_one {
                            prepared.push(PreparedRule::VisibleCellCountEither(
                                index,
                                visible_count - 1,
                                visible_count + 1,
                            ));
                        } else {
                            prepared.push(PreparedRule::VisibleCellCount(index, visible_count));
                        }
                    }
                }
            }
            Rule::RegionsHaveDifferentShapes(color) => {
                prepared.push(PreparedRule::RegionsHaveDifferentShapes(*color));
            }
            Rule::NumbersAreOffByOne => {}
            Rule::DartNumbers => {
                for (coord, square) in self.squares() {
                    if let Some((direction, number)) = square.dart_number {
                        let color = square.color.expect("Dart number must come with color");
                        let mut squares = Vec::new();
                        let mut current = coord.neighbor(direction, self.size);
                        while let Some(coord) = current {
                            // We allow skipping over non-existent squares.
                            if let Some(index) = square_indexes.get(&coord) {
                                squares.push(*index);
                            }
                            current = coord.neighbor(direction, self.size);
                        }
                        prepared.push(PreparedRule::ColorCountInSet(
                            number,
                            color.opposite(),
                            squares,
                        ));
                    }
                }
            }
        }
    }

    fn prepare_square(
        &self,
        square: &Square,
        coord: Coord,
        square_indexes: &HashMap<Coord, SquareIndex>,
        prepared: &mut Vec<PreparedRule>,
    ) {
        let index = square_indexes[&coord];
        if let Some(color) = square.color {
            prepared.push(PreparedRule::SquareIsColor(index, color));
        }
        if square.merge_with_right {
            if let Some(right) = coord.right(self.size) {
                let right_index = square_indexes[&right];
                prepared.push(PreparedRule::SquaresAreSameColor(index, right_index));
            }
        }
        if square.merge_with_bottom {
            if let Some(bottom) = coord.below(self.size) {
                let bottom_index = square_indexes[&bottom];
                prepared.push(PreparedRule::SquaresAreSameColor(index, bottom_index));
            }
        }
    }

    pub fn prepare(&self) -> PreparedGrid {
        let mut prepared_squares: Vec<PreparedSquare> = Default::default();
        let mut square_indexes: HashMap<Coord, SquareIndex> = Default::default();

        let mut next_index = 0;
        for (coord, _) in self.squares() {
            let index = SquareIndex(next_index);
            next_index += 1;
            square_indexes.insert(coord, index);
        }

        for (coord, _) in self.squares() {
            let index = square_indexes[&coord];
            let mut prepared = PreparedSquare {
                index,
                left: None,
                right: None,
                above: None,
                below: None,
            };
            if let Some(left) = coord.left() {
                if let Some(left_index) = square_indexes.get(&left) {
                    prepared.left = Some(*left_index);
                }
            }
            if let Some(right) = coord.right(self.size) {
                if let Some(right_index) = square_indexes.get(&right) {
                    prepared.right = Some(*right_index);
                }
            }
            if let Some(above) = coord.above() {
                if let Some(above_index) = square_indexes.get(&above) {
                    prepared.above = Some(*above_index);
                }
            }
            if let Some(below) = coord.below(self.size) {
                if let Some(below_index) = square_indexes.get(&below) {
                    prepared.below = Some(*below_index);
                }
            }
            prepared_squares.push(prepared);
        }
        let off_by_one = self
            .rules
            .iter()
            .any(|rule| matches!(rule, Rule::NumbersAreOffByOne));

        let mut rules = Vec::new();
        for rule in &self.rules {
            self.prepare_rule(rule, &square_indexes, off_by_one, &mut rules);
        }
        for (coord, square) in self.squares() {
            self.prepare_square(&square, coord, &square_indexes, &mut rules);
        }

        PreparedGrid {
            squares: prepared_squares,
            rules,
            square_indexes,
            size: self.size,
        }
    }
}
