# NEXORA — PLATFORM ARCHITECTURE

> Documento de arquitetura para o futuro ecossistema **NEXORA GAME + NEXORA PLATFORM**. Este documento define fronteiras e contratos de integração; não implementa a plataforma completa.

**Status:** Proposed  
**Versão do documento:** 0.1  
**Escopo:** arquitetura de integração entre jogo, site oficial e serviços online  
**Decisão central:** o Game Core deve continuar executável offline sempre que uma funcionalidade não exigir autoridade remota.

## 1. Objetivo e princípios

O NEXORA deverá possuir uma plataforma oficial própria que seja mais do que uma página institucional. Ela deverá funcionar como ponto central para distribuição, identidade, conteúdo cosmético, comunidade, documentação, suporte e atualização do jogo. Ao mesmo tempo, a plataforma não deve se tornar uma dependência estrutural que impeça o jogo de iniciar ou operar localmente.

A arquitetura segue os princípios já estabelecidos pelo repositório: cada dado autoritativo possui um owner lógico [1], conteúdos passam por uma pipeline de validação antes de chegar ao runtime [2], assets mantêm registro de procedência e licença [3], e APIs públicas evoluem por versões explícitas [4].

| Princípio | Decisão arquitetural |
| --- | --- |
| **Offline-first para o core** | O jogo inicializa e executa os modos locais sem conta, site ou conexão, exceto funcionalidades que exigem autoridade online. |
| **Serviços opcionais** | Conta, comunidade, sincronização e atualização são capacidades descobertas e ativadas separadamente. |
| **Fonte única de verdade** | O jogo é owner do estado local e da simulação; a plataforma é owner de identidade, publicação, catálogo e distribuição. |
| **Conteúdo não confiável por padrão** | Uploads nunca entram diretamente no jogo; passam por parse, validação, compatibilidade, moderação e publicação. |
| **Contratos versionados** | Game API, Content Schema, Download Manifest e serviços de conta possuem versões compatíveis e migrações explícitas. |
| **Proveniência obrigatória** | Todo asset oficial, comunitário ou de terceiro carrega origem, licença, autor, versão e status de publicação. |

## 2. Visão geral do ecossistema

A plataforma é composta por um site oficial e serviços online. O jogo é um cliente que consome contratos versionados, mas não conhece detalhes internos do site ou dos serviços. O Launcher, quando existir, é um cliente de distribuição separado do processo do jogo; ele não deve ser necessário para executar uma instalação já válida offline.

```text
                         NEXORA PLATFORM
  ┌─────────────────────────────────────────────────────────────┐
  │ Web App   Account   Catalog   Community   Documentation      │
  │                │         │          │                       │
  │        Identity API  Content API  Download API               │
  │                │         │          │                       │
  │        ┌───────┴─────────┴──────────┴───────┐                │
  │        │ Online Services / Server Authority │                │
  │        └────────────────┬───────────────────┘                │
  └─────────────────────────┼───────────────────────────────────┘
                            │ versioned integration contracts
            ┌───────────────┴────────────────┐
            │                                │
     NEXORA GAME CLIENT                 FUTURE LAUNCHER
     ┌────────────────────┐             ┌────────────────────┐
     │ Core offline       │             │ Check / Download   │
     │ Local content      │             │ Verify / Install   │
     │ Save / settings    │             │ Rollback / Repair  │
     └────────────────────┘             └────────────────────┘
```

A indisponibilidade do site deve degradar apenas as capacidades dependentes dele. O cliente deve expor o estado da conexão e preservar dados locais; não deve transformar uma falha de telemetria, catálogo ou comunidade em falha de inicialização do core.

## 3. Game

### 3.1 O que pertence ao jogo

O **Game Core** contém o runtime, a simulação, a apresentação local, o gerenciamento de saves, as configurações, a resolução de conteúdo instalado e as regras necessárias para os modos offline. Ele é responsável por validar o pacote local antes de carregá-lo e por manter a continuidade de uma sessão quando serviços online não estão disponíveis.

O core não deve conhecer banco de dados da plataforma, provedores de identidade, CDN, fila de moderação ou detalhes de implementação do site. A integração ocorre através de adaptadores e contratos públicos, nunca através de imports diretos de serviços web ou de acesso remoto a estado autoritativo.

