const state = {
  seed: 1847,
  detail: 58,
  roughness: 66,
  erosion: 34,
  mineral: 48,
  method: "hybrid",
  distance: "mid",
};

const ROCKS = {
  natural: {
    label: "Pedra natural",
    base: [137, 112, 88],
    light: [190, 164, 127],
    dark: [54, 49, 43],
    accent: [119, 137, 125],
  },
  granite: {
    label: "Granito compacto",
    base: [126, 134, 133],
    light: [189, 193, 187],
    dark: [48, 54, 55],
    accent: [204, 174, 130],
  },
  eroded: {
    label: "Pedra erodida",
    base: [130, 111, 92],
    light: [185, 163, 133],
    dark: [52, 47, 42],
    accent: [112, 126, 112],
  },
};

const DISTANCES = {
  close: { scale: 1.07, detail: 1, label: "CLOSE" },
  mid: { scale: 0.83, detail: 0.66, label: "MÉDIO" },
  far: { scale: 0.57, detail: 0.25, label: "DISTANTE" },
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

class Random {
  constructor(seed) {
    this.value = (seed >>> 0) || 1;
  }

  next() {
    let x = this.value;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this.value = x >>> 0;
    return (this.value >>> 0) / 4294967296;
  }

  range(min, max) {
    return min + (max - min) * this.next();
  }

  pick(items) {
    return items[Math.floor(this.next() * items.length)];
  }
}

function mix(a, b, amount) {
  return a + (b - a) * amount;
}

function smooth(t) {
  return t * t * (3 - 2 * t);
}

function hash2(x, y, seed) {
  let value = Math.sin(x * 127.1 + y * 311.7 + seed * 74.3) * 43758.5453;
  return value - Math.floor(value);
}

function valueNoise(x, y, seed) {
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const tx = smooth(x - x0);
  const ty = smooth(y - y0);
  const a = hash2(x0, y0, seed);
  const b = hash2(x0 + 1, y0, seed);
  const c = hash2(x0, y0 + 1, seed);
  const d = hash2(x0 + 1, y0 + 1, seed);
  return mix(mix(a, b, tx), mix(c, d, tx), ty);
}

function fbm(x, y, seed, octaves = 4) {
  let amplitude = 0.55;
  let frequency = 1;
  let total = 0;
  let normalization = 0;
  for (let i = 0; i < octaves; i += 1) {
    total += valueNoise(x * frequency, y * frequency, seed + i * 19.17) * amplitude;
    normalization += amplitude;
    amplitude *= 0.5;
    frequency *= 2.02;
  }
  return total / normalization;
}

function color(rgb, alpha = 1) {
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

function lighten(rgb, amount) {
  return rgb.map((channel) => Math.round(mix(channel, 255, amount)));
}

function darken(rgb, amount) {
  return rgb.map((channel) => Math.round(mix(channel, 0, amount)));
}

function polygonPath(ctx, points) {
  ctx.beginPath();
  const first = points[0];
  const last = points[points.length - 1];
  ctx.moveTo((last.x + first.x) / 2, (last.y + first.y) / 2);
  points.forEach((point, index) => {
    const next = points[(index + 1) % points.length];
    const midpoint = { x: (point.x + next.x) / 2, y: (point.y + next.y) / 2 };
    ctx.quadraticCurveTo(point.x, point.y, midpoint.x, midpoint.y);
  });
  ctx.closePath();
}

function buildOutline(rockType, random, roughness, scale) {
  const count = rockType === "eroded" ? 21 : 18;
  const points = [];
  const bias = rockType === "granite" ? 0.94 : 1;
  for (let i = 0; i < count; i += 1) {
    const angle = (Math.PI * 2 * i) / count;
    const harmonic = Math.sin(angle * 3 + random.range(-0.45, 0.45)) * 0.045;
    const noise = random.range(-0.075, 0.075) * (roughness / 100);
    const radius = (0.89 + noise + harmonic) * scale;
    points.push({
      x: Math.cos(angle) * radius * bias,
      y: Math.sin(angle) * radius,
    });
  }
  return points;
}

function drawBackground(ctx, width, height) {
  const background = ctx.createRadialGradient(width * 0.5, height * 0.24, 4, width * 0.5, height * 0.5, width * 0.75);
  background.addColorStop(0, "#272a25");
  background.addColorStop(0.45, "#171a18");
  background.addColorStop(1, "#0d100f");
  ctx.fillStyle = background;
  ctx.fillRect(0, 0, width, height);

  const floor = ctx.createLinearGradient(0, height * 0.63, 0, height);
  floor.addColorStop(0, "rgba(10, 13, 12, 0)");
  floor.addColorStop(1, "rgba(4, 5, 5, .55)");
  ctx.fillStyle = floor;
  ctx.fillRect(0, 0, width, height);

  ctx.strokeStyle = "rgba(233, 230, 220, .035)";
  ctx.lineWidth = 1;
  for (let x = 0; x < width; x += 48) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }
  for (let y = 0; y < height; y += 48) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }
}

