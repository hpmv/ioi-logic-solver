import logo from './logo.svg';
import './App.css';
import { Grid, prepareGrid } from './Grid';
import { Fragment, useEffect, useState } from 'react';

function App() {
  const [decoded, setDecoded] = useState(null);
  useEffect(() => {
    fetch('/decoded.json').then(response => response.json()).then(data => {
      let sorted = data;
      sorted.sort((a, b) => a.cols * a.rows - b.cols * b.rows || a.rows - b.rows || a.cols - b.cols || a.difficulty - b.difficulty || ruleMask(a) - ruleMask(b));
      // sorted = sorted.slice(0, 100);

      const bySize = [];
      for (let i = 0; i < sorted.length; i++) {
        if (i === 0 || sorted[i].cols !== sorted[i - 1].cols || sorted[i].rows !== sorted[i - 1].rows) {
          bySize.push([]);
        }
        bySize[bySize.length - 1].push(prepareGrid(sorted[i]));
      }

      setDecoded(bySize);
    });
  }, []);
  return decoded ? <div className="groups">
    {decoded.map((group, index) => <div key={index} className="group">
      <div className="group-heading">{group[0].rows} x {group[0].cols}</div>
      <div className="group-grids">
        {group.map((grid, index) => <Grid key={index} grid={grid} />)}
      </div>
    </div>)}
  </div> : <div>Loading...</div>;
}

export default App;

function ruleMask(grid) {
  let lights = 0;
  let darks = 0;
  let mask = 0;

  for (const rule of grid.rules) {
    if (rule[0] == 'light') {
      lights = rule[1].length;
    } else if (rule[0] == 'dark') {
      darks = rule[1].length;
    } else if (rule[0] == 'area') {
      mask += 1 << 0;
    } else if (rule[0] == 'viewpoint') {
      mask += 1 << 3;
    } else if (rule[0] == 'dart') {
      mask += 1 << 4;
    } else if (rule[0] == 'myopia') {
      mask += 1 << 7;
    } else if (rule[0] == 'galaxy') {
      mask += 1 << 5;
    } else if (rule[0] == 'lotus') {
      mask += 1 << 6;
    } else if (rule[0] == 'letters') {
      mask += 1 << 2;
    } else if (rule[0] == 'ban_patterns') {
      mask += 1 << 1;
    }
  }
  return lights * (1 << 20) + darks * (1 << 10) + mask;
}