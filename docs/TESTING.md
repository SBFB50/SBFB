# NEXUS Test Suite Documentation

## 1. Overview

| Metric | Value |
|---|---|
| **Total test methods** | **416** (including 31 parametrized) |
| **Test files** (in scope) | 11 |
| **Test classes** | 72 |
| **Framework** | pytest + pytest-asyncio |
| **Async tests** | ~110 (all DB CRUD and lifecycle tests) |
| **Parametrized tests** | 31 (GOV worker imports) |

All tests run against **in-memory SQLite** databases -- no external services (Ollama, Neo4j, ChromaDB, Docker) required.

---

## 2. Test File Inventory

| File | Phase | Tests | Covers |
|---|---|---:|---|
| `test_compute.py` | Phase 1 -- Distributed GPU core | 69 | ComputeDatabase CRUD, task lifecycle, auth helpers, dispatcher, events, Pydantic models, config |
| `test_compute_phase2.py` | Phase 2 -- Auto-scaling | 47 | ModelSelector, tier helpers, model tracking DB, transitions, Pydantic models |
| `test_compute_phase4.py` | Phase 4 -- Hybrid mode (exo) | 37 | HybridRouter routing, ExecutionMode, ExoBackend, exo peer, config |
| `test_compute_phase5.py` | Phase 5 -- Dashboard + gamification | 14 | Badge calculation, badge summary, idempotency |
| `test_compute_phase6.py` | Phase 6 -- Security (Proof-of-Computation) | 29 | Ed25519 crypto, digest whitelist, logprob fingerprinting, ResultVerifier |
| `test_compute_phase7.py` | Phase 7 -- Petals swarm | 36 | PetalsBackend, SwarmManager, SwarmHealth, HybridRouter PETALS mode, config |
| `test_compute_phase8.py` | Phase 8 -- Swarm public permanent | 12 | Uptime logging, per-node uptime, network uptime, contributor impact |
| `test_sync.py` | Phase 9 -- Real-time sync (cr-sqlite) | 18 | SyncBroadcaster, SyncReceiver, SYNC_TABLES, config |
| `test_worker.py` | Phase 3 -- Worker client (nexus-worker) | 35 | GPU detection, config persistence, NexusClient, WorkerEngine, dashboard, CLI |
| `test_config.py` | Core -- Configuration | 10 | Settings defaults (FastAPI, models, Neo4j, ChromaDB, search, RAG, monitoring, investigation) |
| `test_gov_workers.py` | GOV module -- Workers + identity | 109 | 31 worker imports, event subscriptions, identity resolution, contradiction helpers, GovDatabase, embed worker |

---

## 3. Per-File Breakdown

### `test_compute.py` -- 69 tests

**TestAuthHelpers** (6 tests)
| Method | Description |
|---|---|
| `test_generate_api_key_length` | API key is at least 32 characters |
| `test_generate_api_key_unique` | 100 generated keys are all distinct |
| `test_hash_api_key_deterministic` | Same key produces same SHA-256 hash |
| `test_hash_api_key_differs` | Different keys produce different hashes |
| `test_hash_ip_deterministic` | Same IP produces same hash |
| `test_hash_ip_privacy` | Hashed IP does not contain original IP string |

**TestComputeNodes** (14 tests)
| Method | Description |
|---|---|
| `test_register_node` | Node registration returns correct fields and stores hashed API key |
| `test_get_node` | Retrieve a node by ID |
| `test_get_node_not_found` | Returns None for nonexistent ID |
| `test_get_node_by_api_key` | Lookup node by raw API key (hashed internally) |
| `test_get_node_by_bad_api_key` | Wrong API key returns None |
| `test_list_nodes` | List all registered nodes |
| `test_list_nodes_filter_status` | Filter nodes by status (idle/offline) |
| `test_get_online_nodes` | Returns idle + busy nodes, excludes offline |
| `test_heartbeat` | Heartbeat updates current_model and returns True |
| `test_heartbeat_nonexistent` | Heartbeat on fake ID returns False |
| `test_update_node_trust` | Trust score delta with clamping [0, 100] |
| `test_ban_node` | Ban sets status=banned and trust=0 |
| `test_increment_node_stats` | Increments completed/errored/avg_tokens fields |
| `test_delete_node` | Soft-deletes a node |

**TestComputeTasks** (10 tests)
| Method | Description |
|---|---|
| `test_create_task` | Creates a pending task with correct fields |
| `test_pull_next_task_priority` | Higher priority (lower number) pulled first |
| `test_pull_task_empty_queue` | Empty queue returns None |
| `test_pull_task_model_affinity` | Model affinity overrides priority ordering |
| `test_complete_task` | Sets status=completed with result and validation |
| `test_fail_task_with_retry` | Failure resets to pending if retries remain |
| `test_fail_task_max_retries` | Failure sets status=failed when retries exhausted |
| `test_list_tasks` | Lists all tasks |
| `test_list_tasks_filter_status` | Filters by pending/assigned |
| `test_count_tasks` | Count total or by status |

**TestComputeResults** (2 tests)
| Method | Description |
|---|---|
| `test_store_result` | Stores result with tokens, time, digest |
| `test_validate_result` | Marks result validated with method name |

**TestComputeStats** (4 tests)
| Method | Description |
|---|---|
| `test_network_stats_empty` | Zero stats on empty network |
| `test_network_stats_with_data` | Aggregates VRAM and pending tasks |
| `test_leaderboard` | Ranked by tasks_completed descending |
| `test_leaderboard_excludes_banned` | Banned nodes excluded from leaderboard |

