import React from 'react';
import './Grid.css';
import { CONNECT_ALL_DARK_PATTERN, CONNECT_ALL_LIGHT_PATTERN, PatternGrid } from './Pattern';

export const Grid = ({ grid }) => {
    const [showSolution, setShowSolution] = React.useState(false);
    return <div className='grid'>
        <div className="grid-cells" style={{ width: 24 * grid.cols }}>
            {grid.cells.map((cell, index) => {
                return <GridCell key={index} cell={cell} showSolution={showSolution} />
            })}
        </div>
        {grid.rules.length > 0 && <div className="rules">
            {grid.rules.map((rule, index) => {
                let tooltip = '';
                let element = <>{JSON.stringify(rule)}</>;
                if (rule[0] == 'ban_pattern') {
                    tooltip = 'This pattern is banned';
                    const pattern = rule[1];
                    element = <PatternGrid pattern={{ rows: pattern[0], cols: pattern[1], cells: pattern[3] }} crossOut={true} gap={true} />;
                } else if (rule[0] == 'connect_all_dark') {
                    tooltip = 'Connect all dark cells';
                    element = <PatternGrid pattern={CONNECT_ALL_DARK_PATTERN} crossOut={false} gap={false} />;
                } else if (rule[0] == 'connect_all_light') {
                    tooltip = 'Connect all light cells';
                    element = <PatternGrid pattern={CONNECT_ALL_LIGHT_PATTERN} crossOut={false} gap={false} />;
                } else if (rule[0] == 'dark_area') {
                    tooltip = `All dark regions have area ${rule[1]}`;
                    element = <div className="dark-area">{rule[1]}</div>;
                } else if (rule[0] == 'light_area') {
                    tooltip = `All light regions have area ${rule[1]}`;
                    element = <div className="light-area">{rule[1]}</div>;
                } else if (rule[0] == 'one_symbol_per_light') {
                    tooltip = 'Exactly one symbol per light region';
                    element = <div className="one-symbol"><div className="light-area">X</div></div>;
                } else if (rule[0] == 'one_symbol_per_dark') {
                    tooltip = 'Exactly one symbol per dark region';
                    element = <div className="one-symbol"><div className="dark-area">X</div></div>;
                } else if (rule[0] == 'Underconstrained Grid') {
                    tooltip = 'Underconstrained Grid';
                    element = <div className="light-area">?</div>;
                } else if (rule[0] == 'light_shapes_distinct') {
                    tooltip = 'All light regions have different shapes and areas';
                    element = <div className="light-area">≠</div>;
                } else if (rule[0] == 'dark_shapes_distinct') {
                    tooltip = 'All dark regions have different shapes and areas';
                    element = <div className="dark-area">≠</div>;
                } else if (rule[0] == 'light_shapes_same') {
                    tooltip = 'All light regions have the same shapes';
                    element = <div className="light-area">=</div>;
                } else if (rule[0] == 'dark_shapes_same') {
                    tooltip = 'All dark regions have the same shapes';
                    element = <div className="dark-area">=</div>;
                }
                return <div key={index} className="rule" title={tooltip}>{element}</div>
            })}
        </div>
        }
        <div className="puzzle-header">
            <div className="puzzle-id">{'#' + grid.pid}</div>
            <div className="puzzle-difficulty">
                {grid.difficulty <= 5 ? Array.from({ length: grid.difficulty }).map((_, index) => <div key={index} className="orb">⬤</div>) :
                    Array.from({ length: grid.difficulty - 5 }).map((_, index) => <div key={index} className="star">★</div>)}
            </div>
            <div className={`peek-button ${showSolution ? 'peeking' : 'hiding'}`} onClick={() => setShowSolution(v => !v)}>🫣</div>
        </div>
    </div>
};

const GridCell = ({ cell, showSolution }) => {
    const color = showSolution ? cell.solution : cell.color;
    return <div className={
        ['grid-cell',
            color == 0 ? 'light' : color == 1 ? 'dark' : 'empty',
            cell.exists ? '' : 'hole',
            cell.merges.above ? 'merge-above' : '',
            cell.merges.left ? 'merge-left' : '',
            cell.merges.right ? 'merge-right' : '',
            cell.merges.below ? 'merge-below' : ''].join(' ')
    }>
        {cell.viewpoint != null ? <div className="viewpoint">
            <div className="arrow up"></div>
            <div className="arrow left"></div>
            <div className="arrow down"></div>
            <div className="arrow right"></div>
            <div className="viewpoint-number">{cell.viewpoint}</div>
        </div> : null}
        {cell.area != null ? <div className="area-number">{cell.area}</div> : null}
        {cell.dart != null ? <div className={`dart-number-${cell.dart[1]}`}>{cell.dart[0]}</div> : null}
        {cell.letter != null ? <div className="letter">{cell.letter}</div> : null}
        {cell.galaxies.map(([r, c]) => <div className="galaxy" style={{ top: 12 * r, left: 12 * c }}>⦾</div>)}
        {cell.lotuses.map(([r, c, dir]) => <div className={`lotus lotus-${dir}`} style={{ top: 12 * r, left: 12 * c }}>♡</div>)}
        {cell.myopia != null ? <div className='myopia'>
            {(cell.myopia & 1) > 0 && <div className='myopia-up'>🡒</div>}
            {(cell.myopia & 2) > 0 && <div className='myopia-down'>🡒</div>}
            {(cell.myopia & 4) > 0 && <div className='myopia-left'>🡒</div>}
            {(cell.myopia & 8) > 0 && <div className='myopia-right'>🡒</div>}
        </div> : null}
    </div>
};