function drawShadow(ctx, centerX, centerY, rockScale) {
  ctx.save();
  ctx.translate(centerX + 16, centerY + 130 * rockScale);
  ctx.scale(1.25 * rockScale, 0.18 * rockScale);
  const shadow = ctx.createRadialGradient(0, 0, 2, 0, 0, 170);
  shadow.addColorStop(0, "rgba(0, 0, 0, .58)");
  shadow.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = shadow;
  ctx.beginPath();
  ctx.arc(0, 0, 170, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

function drawPatches(ctx, points, palette, random, rockScale, detail, type, seed, method) {
  ctx.save();
  polygonPath(ctx, points);
  ctx.clip();
  const patchCount = Math.round((method === "texture" ? 10 : 15) * detail);
  for (let i = 0; i < patchCount; i += 1) {
    const x = random.range(-210, 210) * rockScale;
    const y = random.range(-155, 145) * rockScale;
    const radius = random.range(32, 100) * rockScale * (0.6 + detail * 0.4);
    const gradient = ctx.createRadialGradient(x - radius * 0.35, y - radius * 0.45, 0, x, y, radius);
    const patchColor = i % 4 === 0 ? palette.light : i % 3 === 0 ? palette.dark : palette.accent;
    gradient.addColorStop(0, color(patchColor, type === "granite" ? 0.31 : type === "eroded" ? 0.29 : 0.25));
    gradient.addColorStop(0.7, color(patchColor, 0.045));
    gradient.addColorStop(1, color(patchColor, 0));
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();
  }

  const grainCount = Math.round((method === "texture" ? 1050 : 1650) * detail);
  for (let i = 0; i < grainCount; i += 1) {
    const x = random.range(-235, 235) * rockScale;
    const y = random.range(-175, 175) * rockScale;
    const field = fbm(x / (72 * rockScale), y / (72 * rockScale), seed, method === "procedural" ? 5 : 3);
    const size = random.range(0.75, 2.85) * Math.max(0.6, rockScale);
    const alpha = (0.06 + field * 0.2) * (0.65 + detail * 0.55);
    const grainColor = field > 0.58 ? palette.light : palette.dark;
    ctx.fillStyle = color(grainColor, alpha);
    ctx.fillRect(x, y, size, size);
  }
  ctx.restore();
}

function drawMinerals(ctx, points, palette, random, rockScale, detail, mineral, type, method) {
  if (type === "natural" && method === "texture") return;
  ctx.save();
  polygonPath(ctx, points);
  ctx.clip();
  const count = Math.round((type === "granite" ? 135 : 34) * (0.4 + mineral / 100) * detail);
  for (let i = 0; i < count; i += 1) {
    const x = random.range(-220, 220) * rockScale;
    const y = random.range(-160, 160) * rockScale;
    const radius = random.range(0.7, type === "granite" ? 3.5 : 2.1) * rockScale;
    const mineralColor = random.next() > 0.62 ? palette.light : palette.accent;
    ctx.fillStyle = color(mineralColor, random.range(0.16, 0.58));
    ctx.beginPath();
    ctx.ellipse(x, y, radius * random.range(0.7, 1.9), radius, random.range(0, Math.PI), 0, Math.PI * 2);
    ctx.fill();
    if (type === "granite" && radius > 1.8 * rockScale) {
      ctx.strokeStyle = color(palette.dark, 0.18);
      ctx.lineWidth = Math.max(0.4, rockScale);
      ctx.stroke();
    }
  }
  ctx.restore();
}

function drawCavities(ctx, points, palette, random, rockScale, erosion, detail, type) {
  if (type !== "eroded") return;
  ctx.save();
  polygonPath(ctx, points);
  ctx.clip();
  const count = Math.round((10 + erosion / 5.5) * detail);
  for (let i = 0; i < count; i += 1) {
    const x = random.range(-200, 205) * rockScale;
    const y = random.range(-142, 135) * rockScale;
    const radius = random.range(6, 22) * rockScale * (0.65 + erosion / 130);
    const cavity = ctx.createRadialGradient(x - radius * 0.32, y - radius * 0.35, 0, x, y, radius);
    cavity.addColorStop(0, color(darken(palette.dark, 0.12), 0.88));
    cavity.addColorStop(0.42, color(palette.dark, 0.6));
    cavity.addColorStop(0.74, color(palette.light, 0.22));
    cavity.addColorStop(1, color(palette.light, 0));
    ctx.fillStyle = cavity;
    ctx.beginPath();
    ctx.ellipse(x, y, radius * random.range(0.7, 1.5), radius * random.range(0.55, 1), random.range(-0.8, 0.8), 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();
}

function drawStrata(ctx, points, palette, random, rockScale, detail, type, method) {
  if (type === "granite" || method === "texture") return;
  ctx.save();
  polygonPath(ctx, points);
  ctx.clip();
  const lineCount = type === "eroded" ? 7 : 4;
  ctx.lineCap = "round";
  for (let i = 0; i < lineCount; i += 1) {
    const y = random.range(-140, 140) * rockScale;
    ctx.beginPath();
    ctx.moveTo(-230 * rockScale, y);
    for (let x = -180; x <= 220; x += 45) {
      ctx.lineTo(x * rockScale, y + Math.sin(x * 0.035 + i) * random.range(4, 13) * rockScale);
    }
    ctx.strokeStyle = color(i % 2 ? palette.dark : palette.light, 0.12 + detail * 0.08);
    ctx.lineWidth = random.range(1, 3) * rockScale;
    ctx.stroke();
  }
  ctx.restore();
}

function drawCracks(ctx, points, palette, random, rockScale, erosion, detail, type) {
  if (type === "granite" || detail < 0.35) return;
  const count = type === "eroded" ? 3 + Math.round(erosion / 30) : 2;
  ctx.save();
  polygonPath(ctx, points);
  ctx.clip();
  for (let i = 0; i < count; i += 1) {
    const startX = random.range(-170, 80) * rockScale;
    const startY = random.range(-140, 120) * rockScale;
    ctx.beginPath();
    ctx.moveTo(startX, startY);
    for (let j = 1; j < 5; j += 1) {
      ctx.lineTo(startX + j * random.range(12, 28) * rockScale, startY + j * random.range(-18, 21) * rockScale);
    }
    ctx.strokeStyle = color(palette.dark, 0.28);
    ctx.lineWidth = Math.max(0.6, rockScale * 1.2);
    ctx.stroke();
    ctx.strokeStyle = color(palette.light, 0.1);
    ctx.lineWidth = Math.max(0.4, rockScale * 0.55);
    ctx.translate(1, -1);
    ctx.stroke();
    ctx.translate(-1, 1);
  }
  ctx.restore();
}

function drawRock(canvas, rockType) {
  const ctx = canvas.getContext("2d");
  const width = canvas.width;
  const height = canvas.height;
  const palette = ROCKS[rockType];
  const distance = DISTANCES[state.distance];
  const random = new Random(state.seed + rockType.length * 101 + state.method.length * 37);
  const detail = (state.detail / 100) * distance.detail;
  const rockScale = Math.min(width, height) * 0.42 * distance.scale;
  const centerX = width * 0.5;
  const centerY = height * 0.54;
  const outline = buildOutline(rockType, random, state.roughness, rockScale);
  const points = outline.map((point) => ({ x: centerX + point.x, y: centerY + point.y }));

  drawBackground(ctx, width, height);
  drawShadow(ctx, centerX, centerY, distance.scale);

  ctx.save();
  polygonPath(ctx, points);
  const stoneGradient = ctx.createLinearGradient(centerX - rockScale, centerY - rockScale, centerX + rockScale, centerY + rockScale);
  stoneGradient.addColorStop(0, color(lighten(palette.base, 0.22), 1));
  stoneGradient.addColorStop(0.42, color(palette.base, 1));
  stoneGradient.addColorStop(1, color(darken(palette.base, 0.36), 1));
  ctx.fillStyle = stoneGradient;
  ctx.shadowColor = "rgba(0, 0, 0, .5)";
  ctx.shadowBlur = 23;
  ctx.shadowOffsetY = 15;
  ctx.fill();
  ctx.restore();

  drawPatches(ctx, points, palette, random, distance.scale, detail, rockType, state.seed, state.method);
  drawStrata(ctx, points, palette, random, distance.scale, detail, rockType, state.method);
  drawMinerals(ctx, points, palette, random, distance.scale, detail, state.mineral, rockType, state.method);
  drawCavities(ctx, points, palette, random, distance.scale, state.erosion, detail, rockType);
  drawCracks(ctx, points, palette, random, distance.scale, state.erosion, detail, rockType);

  ctx.save();
  polygonPath(ctx, points);
  ctx.strokeStyle = color(lighten(palette.base, 0.28), 0.37);
  ctx.lineWidth = Math.max(1.5, distance.scale * 2.4);
  ctx.stroke();
  ctx.strokeStyle = color(darken(palette.base, 0.55), 0.36);
  ctx.lineWidth = Math.max(2, distance.scale * 4);
  ctx.translate(2 * distance.scale, 5 * distance.scale);
  ctx.stroke();
  ctx.restore();

  // Um brilho estreito simula a incidência de luz e ajuda a comparar a leitura da forma.
  ctx.save();
  ctx.globalCompositeOperation = "screen";
  const light = ctx.createRadialGradient(centerX - rockScale * 0.42, centerY - rockScale * 0.58, 0, centerX, centerY, rockScale * 1.12);
  light.addColorStop(0, "rgba(255, 240, 211, .16)");
  light.addColorStop(0.48, "rgba(255, 244, 220, .025)");
  light.addColorStop(1, "rgba(255, 244, 220, 0)");
  polygonPath(ctx, points);
  ctx.clip();
  ctx.fillStyle = light;
  ctx.fillRect(0, 0, width, height);
  ctx.restore();
}

function syncControlLabels() {
  $("#seed-value").textContent = state.seed;
  $("#detail-value").textContent = `${state.detail}%`;
  $("#roughness-value").textContent = `${state.roughness}%`;
  $("#erosion-value").textContent = `${state.erosion}%`;
  $("#mineral-value").textContent = `${state.mineral}%`;
  $("#seed-readout").textContent = `seed ${state.seed}`;
  $$(".canvas-distance").forEach((element) => { element.textContent = DISTANCES[state.distance].label; });
}

function updateMetrics() {
  const distanceFactor = DISTANCES[state.distance].detail;
  const methodData = {
    texture: { memory: "2.4 MB", shader: "mínimo", shaderNote: "bitmap + normal map", score: 68, calls: 3 },
    procedural: { memory: "0.0 MB", shader: "alto", shaderNote: "8 camadas de ruído", score: 78, calls: 3 },
    hybrid: { memory: "0.5 MB", shader: "médio", shaderNote: "5 camadas + máscara", score: 86, calls: 3 },
  }[state.method];
  const polygonBase = { natural: 1480, granite: 1320, eroded: 1840 };
  const polygons = Math.round((polygonBase.natural + polygonBase.granite + polygonBase.eroded) / 3 + state.roughness * 5.6);
  const adjustedScore = Math.max(45, Math.min(96, methodData.score + Math.round((state.detail - 58) * 0.08) - (state.erosion > 80 ? 3 : 0) + (state.distance === "mid" ? 2 : state.distance === "far" ? 0 : -1)));
  $("#metric-polygons").textContent = `${(polygons / 1000).toFixed(1)}k`;
  $("#metric-memory").textContent = methodData.memory;
  $("#metric-shader").textContent = methodData.shader;
  $("#metric-shader-note").textContent = methodData.shaderNote;
  $("#metric-calls").textContent = methodData.calls;
  $("#score-value").textContent = adjustedScore;
  $("#decision-bar-fill").style.width = `${adjustedScore}%`;

  const titles = {
    texture: "Textura lê bem, mas denuncia repetição.",
    procedural: "Procedural escala, com custo de shader.",
    hybrid: "Híbrido preserva o detalhe útil.",
  };
  const copies = {
    texture: "O resultado é previsível e barato durante o desenho, mas exige memória por conjunto de mapas e começa a repetir quando a câmera se afasta.",
    procedural: "A variação é ampla e determinística, porém o ruído precisa de disciplina para não virar um padrão matemático caro e visualmente genérico.",
    hybrid: "A geometria sustenta a silhueta no médio alcance; o material procedural acrescenta variação sem carregar um atlas por pedra.",
  };
  $("#decision-title").textContent = titles[state.method];
  $("#decision-copy").textContent = copies[state.method];
  $("#decision-meta").textContent = `consistência ${(adjustedScore / 100).toFixed(2)} · ${state.distance === "far" ? "detalhe reduzido" : "custo controlado"}`;
}

function render() {
  syncControlLabels();
  ["natural", "granite", "eroded"].forEach((type) => drawRock($(`#rock-${type}`), type));
  updateMetrics();
}

function setStateFromInput(id, key) {
  const input = $(`#${id}`);
  input.addEventListener("input", () => {
    state[key] = Number(input.value);
    render();
  });
}

function randomizeParameters() {
  const random = new Random(Date.now() % 100000);
  state.seed = Math.floor(random.range(1, 9999));
  state.detail = Math.floor(random.range(42, 83));
  state.roughness = Math.floor(random.range(46, 88));
  state.erosion = Math.floor(random.range(18, 72));
  state.mineral = Math.floor(random.range(28, 79));
  $("#seed").value = state.seed;
  $("#detail").value = state.detail;
  $("#roughness").value = state.roughness;
  $("#erosion").value = state.erosion;
  $("#mineral").value = state.mineral;
  render();
}

["seed", "detail", "roughness", "erosion", "mineral"].forEach((id) => setStateFromInput(id, id));

$$(".method-tab").forEach((button) => {
  button.addEventListener("click", () => {
    state.method = button.dataset.method;
    $$(".method-tab").forEach((item) => item.classList.toggle("active", item === button));
    render();
  });
});

$$(".distance-tab").forEach((button) => {
  button.addEventListener("click", () => {
    state.distance = button.dataset.distance;
    $$(".distance-tab").forEach((item) => item.classList.toggle("active", item === button));
    render();
  });
});

$("#regenerate").addEventListener("click", () => {
  state.seed = (state.seed * 1664525 + 1013904223) % 10000;
  if (state.seed < 1) state.seed = 1;
  $("#seed").value = state.seed;
  render();
});
$("#randomize").addEventListener("click", randomizeParameters);

render();