**TestTaskDispatcher** (5 tests)
| Method | Description |
|---|---|
| `test_model_tiers_ordered` | MODEL_TIERS sorted by min_vram_gb ascending |
| `test_model_tiers_has_zero` | First tier works with 0 VRAM (fallback) |
| `test_spot_check_rate_trusted` | Trusted nodes (>80) get 1% spot-check rate |
| `test_spot_check_rate_standard` | Standard nodes (50-80) get 5% rate |
| `test_spot_check_rate_suspect` | Suspect nodes (<50) get 20% rate |

**TestComputeEventTypes** (6 tests)
| Method | Description |
|---|---|
| `test_node_lifecycle_events` | 4 node lifecycle event types exist |
| `test_task_lifecycle_events` | 5 task lifecycle event types exist |
| `test_validation_events` | 3 validation event types exist |
| `test_model_event` | Model changed event exists |
| `test_tick_events` | Heartbeat and reaper tick events exist |
| `test_total_event_count` | Exactly 16 compute event types total |

**TestPydanticModels** (8 tests)
| Method | Description |
|---|---|
| `test_node_register_valid` | Valid registration request passes |
| `test_node_register_empty_name_rejected` | Empty name raises ValidationError |
| `test_node_register_negative_vram_rejected` | Negative VRAM raises ValidationError |
| `test_task_create_priority_bounds` | Priority 1-10 valid, 0 and 11 rejected |
| `test_task_result_valid` | Valid result request passes |
| `test_task_result_empty_text_rejected` | Empty result text rejected |
| `test_network_stats_response` | NetworkStatsResponse fields |
| `test_leaderboard_response` | LeaderboardResponse with entries |

**TestComputeConfig** (6 tests)
| Method | Description |
|---|---|
| `test_compute_enabled_default` | compute_enabled defaults True |
| `test_compute_heartbeat_timeout` | 90 second default |
| `test_compute_task_timeout` | 300 second default |
| `test_compute_spot_check_rate` | 5% default |
| `test_compute_max_retries` | 3 retries default |
| `test_compute_rate_limit` | 100/minute default |

**TestComputeDatabaseProxy** (2 tests)
| Method | Description |
|---|---|
| `test_proxy_creates_method` | Proxy returns callable for any attribute |
| `test_proxy_different_methods` | Different attributes return different callables |

**TestModuleImports** (6 tests)
| Method | Description |
|---|---|
| `test_import_compute_package` | Top-level package exports main classes |
| `test_import_db` | ComputeDatabase + init_compute_db importable |
| `test_import_dispatcher` | TaskDispatcher + 6 MODEL_TIERS importable |
| `test_import_events` | ComputeEventType importable with >= 10 members |
| `test_import_models` | All Pydantic request/response models importable |
| `test_import_manager` | ComputeManager importable |

---

### `test_compute_phase2.py` -- 47 tests

**TestModelTierHelpers** (11 tests)
| Method | Description |
|---|---|
| `test_zero_vram_returns_basique` | 0 GB maps to Basique tier (gemma-4-12b-q4) |
| `test_16gb_returns_standard` | 16 GB maps to Standard tier |
| `test_14gb_exact_threshold` | 14 GB (exact boundary) maps to Standard |
| `test_13gb_below_standard` | 13 GB falls back to Basique |
| `test_50gb_returns_avance` | 50 GB maps to Avance tier |
| `test_100gb_returns_pro` | 100 GB maps to Pro tier |
| `test_200gb_returns_ultra` | 200 GB maps to Ultra tier |
| `test_500gb_returns_maximum` | 500 GB maps to Maximum tier |
| `test_get_node_model_16gb` | 16384 MB selects 26B model |
| `test_get_node_model_8gb` | 8192 MB selects 12B model |
| `test_get_node_model_24gb` | 24576 MB selects 26B model |

**TestModelSelector** (14 tests)
| Method | Description |
|---|---|
| `test_initial_state` | Selector starts with empty model and STABLE state |
| `test_get_model_for_node_small` | 8 GB node can't run 70B, falls back to individual best |
| `test_get_model_for_node_large` | 48 GB node can run 70B target |
| `test_get_model_for_node_exact_threshold` | 14 GB node runs 26B (exact threshold) |
| `test_get_task_model_stable` | Stable state returns target model |
| `test_get_task_model_transitioning_urgent` | Urgent tasks during transition accept any model |
| `test_get_task_model_transitioning_batch` | Batch tasks during transition wait for target |
| `test_get_min_vram_known_model` | Known model returns correct min VRAM |
| `test_get_min_vram_unknown_model` | Unknown model returns 0 |
| `test_get_status` | Status dict contains target_model, target_tier, state |
| `test_calculate_readiness_no_nodes` | No nodes = STABLE at 100% readiness |
| `test_calculate_readiness_all_ready` | All nodes ready = STABLE at 100% |
| `test_calculate_readiness_some_pulling` | Mixed states = TRANSITIONING at 50% |
| `test_calculate_readiness_incompatible_nodes` | Incompatible nodes = STABLE (nothing to transition) |

**TestModelTracking** (5 tests)
| Method | Description |
|---|---|
| `test_update_node_model_status_ready` | Sets current_model, assigned_model, model_status=ready |
| `test_update_node_model_status_pulling` | Sets assigned_model + pull timestamp, keeps current_model empty |
| `test_set_node_assigned_model` | Updates only assigned_model field |
| `test_get_nodes_by_model` | Filters nodes by current model name |
| `test_get_nodes_needing_pull` | Returns nodes where assigned != current |

**TestModelTransitions** (4 tests)
| Method | Description |
|---|---|
| `test_create_transition` | Creates transition record with old/new model+tier |
| `test_get_active_transition` | Returns active (transitioning) transition |
| `test_complete_transition` | Completing clears active transition |
| `test_list_transitions` | Lists transitions most-recent-first |

