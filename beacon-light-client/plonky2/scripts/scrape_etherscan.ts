
import { parse } from 'node-html-parser';

(async () => {
  let page = 1;
  while (true) {
    const response = await fetch(`https://etherscan.io/blocks_forked?p=${page}`);
    const text = await response.text();
    const root = parse(text);
    const entries = root.querySelectorAll('tbody > tr');
    const depths = entries.map(row => ({ slot: row?.querySelector('td:first-of-type')?.innerText, depth: row?.querySelector('td:last-of-type')?.innerText }));

    const filtered = depths.filter(entry => entry.depth != '1');
    if (filtered.length > 0) {
      console.log(filtered);
    }

    page += 1;
  }
})();
