import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const lab = join(here, "..");
const materialsPath = join(lab, "materials", "rock-materials.json");
const appPath = join(lab, "prototype", "app.js");
const htmlPath = join(lab, "prototype", "index.html");

function readMaterials() {
  return JSON.parse(readFileSync(materialsPath, "utf8"));
}

test("o laboratório mantém as três famílias exigidas", () => {
  const materials = readMaterials().materials;
  assert.deepEqual(materials.map((item) => item.id), [
    "nexora:rock_natural_prototype",
    "nexora:rock_granite_prototype",
    "nexora:rock_eroded_prototype",
  ]);
});

test("cada família possui parâmetros reproduzíveis e features", () => {
  const materials = readMaterials().materials;
  for (const material of materials) {
    assert.ok(Array.isArray(material.baseColor) && material.baseColor.length === 3);
    assert.ok(material.roughness >= 0 && material.roughness <= 1);
    assert.ok(material.detailScale >= 0 && material.detailScale <= 1);
    assert.ok(material.erosion >= 0 && material.erosion <= 1);
    assert.ok(material.mineralVariation >= 0 && material.mineralVariation <= 1);
    assert.ok(material.features.length >= 3);
  }
});

test("os três métodos estão registrados com perfis distintos", () => {
  const methods = readMaterials().methods;
  assert.deepEqual(methods.map((item) => item.id), ["texture", "procedural", "hybrid"]);
  assert.notEqual(methods[0].runtimeMemoryMb, methods[1].runtimeMemoryMb);
  assert.equal(methods[1].shaderLayers, 8);
  assert.equal(methods[2].shaderLayers, 5);
});

test("a interface permanece autocontida e sem asset externo", () => {
  const app = readFileSync(appPath, "utf8");
  const html = readFileSync(htmlPath, "utf8");
  assert.ok(existsSync(join(lab, "documentation", "Rock Material Experiment.md")));
  assert.ok(existsSync(join(lab, "shaders", "rock-material-lab.glsl")));
  assert.match(app, /class Random/);
  assert.match(app, /function fbm/);
  assert.match(app, /rockType === "eroded"/);
  assert.match(html, /sem textura externa/i);
  assert.doesNotMatch(app, /https?:\/\//);
});