**TestPhase2Models** (6 tests)
| Method | Description |
|---|---|
| `test_model_ready_request_valid` | Valid model name accepted |
| `test_model_ready_request_empty_rejected` | Empty model name rejected |
| `test_model_status_response` | Response fields validated |
| `test_node_assignment` | Node assignment fields validated |
| `test_model_transition_entry` | Transition entry fields validated |
| `test_model_ready_response` | Ready response fields validated |

**TestTransitionState** (3 tests)
| Method | Description |
|---|---|
| `test_stable` | TransitionState.STABLE == "stable" |
| `test_transitioning` | TransitionState.TRANSITIONING == "transitioning" |
| `test_degraded` | TransitionState.DEGRADED == "degraded" |

**TestPhase2Imports** (4 tests)
| Method | Description |
|---|---|
| `test_import_model_selector` | ModelSelector + MODEL_TIERS + TransitionState importable |
| `test_import_from_package` | Package-level re-exports work |
| `test_import_phase2_models` | Phase 2 Pydantic models importable |
| `test_dispatcher_uses_model_selector` | TaskDispatcher accepts model_selector param |

---

### `test_compute_phase4.py` -- 37 tests

**TestExecutionMode** (4 tests)
| Method | Description |
|---|---|
| `test_local` | ExecutionMode.LOCAL == "local" |
| `test_distributed` | ExecutionMode.DISTRIBUTED == "distributed" |
| `test_overflow` | ExecutionMode.OVERFLOW == "overflow" |
| `test_count` | 4 execution modes total (LOCAL, DISTRIBUTED, PETALS, OVERFLOW) |

**TestHybridRouter** (14 tests)
| Method | Description |
|---|---|
| `test_exo_disabled_always_local` | All tasks local when exo disabled |
| `test_model_fits_single_node_all_local` | Model fits on one GPU = local |
| `test_model_too_big_all_distributed` | Model exceeds any single node = distributed |
| `test_heavy_task_also_distributed` | Heavy tasks also distributed (no task classification) |
| `test_exo_unavailable_fallback_local` | Cluster down = fallback to local |
| `test_tiny_node_overflow` | Nodes <8 GB serve as overflow |
| `test_normal_node_distributed_not_overflow` | Nodes with decent VRAM participate in distribution |
| `test_needs_distributed_true` | Target > max single node = needs distributed |
| `test_needs_distributed_false` | Target fits single node = no distribution needed |
| `test_update_network_state` | Updates VRAM, max node, target model state |
| `test_get_status` | Status dict includes exo_enabled, exo_url, needs_distributed |
| `test_default_exo_disabled` | Default: exo disabled and unavailable |
| `test_26b_on_16gb_nodes_local` | 26B (14 GB) on 16 GB nodes = local |
| `test_405b_on_16gb_nodes_distributed` | 405B (150 GB) on 16 GB nodes = distributed |

**TestExoBackend** (2 tests)
| Method | Description |
|---|---|
| `test_init` | ExoBackend stores exo_url |
| `test_default_url_from_settings` | Default URL includes port 52415 |

**TestModelSelectorHybrid** (5 tests)
| Method | Description |
|---|---|
| `test_selector_has_hybrid_router` | ModelSelector creates HybridRouter |
| `test_selector_default_execution_mode` | Default = LOCAL |
| `test_get_task_execution_mode_exo_disabled` | Exo disabled = all LOCAL |
| `test_selector_status_includes_hybrid` | Status dict includes execution_mode and hybrid section |
| `test_selector_status_hybrid_has_needs_distributed` | Hybrid section includes needs_distributed + total_vram_gb |

**TestPhase4Models** (2 tests)
| Method | Description |
|---|---|
| `test_hybrid_status_response` | HybridStatusResponse fields validated |
| `test_model_status_has_execution_mode` | ModelStatusResponse includes execution_mode field |

**TestPhase4Config** (3 tests)
| Method | Description |
|---|---|
| `test_exo_disabled_by_default` | exo_enabled defaults False |
| `test_exo_url_default` | exo_url defaults to localhost:52415 |
| `test_exo_health_interval` | 30 second default |

**TestExoPeer** (3 tests)
| Method | Description |
|---|---|
| `test_import` | ExoPeer importable from worker.exo_peer |
| `test_init` | ExoPeer initializes with running=False, healthy=False |
| `test_get_status` | Status includes running and pid fields |

**TestPhase4Imports** (4 tests)
| Method | Description |
|---|---|
| `test_import_hybrid` | HybridRouter, ExoBackend, ExecutionMode importable |
| `test_import_from_package` | Package re-exports work |
| `test_import_exo_peer` | ExoPeer.is_exo_installed is callable |
| `test_import_phase4_models` | HybridStatusResponse importable |

---

### `test_compute_phase5.py` -- 14 tests

**TestBadgeCalculation** (12 tests)
| Method | Description |
|---|---|
| `test_first_task_badge` | 1 completed task awards "first_task" |
| `test_centurion_badge` | 100 tasks awards "centurion" |
| `test_millionnaire_badge` | 1000 tasks awards "millionnaire" |
| `test_pilier_badge` | 10000 tasks awards "pilier" |
| `test_power_node_badge` | >24 GB VRAM awards "power_node" |
| `test_power_node_not_awarded_below_threshold` | 16 GB VRAM does not award "power_node" |
| `test_early_adopter_badge` | First registered node gets "early_adopter" |
| `test_early_adopter_not_after_10` | 11th node does not get "early_adopter" |
| `test_no_badges_for_zero_tasks` | New node with 0 tasks gets no task badges |
| `test_badges_idempotent` | Calling calculate_badges twice produces no duplicates |
| `test_nonexistent_node_returns_empty` | Nonexistent node returns empty badge list |

