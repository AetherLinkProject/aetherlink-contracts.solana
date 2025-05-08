import * as fs from 'fs';

const htmlPath = './tarpaulin-report.html';
const trackerPath = './docs/unit_test_tracker.md';

function extractCoverageData(html: string): { covered: number, coverable: number } {
  const match = html.match(/var data = (\{[\s\S]*?\});/);
  if (!match) throw new Error('data JSON not found');
  const data = JSON.parse(match[1]);
  let covered = 0, coverable = 0;
  for (const file of data.files) {
    if (typeof file.covered === 'number' && typeof file.coverable === 'number') {
      covered += file.covered;
      coverable += file.coverable;
    }
  }
  return { covered, coverable };
}

function updateCoverageInTracker(tracker: string, percent: string): string {
  return tracker.replace(
    /(Current Unit Test Coverage:)[^\n]*/,
    `$1 ${percent}`
  );
}

function main() {
  const html = fs.readFileSync(htmlPath, 'utf8');
  const { covered, coverable } = extractCoverageData(html);
  const percent = coverable > 0 ? `${((covered / coverable) * 100).toFixed(2)}%` : '0%';
  const tracker = fs.readFileSync(trackerPath, 'utf8');
  const updated = updateCoverageInTracker(tracker, percent);
  fs.writeFileSync(trackerPath, updated, 'utf8');
  console.log(`Coverage updated: ${percent}`);
}

main(); 