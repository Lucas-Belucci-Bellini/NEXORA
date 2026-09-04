// NEXORA Rock Material Lab — shader conceitual, não ligado ao runtime.
// Origem: lógica original do experimento; sem textura ou código externo.
// O protótipo executável usa a mesma ideia em Canvas 2D para permanecer isolado.

struct RockMaterialParams {
    float detailScale;
    float roughness;
    float erosion;
    float mineralVariation;
    float seed;
};

float geologicalNoise(vec3 position, float seed);
float fbm(vec3 position, float seed, int octaves);

vec3 rockAlbedo(vec3 position, RockMaterialParams params, vec3 baseColor) {
    float broad = fbm(position * params.detailScale, params.seed, 3);
    float grain = fbm(position * params.detailScale * 4.0, params.seed + 17.0, 4);
    float mineral = smoothstep(0.67, 0.86, geologicalNoise(position * 7.0, params.seed + 41.0));
    float weathering = smoothstep(0.58, 0.88, fbm(position * 2.5, params.seed + 71.0, 3));

    vec3 tonalRange = mix(baseColor * 0.72, baseColor * 1.22, broad);
    tonalRange = mix(tonalRange, tonalRange * 0.84, weathering * params.erosion);
    tonalRange = mix(tonalRange, vec3(0.71, 0.68, 0.60), mineral * params.mineralVariation * 0.13);
    return mix(tonalRange, tonalRange * (0.91 + grain * 0.16), 0.54);
}

float rockRoughness(vec3 position, RockMaterialParams params) {
    float surface = fbm(position * params.detailScale * 3.0, params.seed + 101.0, 4);
    return clamp(0.54 + params.roughness * 0.32 + surface * 0.12, 0.0, 1.0);
}

// A geometria deve continuar responsável pela silhueta e cavidades maiores.
// O shader fica com variação de albedo/roughness/microdetail, evitando o
// anti-padrão de transferir toda a forma para ruído de alta frequência.