**TestBadgeSummary** (2 tests)
| Method | Description |
|---|---|
| `test_summary_empty` | Empty DB returns empty summary |
| `test_summary_with_badges` | Aggregates badge counts across nodes |

---

### `test_compute_phase6.py` -- 29 tests

**TestEd25519Crypto** (8 tests)
| Method | Description |
|---|---|
| `test_generate_keypair` | Generates PEM-encoded private + public keys |
| `test_sign_and_verify` | Sign result, verify with matching public key |
| `test_verify_wrong_data_fails` | Verification fails when result_text differs |
| `test_verify_wrong_key_fails` | Verification fails with wrong public key |
| `test_verify_empty_signature_fails` | Empty signature fails verification |
| `test_sign_without_crypto_returns_empty` | Empty private key returns empty signature |
| `test_build_payload_deterministic` | Same inputs produce same payload bytes |
| `test_build_payload_truncates` | Long text truncated in payload |

**TestDigestWhitelist** (5 tests)
| Method | Description |
|---|---|
| `test_no_whitelist_passes` | No registered digests = all pass |
| `test_missing_digest_fails` | Empty digest fails when whitelist exists |
| `test_matching_digest_passes` | Correct digest matches whitelist |
| `test_wrong_digest_fails` | Wrong digest fails with mismatch reason |
| `test_unknown_model_passes` | Model not in whitelist passes |

**TestLogprobFingerprinting** (7 tests)
| Method | Description |
|---|---|
| `test_no_profiles_passes` | No profiles configured = all pass |
| `test_matching_logprobs_passes` | Logprobs within tolerance pass |
| `test_divergent_logprobs_fails` | Logprobs far from expected fail |
| `test_missing_logprobs_fails` | Empty logprobs dict fails |
| `test_calibration_prompts_not_empty` | At least 4 calibration prompts |
| `test_random_calibration_prompt` | Random prompt is from the known set |
| `test_should_calibrate_probabilistic` | ~10% calibration rate (3%-20% tolerance) |

**TestResultVerifier** (6 tests)
| Method | Description |
|---|---|
| `test_all_pass_no_checks` | No checks configured = +1 trust, pass |
| `test_valid_signature_passes` | Valid Ed25519 signature passes |
| `test_invalid_signature_bans` | Invalid signature = -50 trust, ban |
| `test_digest_mismatch_bans` | Wrong model digest = -50 trust, ban |
| `test_logprob_divergence_flags_but_no_ban` | Logprob divergence = -5 trust, no ban |
| `test_spot_check_needed_trusted` | Trusted nodes rarely spot-checked (<5%) |
| `test_spot_check_needed_suspect` | Suspect nodes frequently spot-checked (>10%) |

**TestPhase6Imports** (3 tests)
| Method | Description |
|---|---|
| `test_import_crypto` | generate_keypair, sign_result, verify_signature importable |
| `test_import_verification` | ResultVerifier, verify_digest, verify_logprobs importable |
| `test_import_calibration_prompts` | CALIBRATION_PROMPTS has >= 4 entries |

---

### `test_compute_phase7.py` -- 36 tests

**TestPetalsBackend** (5 tests)
| Method | Description |
|---|---|
| `test_init_default` | Default model is Llama 3.1 405B, not loaded |
| `test_init_custom_model` | Custom model name stored |
| `test_available_reflects_import` | available matches HAS_PETALS flag |
| `test_get_status_not_loaded` | Status shows loaded=False |
| `test_unload` | Unload sets loaded=False |

**TestSwarmManager** (7 tests)
| Method | Description |
|---|---|
| `test_initial_state` | Starts UNKNOWN, 0 nodes, 0 blocks, not ready |
| `test_coverage_pct_zero` | 0 total blocks = 0% coverage |
| `test_coverage_pct_full` | 80/80 blocks = 100% |
| `test_coverage_pct_partial` | 40/80 blocks = 50% |
| `test_is_ready_when_healthy` | HEALTHY = ready |
| `test_is_ready_when_degraded` | DEGRADED = not ready |
| `test_get_status` | Status dict has all expected fields |

**TestSwarmHealth** (5 tests)
| Method | Description |
|---|---|
| `test_healthy` | SwarmHealth.HEALTHY == "healthy" |
| `test_degraded` | SwarmHealth.DEGRADED == "degraded" |
| `test_offline` | SwarmHealth.OFFLINE == "offline" |
| `test_unknown` | SwarmHealth.UNKNOWN == "unknown" |
| `test_count` | Exactly 4 health states |

**TestModelBlocks** (3 tests)
| Method | Description |
|---|---|
| `test_405b_blocks` | 405B has 126 blocks |
| `test_70b_blocks` | 70B has 80 blocks |
| `test_8b_blocks` | 8B has 32 blocks |

**TestHybridRouterPetals** (6 tests)
| Method | Description |
|---|---|
| `test_petals_mode_when_ready` | Ready swarm with enough VRAM routes to PETALS |
| `test_petals_not_ready_falls_to_exo` | Unready swarm falls back to DISTRIBUTED |
| `test_petals_disabled_uses_exo` | Petals disabled = use exo DISTRIBUTED |
| `test_petals_ready_but_low_vram_uses_exo` | Ready but low VRAM = use exo |
| `test_execution_mode_enum_has_petals` | PETALS in ExecutionMode, total 4 modes |
| `test_get_status_includes_petals` | Status includes petals_enabled + petals_ready |

**TestModelSelectorPetals** (2 tests)
| Method | Description |
|---|---|
| `test_default_no_swarm` | Selector starts with no swarm manager |
| `test_status_includes_swarm_when_available` | Status includes swarm section when manager attached |