| Responsabilidade do Game Core | Não pertence ao Game Core |
| --- | --- |
| Executar a simulação local e o gameplay offline. | Criar ou excluir contas da plataforma. |
| Possuir estado de mundo, entidades, inventário e saves locais. | Publicar skins ou aprovar conteúdo comunitário. |
| Resolver manifests e schemas de conteúdo instalados. | Ser a fonte oficial do catálogo online. |
| Renderizar assets autorizados e compatíveis. | Hospedar downloads, CDN ou changelogs. |
| Aplicar permissões locais e capabilities recebidas em contratos. | Decidir moderação, denúncias ou bloqueios globais. |
| Sincronizar apenas quando uma feature explicitamente exigir servidor. | Confiar em qualquer payload remoto sem validação. |

### 3.2 Boundary de integração

A superfície de integração deverá ser pequena, explícita e substituível. Um backend indisponível deve resultar em `Unavailable` ou `Offline`, não em exceção que atravesse o core. Os contratos abaixo são conceituais e servem para orientar a futura API pública.

```ts
interface IPlatformGateway {
  getCapabilities(): Promise<PlatformCapabilities>;
  beginSession(input: SessionStartRequest): Promise<SessionContext | OfflineContext>;
  resolveEntitlements(input: EntitlementQuery): Promise<EntitlementSnapshot | Unavailable>;
  fetchContentManifest(input: ManifestQuery): Promise<ContentManifest | Unavailable>;
  checkUpdates(input: UpdateQuery): Promise<UpdateManifest | Unavailable>;
}

interface IOfflinePolicy {
  canStartLocalSession(): boolean;
  canUseFeature(feature: FeatureID, context: SessionContext): Decision;
  preserveLocalState(): void;
}
```

O gateway não possui estado de gameplay. Ele apenas consulta capacidades e entrega snapshots assinados ou respostas de indisponibilidade. O Game Core continua sendo o owner do estado autoritativo local, em alinhamento com a política de ownership do NEXORA [1].

## 4. Platform

A **NEXORA Platform** é o site e o conjunto de serviços que oferecem identidade, catálogo, distribuição, publicação, descoberta e suporte. Sua função é administrar recursos online e preparar pacotes que o jogo possa consumir com segurança; ela não substitui o runtime nem deve assumir o ownership de saves locais ou da simulação.

| Módulo da plataforma | Função futura | Fonte de verdade |
| --- | --- | --- |
| **Web App** | Navegação pública, conta, biblioteca, comunidade, downloads e documentação. | Interfaces e snapshots dos serviços. |
| **Account Service** | Identidade, perfil, preferências online e vínculos de conta. | Account Service. |
| **Content Service** | Upload, validação, compatibilidade, catálogo e publicação de assets. | Content Service. |
| **Download Service** | Manifests de versão, arquivos, checksums, canais e histórico. | Release/Download Service. |
| **Community Service** | Descoberta, autoria, avaliações, denúncias e estados de moderação. | Community Service. |
| **Documentation/Support** | Manuais, changelogs, requisitos, incidentes e suporte. | CMS ou repositório editorial futuro. |
| **Audit/Moderation** | Trilha de auditoria, revisão, bloqueio e remoção. | Moderation/Audit Service. |

Os módulos podem compartilhar infraestrutura no início, mas suas responsabilidades e contratos devem permanecer separáveis. Uma primeira versão não precisa de microserviços: a separação aqui é de ownership e boundary, não uma exigência de deployment distribuído.

## 5. Account

A **NEXORA Account** representa a identidade online do jogador. Ela deve ser opcional para o gameplay local e necessária apenas para recursos que precisam de sincronização, entitlement, publicação ou participação comunitária. O login não deve sobrescrever silenciosamente saves locais; a associação entre perfil online e dados locais precisa ser uma operação explícita, reversível e auditável.

```text
NEXORA Account
├── Profile
├── Online Preferences
├── Entitlements
├── Cosmetic Library
│   ├── Character Skins
│   ├── Weapon Skins
│   ├── Vehicle Skins
│   └── Equipment Skins
├── Covers / Banners
├── Achievements (online projection)
├── Community Authorship
└── Security / Sessions / Devices
```

### 5.1 Modelo conceitual mínimo

