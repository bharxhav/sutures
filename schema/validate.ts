import Ajv from "ajv/dist/2020";
import { readdir, readFile } from "fs/promises";

// ═══════════════════════════════════════════════════════════════════════
// UTILS — diagnostics, display, constants
// ═══════════════════════════════════════════════════════════════════════

// ── Diagnostics ──────────────────────────────────────────────────────

interface Diagnostic {
  file: string;
  rule: string;
  message: string;
}

const diagnostics: Diagnostic[] = [];

function fail(file: string, rule: string, message: string) {
  diagnostics.push({ file, rule, message });
}

// ── Display ──────────────────────────────────────────────────────────

const ci = !!process.env.CI;
const RED = ci ? "" : "\x1b[31m";
const GREEN = ci ? "" : "\x1b[32m";
const DIM = ci ? "" : "\x1b[2m";
const BOLD = ci ? "" : "\x1b[1m";
const RESET = ci ? "" : "\x1b[0m";

function printDiagnostics() {
  const grouped = new Map<string, Diagnostic[]>();
  for (const d of diagnostics) {
    const list = grouped.get(d.file) ?? [];
    list.push(d);
    grouped.set(d.file, list);
  }

  for (const [file, diags] of grouped) {
    console.error(`\n${BOLD}${file}${RESET}`);
    for (const d of diags) {
      console.error(`  ${RED}>>  ${d.rule}${RESET}  ${d.message}`);
    }
  }
}

// ── Constants ────────────────────────────────────────────────────────

const KNOWN_DIALECTS = new Set([
  "https://json-schema.org/draft/2020-12/schema",
  "https://json-schema.org/draft/2019-09/schema",
  "http://json-schema.org/draft-07/schema#",
]);

const ID_PATTERN = /^https:\/\/schema\.sutures\.dev\/v(\d+)\.json$/;

// ── Check functions ──────────────────────────────────────────────────

function checkBom(file: string, raw: string) {
  if (raw.charCodeAt(0) === 0xfeff) {
    fail(file, "no-bom", "File contains a UTF-8 BOM — remove it");
  }
}

function checkNoTrailingWhitespace(file: string, raw: string) {
  const lines = raw.split("\n");
  for (let i = 0; i < lines.length; i++) {
    if (/\s+$/.test(lines[i]) && lines[i].trim().length > 0) {
      fail(
        file,
        "trailing-whitespace",
        `Line ${i + 1} has trailing whitespace`,
      );
      return;
    }
  }
}

function checkJsonParse(file: string, raw: string): unknown | null {
  try {
    return JSON.parse(raw);
  } catch (e: any) {
    const match = e.message.match(/position (\d+)/);
    const pos = match ? ` at byte ${match[1]}` : "";
    fail(file, "json-parse", `Invalid JSON${pos}: ${e.message}`);
    return null;
  }
}

function checkIsObject(
  file: string,
  schema: unknown,
): schema is Record<string, unknown> {
  if (typeof schema !== "object" || schema === null || Array.isArray(schema)) {
    fail(file, "schema-type", "Schema root must be an object");
    return false;
  }
  return true;
}

function checkDialect(file: string, schema: Record<string, unknown>) {
  const dialect = schema["$schema"];
  if (typeof dialect !== "string") {
    fail(file, "$schema-required", "Missing $schema field");
    return;
  }
  if (!KNOWN_DIALECTS.has(dialect)) {
    fail(file, "$schema-known", `Unrecognized dialect: ${dialect}`);
  }
}

function checkId(file: string, schema: Record<string, unknown>) {
  const id = schema["$id"];
  if (typeof id !== "string") {
    fail(file, "$id-required", "Missing $id field");
    return;
  }
  if (!id.startsWith("https://")) {
    fail(file, "$id-https", `$id must use https:// — got: ${id}`);
  }
  const match = id.match(ID_PATTERN);
  if (!match) {
    fail(
      file,
      "$id-format",
      `$id must match https://schema.sutures.dev/v{N}.json — got: ${id}`,
    );
    return;
  }
  if (`v${match[1]}.json` !== file) {
    fail(
      file,
      "$id-version-mismatch",
      `$id says v${match[1]}.json but file is ${file}`,
    );
  }
}

function checkTitle(file: string, schema: Record<string, unknown>) {
  if (typeof schema["title"] !== "string" || schema["title"].length === 0) {
    fail(file, "title-required", "Schema must have a non-empty title");
  }
}

function checkAjv(file: string, schema: Record<string, unknown>) {
  try {
    const ajv = new Ajv({ strict: true, allErrors: true });
    ajv.addKeyword("version");
    const valid = ajv.validateSchema(schema);
    if (!valid && ajv.errors) {
      for (const err of ajv.errors) {
        const path = err.instancePath || "/";
        const params = err.params ? ` ${JSON.stringify(err.params)}` : "";
        fail(file, "schema-invalid", `${path}: ${err.message}${params}`);
      }
    }
  } catch (e: any) {
    fail(file, "schema-invalid", e.message);
  }
}

// ═══════════════════════════════════════════════════════════════════════
// 1. DISCOVER — find all versioned schema files (v1.json, v2.json, ...)
// ═══════════════════════════════════════════════════════════════════════

const files = (await readdir(".", { withFileTypes: true }))
  .filter((d) => d.isFile() && /^v\d+\.json$/.test(d.name))
  .map((d) => d.name)
  .sort((a, b) => parseInt(a.slice(1)) - parseInt(b.slice(1)));

if (files.length === 0) {
  console.error(
    `${RED}>>  No versioned schema files found (v1.json, v2.json, ...)${RESET}`,
  );
  process.exit(1);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. VALIDATE — run checks on each schema, collecting all diagnostics
// ═══════════════════════════════════════════════════════════════════════

let checked = 0;

for (const file of files) {
  const raw = await readFile(file, "utf-8");

  // 2.2 raw text checks (bail early if json is broken)
  checkBom(file, raw);
  checkNoTrailingWhitespace(file, raw);

  const parsed = checkJsonParse(file, raw);
  if (parsed === null) continue;
  if (!checkIsObject(file, parsed)) continue;

  // 2.3 metadata checks
  checkDialect(file, parsed);
  checkId(file, parsed);
  checkTitle(file, parsed);

  // 2.4 schema validation (ajv strict mode)
  checkAjv(file, parsed);

  checked++;
}

// ═══════════════════════════════════════════════════════════════════════
// 3. REPORT — print results, exit 1 on any failure
// ═══════════════════════════════════════════════════════════════════════

if (diagnostics.length > 0) {
  printDiagnostics();
  const affected = new Set(diagnostics.map((d) => d.file)).size;
  console.error(
    `\n${RED}${BOLD}${diagnostics.length} problem${diagnostics.length === 1 ? "" : "s"} in ${affected} file${affected === 1 ? "" : "s"}${RESET}\n`,
  );
  process.exit(1);
}

console.log(
  `${GREEN}${BOLD}${checked} schema${checked === 1 ? "" : "s"} validated${RESET} ${DIM}(${files.join(", ")})${RESET}`,
);