**TestPhase7Config** (5 tests)
| Method | Description |
|---|---|
| `test_petals_disabled_by_default` | petals_enabled defaults False |
| `test_petals_model_default` | Default model contains "405B" |
| `test_petals_initial_peers_empty` | No initial peers by default |
| `test_petals_health_interval` | 60 second default |
| `test_petals_min_vram` | 150 GB minimum VRAM for Petals |

**TestPhase7Imports** (3 tests)
| Method | Description |
|---|---|
| `test_import_petals_backend` | PetalsBackend importable |
| `test_import_swarm` | SwarmManager, SwarmHealth, MODEL_BLOCKS importable |
| `test_import_hybrid_petals_mode` | ExecutionMode.PETALS exists |

---

### `test_compute_phase8.py` -- 12 tests

**TestUptimeLogging** (3 tests)
| Method | Description |
|---|---|
| `test_log_connect` | Log connect returns valid log ID |
| `test_log_disconnect` | Disconnect sets duration_seconds >= 0 |
| `test_multiple_sessions` | Two connect/disconnect cycles = 2 sessions |

**TestNodeUptime** (3 tests)
| Method | Description |
|---|---|
| `test_no_sessions` | Node with no sessions returns 0 total seconds |
| `test_open_session` | Open session shows current_session_seconds >= 0 |
| `test_closed_session` | Closed session shows current_session_seconds == 0 |

**TestNetworkUptime** (2 tests)
| Method | Description |
|---|---|
| `test_empty_network` | Empty network returns all zeros |
| `test_with_nodes` | Two idle nodes = 100% uptime |

**TestNodeImpact** (4 tests)
| Method | Description |
|---|---|
| `test_nonexistent_node` | Nonexistent node returns empty dict |
| `test_new_node_impact` | New node has 0 tasks, 0 tokens, 0 percentile |
| `test_node_with_tasks` | Node with 100 tasks shows correct percentile |
| `test_impact_includes_uptime` | Impact includes uptime section with sessions |

---

### `test_sync.py` -- 18 tests

**TestSyncBroadcaster** (4 tests)
| Method | Description |
|---|---|
| `test_init_default` | Starts with 0 clients, version 0, 0 changes sent |
| `test_crsqlite_initially_false` | cr-sqlite not available by default |
| `test_get_status` | Status has running, crsqlite_available, db_version, etc. |
| `test_stop_without_start` | Stopping without starting does not raise |

**TestSyncReceiver** (5 tests)
| Method | Description |
|---|---|
| `test_init` | Starts disconnected, version 0, 0 changes applied |
| `test_get_status` | Status has running, connected, local_version, etc. |
| `test_stop_without_start` | Stopping without starting does not raise |
| `test_default_local_path` | Local DB path includes "nexus_local.db" |
| `test_snapshot_url_derived` | Derives HTTPS snapshot URL from WSS server URL |
| `test_snapshot_url_ws_to_http` | Derives HTTP snapshot URL from WS server URL |

**TestSyncTables** (3 tests)
| Method | Description |
|---|---|
| `test_tables_not_empty` | At least 15 sync tables configured |
| `test_core_tables_present` | politicians, positions, contradictions, laws, press present |
| `test_all_tables_are_strings` | All table names are strings starting with "gov_" |

**TestPhase9Config** (2 tests)
| Method | Description |
|---|---|
| `test_sync_disabled_by_default` | sync_enabled defaults False |
| `test_sync_poll_interval` | 0.1 second default poll interval |

**TestPhase9Imports** (4 tests)
| Method | Description |
|---|---|
| `test_import_broadcaster` | SyncBroadcaster + SYNC_TABLES importable |
| `test_import_receiver` | SyncReceiver importable |
| `test_import_api` | sync API router importable |
| `test_import_package` | Package re-exports work |

---

### `test_worker.py` -- 35 tests

**TestGPUDetect** (7 tests)
| Method | Description |
|---|---|
| `test_format_vram_gb` | 16384 MB formats as "16 GB" |
| `test_format_vram_mb` | 512 MB formats as "512 MB" |
| `test_format_vram_zero` | 0 formats as "0 MB" |
| `test_detect_gpu_returns_dict` | Returns dict with gpu_model, vram_mb, platform |
| `test_detect_gpu_platform_set` | Platform is windows, linux, or darwin |
| `test_detect_gpu_no_gpu` | No GPU detected = "Unknown GPU", 0 VRAM |
| `test_detect_gpu_pynvml_preferred` | pynvml detection preferred over nvidia-smi |

**TestConfig** (4 tests)
| Method | Description |
|---|---|
| `test_defaults_have_required_keys` | Required keys present in _DEFAULTS |
| `test_load_config_defaults` | Missing config file returns defaults |
| `test_save_and_load_config` | Save then load preserves values + merges defaults |
| `test_is_registered_false` | No config file = not registered |
| `test_is_registered_true` | Config with server_url + api_key = registered |

**TestNexusClient** (5 tests)
| Method | Description |
|---|---|
| `test_client_init` | Stores server_url |
| `test_client_strips_trailing_slash` | Trailing slash removed from URL |
| `test_auth_headers` | API key added as Bearer token |
| `test_auth_headers_empty` | No key = no Authorization header |
| `test_close_without_open` | Closing unused client does not error |

**TestWorkerEngine** (4 tests)
| Method | Description |
|---|---|
| `test_initial_state` | Starts IDLE, empty model, no task, 0 stats |
| `test_pause_resume` | Pause sets PAUSED, resume sets IDLE |
| `test_uptime_zero_before_start` | Uptime is 0 before starting |
| `test_state_callback` | State change callback receives state transitions |

**TestWorkerState** (2 tests)
| Method | Description |
|---|---|
| `test_all_states_exist` | All 6 states: idle, pulling_model, processing, paused, error, stopped |
| `test_state_count` | Exactly 6 states |