| Entidade | Campos conceituais | Owner | Observação |
| --- | --- | --- | --- |
| `Account` | `account_id`, `created_at`, `status`, `region` | Account Service | Não contém save autoritativo. |
| `Profile` | `account_id`, `display_name`, `avatar_asset_id`, `cover_asset_id` | Account Service | Dados públicos devem ter política de visibilidade. |
| `Entitlement` | `entitlement_id`, `account_id`, `content_id`, `scope`, `status` | Account/Commerce futuro | Não é prova suficiente de que um asset é compatível. |
| `DeviceSession` | `session_id`, `account_id`, `issued_at`, `expires_at`, `revoked_at` | Identity Service | Tokens curtos e revogáveis. |
| `LocalLink` | `installation_id`, `account_id`, `linked_at`, `last_sync` | Cliente + Account Service | Nunca move save sem consentimento explícito. |

A implementação de autenticação, pagamento ou marketplace não faz parte desta etapa. O contrato deve apenas reservar espaço para identidade e entitlements sem obrigar o core a depender deles.

## 6. Content

Todo conteúdo deve ser tratado como um pacote identificável, versionado e validável. Skins, capas, banners, mapas, mods e assets derivados da comunidade compartilham um envelope de metadados, mas cada tipo possui schema, capabilities e limites próprios. O sistema de conteúdo do jogo já prevê que entradas sejam validadas, compiladas, otimizadas, registradas e empacotadas antes do runtime [2].

### 6.1 Asset envelope

```ts
interface ContentAsset {
  asset_id: string;
  author_id: string;
  version: string;
  type: "skin" | "cover" | "banner" | "map" | "mod" | "asset";
  created_at: string;
  updated_at: string;
  engine_api_range: string;
  content_schema_range: string;
  dependencies: string[];
  license: LicenseRef;
  provenance: ProvenanceRef;
  permissions: Capability[];
  moderation_status: "draft" | "review" | "published" | "restricted" | "blocked" | "removed";
  compatibility: CompatibilityReport;
  content_hash: string;
  package_uri?: string;
}
```

`asset_id` identifica a família lógica; `version` identifica uma revisão compatível. O `content_hash` identifica os bytes do pacote publicado. Renomeações e mudanças breaking devem usar aliases ou migrações, nunca substituir silenciosamente um ID, conforme as regras de versionamento do NEXORA [4].

### 6.2 Classes de origem e licença

| Classe | Significado | Requisito |
| --- | --- | --- |
| `NEXORA_OFFICIAL` | Criado ou comissionado pelo projeto. | Registro interno, licença de distribuição e revisão de release. |
| `COMMUNITY_CREATED` | Criado por um usuário ou grupo da comunidade. | Autor identificado, licença declarada, compatibilidade e moderação. |
| `THIRD_PARTY` | Recurso externo licenciado. | Prova de licença, atribuição e aprovação de incorporação. |

O registro de procedência é a fonte de verdade para a origem do asset, enquanto o Git guarda a evolução dos arquivos. Isso preserva a distinção já definida no registro de assets do NEXORA [3]. Nenhum conteúdo copiado de outro jogo deve ser usado para popular a plataforma.

### 6.3 Pipeline de conteúdo comunitário

```text
Creator
  ↓
Upload
  ↓
Parse + Malware/Format Scan
  ↓
Schema Validation
  ↓
Compatibility Check
  ↓
License / Provenance Review
  ↓
Automated Limits
  ↓
Moderation
  ↓
Publish Immutable Package
  ↓
Catalog + Player Discovery
```

O jogo baixa somente pacotes publicados e compatíveis. A validação local continua obrigatória, pois um manifesto remoto pode estar expirado, corrompido ou ser incompatível com a instalação atual. Conteúdo comunitário não deve obter capacidade de executar processos nativos ou acessar dados do usuário sem capability declarada e política de confiança compatível.

## 7. Download

A distribuição oficial deve existir independentemente de uma loja externa. A página de download poderá oferecer builds para Windows, Linux e plataformas futuras, organizadas por canal e arquitetura. O site não deve apresentar um arquivo sem também apresentar sua versão, tamanho, checksum, requisitos e estado de suporte.

```ts
interface DownloadManifest {
  product_id: "nexora-game";
  version: string;
  channel: "stable" | "experimental" | "legacy";
  platform: "windows" | "linux" | string;
  architecture: "x64" | "arm64" | string;
  size_bytes: number;
  sha256: string;
  signed_metadata: string;
  minimum_os: string;
  release_notes_url: string;
  artifacts: DownloadArtifact[];
}
```

