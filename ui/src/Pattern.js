import React from 'react';
import './Pattern.css';

export const PatternGrid = ({ pattern, crossOut, gap }) => {
    const cellWidth = gap ? 12 : 6;
    return <div className="pattern" style={{ width: cellWidth * pattern.cols + 4 }}>
        <div className={`pattern-grid ${gap ? 'with-gap' : ''}`}>
            {pattern.cells.map((cell, index) => <PatternCell key={index} cell={cell} />)}
        </div>
        {crossOut && <div className="cross-out">
            <div className="cross-out-left"></div>
            <div className="cross-out-right"></div>
        </div>}
    </div>;
};

const PatternCell = ({ cell }) => {
    return <div className={
        ['pattern-cell',
            cell == 0 ? 'light' : cell == 1 ? 'dark' : 'empty',
        ].join(' ')
    }></div>;
};

export const CONNECT_ALL_DARK_PATTERN = {
    cols: 5,
    height: 5,
    cells: [
        1, 1, 1, 1, 2,
        2, 2, 2, 1, 2,
        2, 1, 1, 1, 2,
        2, 1, 2, 2, 2,
        2, 1, 1, 1, 1,
    ]
};

export const CONNECT_ALL_LIGHT_PATTERN = {
    cols: 5,
    height: 5,
    cells: [
        0, 0, 0, 0, 2,
        2, 2, 2, 0, 2,
        2, 0, 0, 0, 2,
        2, 0, 2, 2, 2,
        2, 0, 0, 0, 0,
    ]
};