**TestDashboard** (3 tests)
| Method | Description |
|---|---|
| `test_build_dashboard_returns_panel` | Returns Rich Panel instance |
| `test_build_dashboard_shows_name` | Dashboard renders node name |
| `test_build_dashboard_shows_gpu` | Dashboard renders GPU model + VRAM |

**TestCLI** (3 tests)
| Method | Description |
|---|---|
| `test_main_import` | cli.main is callable |
| `test_register_parser` | Module loads without error |
| `test_version` | Package version is "0.1.0" |

**TestWorkerImports** (7 tests)
| Method | Description |
|---|---|
| `test_import_config` | load_config, save_config, is_registered importable |
| `test_import_gpu_detect` | detect_gpu, format_vram importable |
| `test_import_client` | NexusClient importable |
| `test_import_engine` | WorkerEngine + 6 WorkerStates importable |
| `test_import_dashboard` | build_dashboard, run_dashboard importable |
| `test_import_cli` | cli.main importable |
| `test_import_package` | worker package has __version__ |

---

### `test_config.py` -- 10 tests

**TestSettingsDefaults** (10 tests)
| Method | Description |
|---|---|
| `test_fastapi_defaults` | Host 0.0.0.0, port 8000, debug True |
| `test_model_defaults` | Fast/reasoning/deep all use gemma-4-26B, embedding uses nomic |
| `test_neo4j_defaults` | bolt://localhost:7687, user neo4j |
| `test_chromadb_defaults` | localhost:8100 |
| `test_search_defaults` | SearXNG localhost:8888, Robin localhost:9090 |
| `test_ollama_default` | localhost:11434 |
| `test_rag_defaults` | chunk_size 512, overlap 128, top_k 20 |
| `test_monitoring_intervals` | Clearweb 6h, darkweb 24h |
| `test_investigation_loop_defaults` | 30min cycle, 70% relevance threshold, etc. |
| `test_auto_module_flags` | All 7 auto-module flags default True |
| `test_storage_paths` | data_dir, upload_dir, sqlite_path end correctly |

---

### `test_gov_workers.py` -- 109 tests

**TestGovEventTypes** (5 tests)
| Method | Description |
|---|---|
| `test_data_ingestion_events` | 6 ingestion event types exist |
| `test_media_processing_events` | 3 media processing event types exist |
| `test_analysis_events` | 3 analysis event types exist |
| `test_tick_events` | 4 tick event types (hourly, daily, weekly, monthly) exist |
| `test_all_required_event_names_exist` | All 19 required event type names present |
| `test_event_count_at_least_19` | At least 19 GovEventType members |

**TestGovWorkerImports** (32 tests)
| Method | Description |
|---|---|
| `test_worker_import[*]` | 31 parametrized tests: each worker module imports, exposes its class, has subscriptions and name |
| `test_total_worker_count` | All 31 workers importable |

Workers verified: GovVoteSyncWorker, GovDeputeSyncWorker, GovSenatSyncWorker, GovLawSyncWorker, GovHATVPSyncWorker, GovFabriqueSyncWorker, GovWikidataSyncWorker, GovAffairsSyncWorker, GovPressSyncWorker, GovFactcheckSyncWorker, GovEUParliamentSyncWorker, GovEURlexSyncWorker, GovTwitterSyncWorker, GovFacebookSyncWorker, GovInstagramSyncWorker, GovYouTubeSyncWorker, GovTikTokSyncWorker, GovTranscriptionWorker, GovVisionWorker, GovContradictionAnalyzer, GovVotingPatternAnalyzer, GovNeo4jSyncWorker, GovSentimentAnalyzer, GovAlertWorker, GovEmbedWorker, GovBiographyWorker, GovWeeklyRecapWorker, GovVoteImpactWorker, GovPressAffairDetector, GovNewsletterWorker, GovSocialPublishWorker.

**TestGovWorkerSubscriptions** (31 tests)
| Method | Description |
|---|---|
| `test_contradiction_analyzer_subscriptions` | Subscribes to position+social+transcription+press (4 events) |
| `test_voting_pattern_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_sentiment_subscriptions` | Subscribes to GOV_PRESS_ADDED |
| `test_transcription_subscriptions` | Subscribes to GOV_VIDEO_DOWNLOADED |
| `test_vision_subscriptions` | Subscribes to GOV_VIDEO_DOWNLOADED |
| `test_alert_subscriptions` | Subscribes to contradiction+affair+pattern |
| `test_embedding_subscriptions` | Subscribes to position+social+transcription+press |
| `test_neo4j_sync_subscriptions` | Subscribes to position+affair+press+contradiction+politician+declaration |
| `test_biography_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_weekly_recap_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_vote_impact_subscriptions` | Subscribes to TICK_DAILY |
| `test_press_affair_detector_subscriptions` | Subscribes to GOV_PRESS_ADDED |
| `test_newsletter_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_social_publish_subscriptions` | Subscribes to GOV_CONTRADICTION_FOUND |
| `test_vote_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_depute_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_senat_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_hatvp_sync_subscriptions` | Subscribes to TICK_MONTHLY |
| `test_law_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_fabrique_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_wikidata_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_affairs_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_press_sync_subscriptions` | Subscribes to TICK_HOURLY |
| `test_factcheck_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_eu_parliament_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_eurlex_sync_subscriptions` | Subscribes to TICK_WEEKLY |
| `test_twitter_sync_subscriptions` | Subscribes to TICK_HOURLY |
| `test_facebook_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_instagram_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_youtube_sync_subscriptions` | Subscribes to TICK_DAILY |
| `test_tiktok_sync_subscriptions` | Subscribes to TICK_DAILY |