O histórico de releases precisa ser imutável depois de publicado, salvo uma entrada de retirada que preserve a evidência. Uma build retirada não deve ser substituída silenciosamente por outro arquivo com a mesma versão e checksum diferente.

| Informação apresentada ao jogador | Motivo |
| --- | --- |
| Versão e canal | Permitir decisão consciente entre estável e experimental. |
| Plataforma, arquitetura e requisitos | Evitar instalação incompatível. |
| Tamanho e espaço necessário | Antecipar custo de download e instalação. |
| SHA-256 e assinatura | Verificar integridade e autenticidade. |
| Changelog | Expor alterações e riscos conhecidos. |
| Histórico | Permitir diagnóstico, rollback e suporte. |

## 8. Updates

O futuro fluxo de atualização poderá ser implementado por um Launcher, pelo instalador ou por um mecanismo de atualização separado. A responsabilidade do launcher é distribuição; a responsabilidade do Game Core é validar que a instalação e os pacotes carregados são compatíveis.

```text
Check Version
  ↓
Resolve Channel + Platform
  ↓
Fetch Signed Manifest
  ↓
Compare Installed Fingerprint
  ↓
Download to Staging
  ↓
Verify Signature + Checksum
  ↓
Install Atomically
  ↓
Run Migration / Compatibility Check
  ↓
Commit or Rollback
```

Atualizações devem ser atômicas e staged. Se a validação, migração ou instalação falhar, a versão anterior deve permanecer inicializável. Migrações de save e content devem preservar a origem e produzir diagnóstico, seguindo o fluxo `Detect → Validate Source → Plan → Dry Run → Transform → Validate Target → Commit` [5].

O NEXORA deverá distinguir pelo menos `stable`, `experimental` e versões legadas mantidas para suporte. O canal escolhido é uma preferência de distribuição, não uma permissão para o servidor alterar estado local sem consentimento.

## 9. Community

O Community Hub futuro será um catálogo moderado de conteúdos permitidos. Ele deve privilegiar descoberta, autoria, compatibilidade e procedência, e não começar como marketplace ou rede social. Avaliações, favoritos e comentários podem ser derivados do serviço, mas não são necessários para o contrato inicial de publicação.

| Área comunitária | Conteúdo inicial possível | Dependências obrigatórias |
| --- | --- | --- |
| Skins | Cosméticos para personagens, armas, veículos e equipamentos. | Schema visual, preview, compatibilidade e licença. |
| Covers | Capas, banners e backgrounds de perfil. | Limites de dimensão, formato e moderação visual. |
| Maps | Mapas ou cenários compartilháveis. | Schema de mundo, dependências e validação de segurança. |
| Mods | Extensões declarativas ou scripts autorizados. | API versionada, capabilities, quotas e sandbox/trust policy. |
| Assets | Recursos reutilizáveis por conteúdos compatíveis. | Proveniência, licença e dependências explícitas. |

Denúncias, revisão, bloqueio, remoção e histórico devem ser modelados desde o começo como estados auditáveis, sem exigir que o usuário conheça a implementação de moderação. Um asset removido não deve desaparecer da trilha de autoria e release; deve manter seu estado `removed` ou `blocked` e o motivo acessível a operadores autorizados.

## 10. Offline

A regra operacional é: **o Game Core pode operar offline; Online Services ampliam a experiência**. Isso significa que o jogo deve abrir com conteúdo local válido, iniciar uma sessão local, carregar saves locais, aplicar configurações e renderizar assets instalados sem consultar a plataforma. O comportamento exato de cada recurso deve ser declarado por capability.

| Recurso | Offline | Online necessário |
| --- | --- | --- |
| Iniciar jogo e sessão local | Sim | Não |
| Carregar save local | Sim | Não, salvo sincronização explícita. |
| Configurações locais | Sim | Não |
| Usar conteúdo já instalado e validado | Sim | Não |
| Criar ou usar perfil online | Não | Sim |
| Sincronizar perfil e entitlements | Não | Sim |
| Navegar no Community Hub | Não | Sim |
| Publicar ou denunciar conteúdo | Não | Sim |
| Verificar atualização | Não | Sim, mas falha deve ser tolerada. |
| Baixar nova versão ou conteúdo | Não | Sim |

