"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const fs = __importStar(require("fs"));
const htmlPath = './tarpaulin-report.html';
const trackerPath = './docs/unit_test_tracker.md';
function extractCoverageData(html) {
    const match = html.match(/var data = (\{[\s\S]*?\});/);
    if (!match)
        throw new Error('data JSON not found');
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
function updateCoverageInTracker(tracker, percent) {
    return tracker.replace(/(Current Unit Test Coverage:)[^\n]*/, `$1 ${percent}`);
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