**TestGovIdentityNormalizeName** (17 tests)
| Method | Description |
|---|---|
| `test_basic_lowercase` | "Emmanuel Macron" -> "emmanuel macron" |
| `test_strip_monsieur` | Removes "M." prefix and particles |
| `test_strip_madame` | Removes "Mme" prefix and particles |
| `test_strip_madame_dot` | Removes "Mme." prefix |
| `test_strip_accents` | Strips accented characters |
| `test_strip_accents_cedilla` | Handles c-cedilla |
| `test_remove_d_apostrophe` | "D'Aubert" -> "aubert" |
| `test_remove_l_apostrophe` | "L'Huillier" -> "huillier" |
| `test_remove_particle_de` | "de Villepin" -> "villepin" |
| `test_remove_particle_du` | "du Pont" -> "pont" |
| `test_remove_hyphens` | "Jean-Luc" -> "jean luc" |
| `test_empty_string` | Empty input returns empty |
| `test_none_input` | None returns empty |
| `test_non_string_input` | Integer input returns empty |
| `test_whitespace_only` | Whitespace-only returns empty |
| `test_complex_compound_name` | Complex name with all transformations applied |
| `test_title_dr` | "Dr." prefix stripped |

**TestGovIdentityComputeSimilarity** (7 tests)
| Method | Description |
|---|---|
| `test_identical_names_score_one` | Identical names score > 0.99 |
| `test_identical_after_normalization` | Same person with different titles score > 0.99 |
| `test_different_people_low_score` | Unrelated names score < 0.6 |
| `test_similar_names_moderate_score` | Partially matching names score 0.3-0.95 |
| `test_empty_name_returns_zero` | Empty string returns 0.0 |
| `test_accented_vs_plain` | Accented vs plain names match after normalization |
| `test_symmetry` | similarity(A,B) == similarity(B,A) |

**TestContradictionAnalyzerHelpers** (7 tests)
| Method | Description |
|---|---|
| `test_subject_keywords_basic` | Extracts keywords from subject string |
| `test_subject_keywords_filters_short` | Filters stopwords and short particles |
| `test_subject_keywords_empty` | Empty/None returns empty set |
| `test_subjects_overlap_true` | Overlapping keywords detected |
| `test_subjects_overlap_apostrophe_no_match` | Apostrophe tokens don't cross-match |
| `test_subjects_overlap_false` | Non-overlapping subjects return False |
| `test_subjects_overlap_empty` | Empty input returns False |

**TestGovDatabase** (2 tests)
| Method | Description |
|---|---|
| `test_instantiation_with_memory_db` | Creates tables, verifies gov_politicians/positions/contradictions/alerts exist |
| `test_create_and_get_politician` | CRUD: create Marine Le Pen, retrieve by ID |

**TestGovManagerWorkerSpecs** (1 test)
| Method | Description |
|---|---|
| `test_worker_specs_list_has_31_entries` | GovManager.start() source contains 31 worker specs |

**TestGovEmbedWorker** (5 tests)
| Method | Description |
|---|---|
| `test_make_embed_id_deterministic` | Same inputs produce same embed ID |
| `test_make_embed_id_differs_by_chunk` | Different chunk index = different ID |
| `test_make_embed_id_differs_by_source_type` | Different source type = different ID |
| `test_chunk_text_short` | Short text returns 1 chunk |
| `test_chunk_text_long` | Long text split into multiple non-empty chunks |

**TestGovDatabaseProxy** (2 tests)
| Method | Description |
|---|---|
| `test_getattr_returns_callable` | Proxy returns callable for any attribute |
| `test_different_methods_return_different_callables` | Different attributes return different callables |

---

## 4. How to Run

### Full suite
```bash
python -m pytest tests/ -v
```

### Single phase
```bash
# Phase 1: Distributed GPU core
python -m pytest tests/test_compute.py -v

# Phase 2: Auto-scaling
python -m pytest tests/test_compute_phase2.py -v

# Phase 4: Hybrid mode
python -m pytest tests/test_compute_phase4.py -v

# Phase 5: Dashboard + badges
python -m pytest tests/test_compute_phase5.py -v

# Phase 6: Security / Proof-of-Computation
python -m pytest tests/test_compute_phase6.py -v

# Phase 7: Petals swarm
python -m pytest tests/test_compute_phase7.py -v

# Phase 8: Swarm public permanent
python -m pytest tests/test_compute_phase8.py -v

# Phase 9: Real-time sync
python -m pytest tests/test_sync.py -v

# Phase 3: Worker client
python -m pytest tests/test_worker.py -v

# Config
python -m pytest tests/test_config.py -v

# GOV module
python -m pytest tests/test_gov_workers.py -v
```

### Single test class
```bash
python -m pytest tests/test_compute.py::TestComputeNodes -v
```

### Single test method
```bash
python -m pytest tests/test_compute.py::TestComputeNodes::test_register_node -v
```

### By keyword
```bash
python -m pytest tests/ -k "badge" -v
python -m pytest tests/ -k "petals" -v
```

### Quiet summary (CI mode)
```bash
python -m pytest tests/ -q
```

---

## 5. Test Patterns Used

### In-memory SQLite fixtures

Every DB test uses `aiosqlite.connect(":memory:")` with the real DDL schema applied. Fixture defined per-file or in `conftest.py`:

```python
@pytest_asyncio.fixture
async def db():
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.executescript(_COMPUTE_CREATE_TABLES)
    await conn.executescript(_COMPUTE_CREATE_INDEXES)
    yield ComputeDatabase(conn)
    await conn.close()
```

Shared fixtures in `tests/conftest.py`:
- `memory_conn` -- raw aiosqlite connection with core schema
- `db` -- `Database` instance wrapping memory_conn
- `bus` -- `EventBus` backed by a temp file (uses `tmp_path`)

### Mocking

