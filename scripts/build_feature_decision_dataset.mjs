import fs from "node:fs/promises";
import path from "node:path";
import { SpreadsheetFile, Workbook } from "@oai/artifact-tool";

const root = "/Users/sungjinkim/Data/github_clone/rustcoding";
const outputDir = path.join(root, "outputs", "feature_decision_dataset");
const outputPath = path.join(outputDir, "feature_decision_dataset.xlsx");

const workbook = Workbook.create();
const sheet = workbook.worksheets.add("Dataset");

const headers = [["Record ID", "Feature 1", "Feature 2", "Decision"]];
const rows = Array.from({ length: 100 }, (_, index) => {
  const recordId = index + 1;
  const feature1 = Number((12 + Math.sin(recordId / 5) * 4 + recordId * 0.37).toFixed(2));
  const feature2 = Number((24 + Math.cos(recordId / 7) * 6 + recordId * 0.21).toFixed(2));
  const decision = feature1 * 0.62 + feature2 * 0.38 >= 27 ? 1 : 0;
  return [recordId, feature1, feature2, decision];
});

sheet.getRange("A1:D1").values = headers;
sheet.getRange("A2:D101").values = rows;

sheet.getRange("A1:D1").format = {
  fontWeight: "bold",
  fill: { color: "#1F4E78" },
  fontColor: "#FFFFFF",
  horizontalAlignment: "center",
};
sheet.getRange("A2:A101").numberFormat = "0";
sheet.getRange("B2:C101").numberFormat = "0.00";
sheet.getRange("D2:D101").numberFormat = "0";
sheet.getRange("A:D").columnWidthPx = 110;
sheet.getRange("A1:D101").border = {
  bottom: { style: "thin", color: "#D9E2F3" },
};
const dataCheck = await workbook.inspect({
  kind: "table",
  range: "Dataset!A1:D101",
  include: "values",
  tableMaxRows: 105,
  tableMaxCols: 4,
});
console.log(dataCheck.ndjson);

const errors = await workbook.inspect({
  kind: "match",
  searchTerm: "#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A",
  options: { useRegex: true, maxResults: 50 },
  summary: "final formula error scan",
});
console.log(errors.ndjson);

await workbook.render({ sheetName: "Dataset", range: "A1:D25", scale: 2 });

await fs.mkdir(outputDir, { recursive: true });
const output = await SpreadsheetFile.exportXlsx(workbook);
await output.save(outputPath);
console.log(`Saved ${outputPath}`);