export function prepareGrid(grid) {
    const cells = [];
    const rules = [];
    for (let i = 0; i < grid.rows; i++) {
        for (let j = 0; j < grid.cols; j++) {
            cells.push({
                row: i, col: j,
                mergeRight: false,
                mergeDown: false,
                exists: true,
                color: 2,
                solution: 2,
                area: null, viewpoint: null, dart: null, myopia: null,
                galaxies: [],
                lotuses: [],
                letter: null,
                merges: {
                    above: false,
                    left: false,
                    right: false,
                    below: false,
                },
            });
        }
    }
    for (const topology of grid.topology) {
        if (topology[0] == 'merge') {
            for (const edge of topology[1]) {
                const edge_id = (edge - edge % 2) / 2;
                const row = Math.floor((edge_id - grid.cols) / (grid.cols + grid.cols + 1));
                const edge_of_row = edge_id - grid.cols - row * (grid.cols + grid.cols + 1);
                if (edge_of_row < grid.cols + 1) {
                    const col = edge_of_row - 1;
                    cells[row * grid.cols + col].mergeRight = true;
                } else {
                    const col = edge_of_row - grid.cols - 1;
                    cells[row * grid.cols + col].mergeDown = true;
                }
                console.log({ edge, edge_id, row, edge_of_row })
            }
        } else if (topology[0] == 'hole') {
            for (const cell of topology[1]) {
                cells[cell].exists = false;
            }
        }
    }

    for (let i = 0; i < grid.rows * grid.cols; i++) {
        const cell = cells[i];
        const left = i % grid.cols > 0 ? cells[i - 1].exists && cells[i - 1].mergeRight : false;
        const right = cell.mergeRight;
        const above = i >= grid.cols ? cells[i - grid.cols].exists && cells[i - grid.cols].mergeDown : false;
        const below = cell.mergeDown;
        cell.merges = { above, left, right, below };
    }

    for (const rule of grid.rules) {
        if (rule[0] == 'light') {
            for (const cell of rule[1]) {
                cells[cell].color = 0;
            }
        } else if (rule[0] == 'dark') {
            for (const cell of rule[1]) {
                cells[cell].color = 1;
            }
        } else if (rule[0] == 'area') {
            for (const [cell, num] of rule[1]) {
                cells[cell].area = num;
            }
        } else if (rule[0] == 'viewpoint') {
            for (const [cell, num] of rule[1]) {
                cells[cell].viewpoint = num;
            }
        } else if (rule[0] == 'dart') {
            for (const [cell, num, dir] of rule[1]) {
                cells[cell].dart = [num, dir];
            }
        } else if (rule[0] == 'myopia') {
            for (const [cell, dir] of rule[1]) {
                cells[cell].myopia = dir;
            }
        } else if (rule[0] == 'galaxy') {
            for (const loc of rule[1]) {
                const row = Math.floor(loc / (grid.cols * 2 + 1));
                const col = loc - row * (grid.cols * 2 + 1);
                const cellRow = Math.floor(row / 2);
                const cellCol = Math.floor(col / 2);
                cells[cellRow * grid.cols + cellCol].galaxies.push([row - cellRow * 2, col - cellCol * 2]);
            }
        } else if (rule[0] == 'lotus') {
            for (const [loc, dir] of rule[1]) {
                const row = Math.floor(loc / (grid.cols * 2 + 1));
                const col = loc - row * (grid.cols * 2 + 1);
                const cellRow = Math.floor(row / 2);
                const cellCol = Math.floor(col / 2);
                cells[cellRow * grid.cols + cellCol].lotuses.push([row - cellRow * 2, col - cellCol * 2, dir]);
            }
        } else if (rule[0] == 'letters') {
            for (const [cell, letter] of rule[1]) {
                cells[cell].letter = String.fromCharCode(letter + 65);
            }
        } else if (rule[0] == 'ban_patterns') {
            for (const pattern of rule[1]) {
                rules.push(['ban_pattern', pattern]);
            }
        } else {
            rules.push(rule);
        }
    }
    if (grid.solution != null) {
        if (grid.solution[0] == 2) {
            rules.push(['Underconstrained Grid']);
            for (let i = 0; i < grid.rows * grid.cols; i++) {
                cells[i].solution = grid.solution[2][i];
            }
        } else {
            for (let i = 0; i < grid.rows * grid.cols; i++) {
                cells[i].solution = grid.solution[2][i];
            }
        }
    }
    // console.log({ grid, cells, rules })
    return { pid: grid.pid, difficulty: grid.difficulty, rows: grid.rows, cols: grid.cols, cells, rules };
}