Used sparingly, primarily for GPU detection and config file paths:
- `unittest.mock.patch` for GPU backend isolation (pynvml, nvidia-smi, apple silicon)
- `unittest.mock.patch` for config file paths (redirecting to `/nonexistent/` or tempdir)
- `unittest.mock.AsyncMock` available but rarely needed (tests target pure logic)

### Async tests

All database CRUD tests use `@pytest.mark.asyncio` with pytest-asyncio's auto mode. The `conftest.py` sets `event_loop_policy` at session scope.

### Parametrized tests

GOV worker imports use `@pytest.mark.parametrize` over 31 `(module_path, class_name)` tuples to verify every worker is importable and has `subscriptions` + `name`.

### Setup/teardown

Phase 6 tests use `setup_method` to clear module-level state:
```python
def setup_method(self):
    _DIGEST_WHITELIST.clear()
    _LOGPROB_PROFILES.clear()
```

### Conditional skips

Ed25519 crypto tests use `@pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")` to skip when the optional `cryptography` package is absent.

---

## 6. Coverage by Component

### Well-tested (dedicated test files)

| Component | Tests | Notes |
|---|---:|---|
| `nexus.compute.db` (ComputeDatabase) | ~60 | Full CRUD: nodes, tasks, results, stats, badges, uptime, impact, transitions |
| `nexus.compute.model_selector` (ModelSelector) | ~25 | Tier selection, per-node assignment, transitions, readiness |
| `nexus.compute.hybrid` (HybridRouter) | ~20 | All 4 execution modes, routing logic, network state |
| `nexus.compute.verification` (ResultVerifier) | ~18 | 3-layer verification: crypto + digest + logprobs |
| `nexus.compute.crypto` | 8 | Ed25519 keygen, signing, verification |
| `nexus.compute.models` (Pydantic) | ~16 | Validation for all request/response types |
| `nexus.config.Settings` | ~25 | All config defaults across all phases |
| `nexus.gov.events` (GovEventType) | 6 | All 19+ event types verified |
| `nexus.gov.workers.*` (31 workers) | ~63 | Import + subscription verification for all 31 workers |
| `nexus.gov.identity` | 24 | normalize_name edge cases + compute_similarity |
| `nexus.gov.workers.contradiction_analyzer` | 7 | Pure helper functions |
| `nexus.gov.workers.embedding` | 5 | Embed ID generation + chunking |
| `nexus.gov.db` (GovernmentDatabase) | 2 | Schema creation + basic CRUD |
| `nexus.sync.*` | 12 | Broadcaster + Receiver init/status |
| `worker.*` (client package) | 35 | GPU detect, config, client, engine, dashboard, CLI |

### Gaps (no dedicated tests in these 11 files)

These modules have test files in the `tests/` directory that were **not in scope** for this document:

| File | Tests | Module covered |
|---|---|---|
| `test_db.py` | - | `nexus.db.sqlite_db` core database |
| `test_api.py` | - | FastAPI endpoint integration |
| `test_event_bus.py` | - | `nexus.events.bus` EventBus |
| `test_vram_scheduler.py` | - | `nexus.events.vram_scheduler` |
| `test_reactive_worker.py` | - | `nexus.events.worker` base class |
| `test_monitoring_loop.py` | - | `nexus.events.monitoring_loop` |
| `test_workers.py` | - | Core investigation workers |
| `test_retriever.py` | - | RAG hybrid retriever |
| `test_hypothesis_engine.py` | - | Hypothesis generation + ACH |
| `test_contradiction_detector.py` | - | Core contradiction detection |
| `test_suspect_scorer.py` | - | 5-factor suspect scoring |
| `test_ingest.py` | - | Evidence ingestion pipeline |
| `test_forensics.py` | - | BPA, acoustics, traces |
| `test_parsers.py` | - | Document parsers |
| `test_chunker.py` | - | Text chunking |
| `test_geo.py` | - | Geocoding |
| `test_sse_bridge.py` | - | SSE event bridge |
| `test_e2e_gov.py` | - | GOV end-to-end integration |

### Modules with no known test coverage

| Module | Risk |
|---|---|
| `nexus.compute.manager` (ComputeManager) | Import-tested only, no behavioral tests |
| `nexus.compute.dispatcher` (full TaskDispatcher) | Only spot-check rates tested, not full dispatch logic |
| `nexus.llm.*` | Router, Ollama client untested (requires live Ollama) |
| `nexus.monitoring.*` | SearXNG, Robin, Wayback monitors untested |
| `nexus.vision.*` | DINOv2, CLIP untested |
| `nexus.recon.*` | OSINT tools untested |
| `nexus.api.*` routers | Only via test_api.py (out of scope) |

---

## 7. CI Integration

GitHub Actions CI is defined in `.github/workflows/ci.yml` with **3 jobs**:

### `backend-tests`
- **Runs on**: `ubuntu-latest`
- **Python**: 3.13 with pip cache
- **Install**: `requirements.txt` + `pytest pytest-asyncio aiosqlite`
- **Command**: `python -m pytest tests/ -q --ignore=tests/test_vram_scheduler.py`
- **Environment**: `NEO4J_URI=""` and `CHROMA_HOST=""` (disables external services)
- **Note**: `test_vram_scheduler.py` is explicitly ignored (requires GPU)

### `frontend-typecheck`
- **Runs on**: `ubuntu-latest`
- **Node**: 22 with npm cache
- **Steps**: `npm ci` -> `tsc --noEmit` -> `vite build`

### `lint`
- **Runs on**: `ubuntu-latest`
- **Steps**: `py_compile nexus/main.py` + `py_compile nexus/gov/events.py`

### Triggers
- Push to `main` or `master`
- Pull requests targeting `main` or `master`