O cliente deve indicar se está em `ONLINE`, `OFFLINE`, `DEGRADED` ou `AUTH_REQUIRED`. Em modo degradado, o sistema não deve descartar cache válido nem bloquear recursos que não dependem do servidor. Tokens, snapshots e manifests devem expirar com políticas claras; expiração de sessão online não invalida automaticamente um save local.

## 11. Security

As superfícies online introduzem confiança, privacidade e distribuição de código. O servidor deve validar tudo que for autoridade remota, enquanto o cliente deve validar tudo que for necessário para sua segurança local. A política do NEXORA já exige parse, validação, autorização, rate limit, execução e registro para entradas sensíveis [6].

### 11.1 Trust boundaries

```text
PUBLIC WEB / CREATOR
  → upload boundary
CONTENT VALIDATION SERVICE
  → published immutable package
DOWNLOAD SERVICE
  → signed manifest / artifact
GAME CLIENT
  → local schema, signature, hash, capability validation
GAME CORE
  → local execution within declared limits
```

| Superfície | Validações mínimas |
| --- | --- |
| Conta e sessão | Tokens curtos, revogação, proteção contra replay, escopos mínimos e auditoria. |
| Upload | Tipo, tamanho, decompression ratio, malware scan, schema, dependências e quotas. |
| Assets | Hash, assinatura, procedência, licença, compatibilidade e status de publicação. |
| Mods/scripts | API range, capabilities, sandbox/trust policy, limites de CPU/memória/IO e ausência de acesso arbitrário ao host. |
| Downloads | HTTPS, assinatura de manifest, SHA-256, staging, verificação e rollback. |
| Community | Rate limit, denúncia, revisão, bloqueio, retenção de evidências e logs administrativos. |
| Game commands online | Servidor como autoridade; cliente nunca muta diretamente o estado autoritativo remoto. |

Dados pessoais e dados de gameplay devem possuir owners e retenção diferentes. A plataforma não deve coletar um save completo apenas para oferecer perfil ou cosméticos. Quando uma sincronização for necessária, o contrato deve declarar campos, finalidade, versão, consentimento e comportamento de conflito.

## 12. Contratos de integração

Os contratos abaixo são a fronteira mínima sugerida para a futura implementação. Eles devem ser publicados em schemas versionados e testados com fixtures, sem obrigar o core a depender de um provedor específico.

| Contrato | Produtor | Consumidor | Falha aceitável |
| --- | --- | --- | --- |
| `AccountSession v1` | Identity Service | Game Client / Web App | `AUTH_REQUIRED`, `EXPIRED`, `UNAVAILABLE`. |
| `EntitlementSnapshot v1` | Account/Content Service | Game Client | Usa último snapshot válido ou não aplica recurso online. |
| `ContentManifest v1` | Content Service | Game Client / Launcher | Mantém conteúdo local previamente validado. |
| `DownloadManifest v1` | Download Service | Launcher / Installer | Não altera instalação se assinatura falhar. |
| `CompatibilityReport v1` | Validation Service | Catalog / Game Client | Rejeita pacote incompatível de forma diagnóstica. |
| `ModerationState v1` | Moderation Service | Content / Community | Oculta conteúdo afetado sem apagar evidência. |
| `TelemetryEvent v1` | Game Client | Observability futura | Drop ou buffer local; nunca bloqueia gameplay. |

Todas as respostas remotas precisam transportar `schema_version`, `request_id`, `generated_at`, `expires_at` quando aplicável e um estado explícito de erro. O cliente deve rejeitar campos críticos ausentes e ignorar apenas extensões não críticas declaradas como opcionais.

## 13. Ownership e sincronização

O modelo de dados deve impedir que conta, site e jogo mantenham três verdades concorrentes para o mesmo dado. A plataforma owns identidade, catálogo e publicação; o core owns save, simulação e estado local; o launcher owns a instalação física enquanto executa uma transação de atualização.

| Dado | Owner primário | Outros sistemas podem |
| --- | --- | --- |
| `account_id`, perfil online | Account Service | Consultar snapshots e solicitar comandos autorizados. |
| Catálogo e status de publicação | Content/Community Service | Consultar e indexar como cache derivado. |
| Arquivo publicado e checksum | Download/Release Service | Verificar e armazenar staging local. |
| Instalação local | Launcher/Installer | Game Core consulta fingerprint e solicita atualização. |
| Save e estado de mundo | Game Core / Persistence | Exportar ou sincronizar por contrato explícito. |
| Entitlements | Account Service | Game Client aplica snapshot validado. |
| Estado da simulação remota futura | Servidor de jogo | Cliente envia comandos; nunca escreve autoridade diretamente. |

