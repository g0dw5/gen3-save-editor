/** Reuse the application's pure form resolver and translations in the reviewer. */
import fs from "node:fs";
import ts from "typescript";
const code = ts.transpileModule(
  fs.readFileSync(new URL("../ui/speciesForms.ts", import.meta.url), "utf8"),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ES2022,
    },
  },
).outputText;
const source = ts.createSourceFile(
  "i18n.tsx",
  fs.readFileSync(new URL("../ui/i18n.tsx", import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
  ts.ScriptKind.TSX,
);
const labels = {};
for (const statement of source.statements) {
  if (!ts.isVariableStatement(statement)) continue;
  for (const declaration of statement.declarationList.declarations) {
    if (
      declaration.name.getText(source) !== "zh" ||
      !ts.isObjectLiteralExpression(declaration.initializer)
    )
      continue;
    for (const property of declaration.initializer.properties) {
      if (
        ts.isPropertyAssignment(property) &&
        ts.isStringLiteral(property.initializer)
      ) {
        labels[property.name.getText(source).replace(/^['"]|['"]$/g, "")] =
          property.initializer.text;
      }
    }
  }
}
process.stdout.write(
  code + "\nexport const formText = " + JSON.stringify(labels) + ";\n",
);
