# Prompt de reprise — test sharding « plus gros que la RTX 5080 »

> Colle ce bloc dans une session fraîche (`claude --resume e65948cc-dd02-4a3d-8d2a-3e6217945ac9`
> pour garder le contexte, ou nouvelle session — tout est ci-dessous). Mode ULTRACODE.

## Objectif
Prouver qu'un modèle **trop gros pour les 16 Go de VRAM de la RTX 5080** tourne
**éclaté sur 2 machines** (5080 CUDA + Mac M2 Metal), via le fork du sprint S77.

## Déjà fait (ne pas refaire)
- **Preuve 7B cross-machine RÉUSSIE** : Windows-head (CPU, couches [0,16)) → Mac-tail
  (Metal, [16,32)) == modèle entier, **cosine 0.999727**. Documenté dans
  `.planning/research/cross_machine_sharding_proof_2026-06-23.md`.
- Coordination multi-nœud : `nexus-test-harness` 12/12 live.
- Driver : `crates/nexus-worker-core/examples/shard_node.rs` (modes `whole|head|tail`,
  état-frontière f32 little-endian sur stdin/stdout). **Untracked** — committer si on garde.

## Environnement (vérifié)
- **Windows** (cette machine) : RTX 5080 16 Go VRAM, 31 Go RAM, **nvcc CUDA 12.8 présent**.
  Binaire CPU OK : `target/release/examples/shard_node.exe`.
- **Mac** : alias SSH `mac` (192.168.1.53, user theophilevasseur), M2 arm64 8 Go, Metal.
  PRÊT : `~/cmake-3.30.5-macos-universal/CMake.app/Contents/bin`, shim `~/binshim/ccache`,
  repo `~/nexus` (fork vendoré + `[patch]` synchronisés), binaire Metal
  `~/nexus/target/release/examples/shard_node`, GGUF `~/spike_fork/mistral-7b-q4.gguf`
  + **`~/spike_fork/gemma-26b.gguf`** (le gros, transféré).
- **VPS** : alias `vps` (135.181.42.188), non requis.

## Modèle cible
**`gemma-4-26B-A4B-it Q4` = 16.8 Go** → dépasse les 16 Go VRAM (Ollama ne l'a fait
tourner qu'en offloadant sur CPU). Blob Windows :
`~/.ollama/models/blobs/sha256-b8b76e7206ab7343821f02ed4ad6b4927abe7767a0e7a648354c2e0b98d32b1e`.
Déjà sur le Mac à `~/spike_fork/gemma-26b.gguf`.

## LE BLOQUEUR à régler en premier
Le build CUDA Windows échoue : **`CMake Error: No CUDA toolset found`** — CMake
utilise le générateur `Visual Studio 18 2026` mais l'intégration CUDA→VS de
CUDA 12.8 n'existe pas pour VS 2026 (nvcc est là, l'intégration MSBuild non).

**Fix (par fiabilité) :**
1. **Ninja** (contourne la détection toolset VS, utilise nvcc direct) :
   ```bash
   # ninja si absent : winget install Ninja-build.Ninja  (ou pip install ninja)
   cp target/release/examples/shard_node.exe /tmp/shard_node_cpu.exe   # garder le binaire CPU (référence)
   CMAKE_GENERATOR=Ninja cargo build --release --example shard_node --features llm_llama_cpp_cuda
   ```
2. Ou copier `CUDA/v12.8/extras/visual_studio_integration/MSBuildExtensions/*` dans le
   `BuildCustomizations` de VS 2026.
3. Ou `-G "Visual Studio 17 2022"` si VS 2022 est installé.

## Test une fois CUDA buildé
1. **Prouver que ça dépasse la 5080** :
   ```bash
   B26=~/.ollama/models/blobs/sha256-b8b76e7206ab7343821f02ed4ad6b4927abe7767a0e7a648354c2e0b98d32b1e
   ./target/release/examples/shard_node.exe whole "$B26" "Bonjour"   # CUDA -> doit OOM (16.8>16 Go VRAM)
   nvidia-smi   # confirmer la saturation VRAM
   ```
2. **Split GPU cross-machine** (head CUDA 5080 + tail Metal Mac). Choisir K pour que
   `[0,K)` tienne < 16 Go VRAM (commencer K≈ moitié, ajuster si OOM) ; N = nb couches du 26B :
   ```bash
   ./target/release/examples/shard_node.exe head "$B26" 0 K "The quick brown fox" > /tmp/b26.bin
   ssh mac "~/nexus/target/release/examples/shard_node tail ~/spike_fork/gemma-26b.gguf K N" \
     < /tmp/b26.bin > /tmp/split26.bin
   ```
   (Si la tranche Mac `[K,N)` ne tient pas dans ~5 Go Metal, augmenter K — le Mac garde peu de couches.)
3. **Correction** (référence whole en CPU, tient dans 31 Go RAM) :
   ```bash
   /tmp/shard_node_cpu.exe whole "$B26" "The quick brown fox" > /tmp/full26.bin
   python3 -c "import struct,math;r=lambda p:struct.unpack('<%df'%(len(open(p,'rb').read())//4),open(p,'rb').read());a=r('/tmp/full26.bin');b=r('/tmp/split26.bin');d=sum(x*y for x,y in zip(a,b));print('cosine',d/(math.sqrt(sum(x*x for x in a))*math.sqrt(sum(x*x for x in b))))"
   ```
   Attendu : **cosine > 0.999** = un modèle qui OOM la 5080 tourne correct éclaté 5080+Mac.

## À la fin
Mettre à jour `cross_machine_sharding_proof_2026-06-23.md` avec le résultat 26B.
Décider du sort du driver `examples/shard_node.rs` (committer ou non).

## Contexte projet
S77 DONE (tip `0f597cf`, mémoire à jour). Le sharding live end-to-end (orchestrateur
iroh + boucle autorégressive) reste le carry S78 — ce test prouve la mécanique
capacité/correction, pas le pipeline produit complet.
