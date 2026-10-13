# Scene images, timed frames and history

These contracts describe the bounded shared runtime used for the original Hatsune Miku | GLITCHED HOLOGRAM project (`3420277613`). They do not establish general Wallpaper Engine compatibility or original-engine visual parity. Author assets remain outside Git; preparation follows declarations and resource identities rather than a Workshop-ID branch.

## Custom image properties

An author `scenetexture` declaration with an empty default becomes an image control. Groups remain section metadata and scalar properties retain their existing validation. Nonempty native image defaults and unrelated property types remain gated.

The shell opens its native local-file picker from a background desktop command. Cancellation leaves the draft unchanged. The frontend receives a canonical content token and dimensions, never a path supplied to the renderer. Selecting an image changes the draft; **Save and apply** prepares and accepts the candidate, then atomically saves the assignment and choices. Save failure retains the previous assignment and renderer and leaves the draft editable. **Use author image** omits the saved image choice and restores the authored instance-material fallback, which can differ from the base model texture.

Accepted inputs are static eight-bit PNG and JPEG, at most 32 MiB encoded, with bounded dimensions, decoded extent and metadata. JPEG EXIF orientation is applied. Animated PNG, ICC profiles, explicit non-sRGB EXIF color declarations (including uncalibrated/R03), late or duplicate EXIF, non-sRGB PNG gamma/primaries, HDR metadata, JPEG gain maps, MPO/multiple images, arithmetic and CMYK JPEG are rejected. These conservative input rules do not imply color-managed or HDR rendering.

The selected file is read through one regular-file descriptor with no symlink following. LWE validates and copies its bytes into `$XDG_DATA_HOME/lwe/scene-resources` (by default `~/.local/share/lwe/scene-resources`). Tokens have the form `sha256:<64 lowercase hex digits>`. Restore rereads and hashes the stored bytes; a missing or changed copy reports an error. The original selected file may be removed after successful import. Untrusted IPC cannot request an arbitrary path, URL or encoded image.

The store has a 512 MiB / 256-file limit, private directory/file permissions, a cross-process import lock and immutable atomic creation. Identical imports reuse verified content. Conflicting content is never overwritten. Import does not automatically delete images that may still be referenced by an assignment.

## Frame preparation and presentation

The [timed atlas resource decoder](./scene-timed-atlas.md) reads bounded rectangular BC3 regions directly; it does not decode either whole animation sheet into RGBA. The engine shares immutable encoded/decoded sources and static uploads while retaining private generation state, histories and accepted output caches.

One worker request captures sequence, elapsed time and audio together. Script output, animation regions and text for that request form one packet; stale or partial packets cannot be published. Shader audio uses the same captured spectrum as the VM. The original f32 frame durations are promoted without decimal rounding. Only successful presentation publishes the accepted output.

Paused clocks do not advance script or history. A forced same-size redraw copies the previously accepted output. A resize recomposes at accepted time and inputs, retaining the last history rather than accumulating another sample. Replacement candidates have their own state/history. Freezing the old renderer retains its presented output, moves private history into sealed RGBA CPU snapshots under the same CPU budget, and releases replaceable GPU targets. All readbacks succeed before old GPU histories are released. On restoration, all history uploads complete before the CPU snapshots are released. This enables candidate rejection or atomic-save rollback without pretending to rewind an arbitrary JS heap.

History accumulation, copy and combine are ordered render operations. Temporary targets are reused only after their last reader. If an active draw or swap fails after private state/history has advanced, that generation is quarantined and cannot resume animation or resize. Its accepted output may be redrawn unchanged; engine rejection releases it. Recovery requires a new generation. Candidate failure instead restores the independent frozen previous renderer.

## Limits and host APIs

The shared decoded CPU and logical GPU payload budgets each remain 512 MiB. CPU accounting charges retained allocation capacities and conservative codec/font working reservations. GPU accounting charges unique immutable uploads plus every private output/history/temporary resource, including simultaneous old/candidate ownership. These counters are not total RSS, allocator overhead, driver allocations or measured VRAM. Weak caches do not keep unused assets alive, and budget failures preserve existing owners.

The script extension covers only the required validated forms: immutable `createScriptProperties` controls, frame time, sixteen-band stereo and sixty-four-band average buffers, supported `Float32Array` operations, integer bitwise expressions and string text output. Execution, interruption, memory and worker destruction remain bounded. Fatal worker failure is sticky; there is no fake JS-state rollback. Broader SceneScript lifecycle/editor/storage APIs remain unsupported.

The text path uses bounded font parsing/rasterization, including the original builtin Atami CFF font. Font identity includes source bytes and namespace; immutable font data may be shared across generations. Font parser/rasterizer private heap has a conservative reservation, while source/text output capacities are charged directly. This is not exact third-party allocator instrumentation or Windows text-layout parity.

## Verification boundary

Ordinary tests cover declaration/value separation, safe import and cross-process quota serialization, source identity/lifetime, direct region decoding, packet ordering, history/copy and failure cleanup. Original-package CPU and deterministic offscreen GPU oracles are explicit opt-ins and require private local assets. Alone they cannot certify native desktop presentation, file-picker behavior, persisted restart/restore, captured audio, performance or installed-package support. Separate private-capture/hardware-EGL and native desktop checks retain their own evidence scopes.

The original Miku CPU/GPU, private-capture, nested native UI and finite performance results are recorded separately in the [contributor guide](./guide.md). The full-effect native flow passed, while the 1920×1080 AMD 680M probe reached only 4.53 FPS under a 30 FPS cap. Private acceptance reports retain exact source/binary/input identities. Physical suspend/resume remains explicitly deferred. Nested niri results cannot substitute for physical multi-monitor, installed-package or original-engine parity acceptance.