Conflitos devem ser resolvidos pelo owner, não por uma terceira cópia. Caches online são descartáveis e não devem ser tratados como fonte de verdade, em linha com as regras de ownership existentes [1].

## 14. Roadmap arquitetural, sem implementação imediata

Esta etapa não implementa a plataforma. Ela define a sequência segura para uma futura execução, preservando o core offline e reduzindo dependências prematuras.

| Fase futura | Entregável | Não fazer nesta fase |
| --- | --- | --- |
| 1. Contratos | Schemas de manifest, conteúdo, sessão e compatibilidade com fixtures. | Não criar marketplace nem pagamentos. |
| 2. Distribuição | Página oficial de download, manifests assinados e publicação manual de builds. | Não exigir launcher para iniciar o jogo. |
| 3. Identidade | Account Service mínimo e vínculo opcional da instalação. | Não mover saves sem consentimento. |
| 4. Biblioteca | Catálogo privado de cosméticos e capas próprios. | Não abrir upload público antes da validação. |
| 5. Community Hub | Upload, validação, moderação, catálogo e descoberta. | Não permitir pacote não publicado no runtime. |
| 6. Atualizações | Launcher simples com staging, verify, install e rollback. | Não criar CDN definitiva antes dos requisitos reais. |
| 7. Escala | Observabilidade, quotas, cache, replicação e suporte operacional. | Não transformar uma decisão de deployment em requisito do core. |

## 15. Fora de escopo desta etapa

Não serão criados agora uma loja, sistema de pagamentos, marketplace, rede social, servidor completo, launcher definitivo, CDN definitiva, sistema de contas completo, moderação completa ou integração obrigatória entre o protótipo de pedras e serviços online. O objetivo atual é preservar contratos e fronteiras para que essas capacidades possam existir no futuro sem reescrever o jogo.

Também não é objetivo escolher fornecedor de autenticação, hospedagem, banco de dados, armazenamento de objetos, CDN ou provedor de pagamentos. Essas escolhas devem ser feitas quando houver requisitos operacionais, threat model detalhado, orçamento e volume estimado.

## 16. Critérios de aceitação arquitetural

A arquitetura futura será considerada coerente quando uma instalação local conseguir iniciar e carregar um save válido sem o site, quando conteúdo remoto for aceito apenas por manifest e pacote validados, quando uma versão puder ser verificada por assinatura e checksum, e quando a remoção de um asset comunitário não apagar sua proveniência ou histórico.

| Critério | Evidência esperada |
| --- | --- |
| Core independente | Teste de inicialização com DNS/serviços online indisponíveis. |
| Conteúdo seguro | Fixture de pacote válido, inválido, incompatível e revogado. |
| Distribuição verificável | Manifest assinado, checksum incorreto e rollback testados. |
| Versionamento explícito | Testes de compatibilidade para cada Content/API range suportado. |
| Ownership claro | Matriz de dados sem owners duplicados nem writes diretos fora do owner. |
| Moderação auditável | Histórico de review, publish, restrict, block e remove. |
| Privacidade mínima | Contrato de sincronização que declare finalidade e campos, sem upload implícito de save. |

## Referências

[1]: [NEXORA DATA OWNERSHIP AND SOURCE OF TRUTH.md](NEXORA%20DATA%20OWNERSHIP%20AND%20SOURCE%20OF%20TRUTH.md) — ownership lógico e fonte de verdade.

[2]: [CONTENT PIPELINE.md](CONTENT%20PIPELINE.md) — pipeline de conteúdo e validação antes do runtime.

[3]: [NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md](NEXORA%20ASSET%20PROVENANCE%20AND%20LICENSE%20REGISTRY.md) — procedência, licença e estados de release.

[4]: [NEXORA MOD COMPATIBILITY AND API VERSIONING.md](NEXORA%20MOD%20COMPATIBILITY%20AND%20API%20VERSIONING.md) — versões de APIs, manifests e compatibilidade.

[5]: [VERSIONING AND MIGRATION.md](VERSIONING%20AND%20MIGRATION.md) — migração segura e rollback conceitual.

[6]: [NEXORA SECURITY THREAT MODEL.md](NEXORA%20SECURITY%20THREAT%20MODEL.md) — trust boundaries e validação de entradas.
