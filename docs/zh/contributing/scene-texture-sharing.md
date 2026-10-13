# Scene 纹理所有权与预算

Scene 仍为实验性功能，按有界功能子集逐项评估。本次只在同一个存活的 EGL
上下文中共享只读上传，不新增 Scene 格式、特效、动画类型或创意工坊条目兼容性。

## 身份与采样

准备阶段在资源 worker 上计算 SHA-256 身份，输入包括实际解码的同一个有界
descriptor 的字节、解码 RGBA、完整纹理元数据与动画浮点位模式、资源来源、
规范资源根目录、归一化路径、原始绑定、atlas 模式和解码契约版本。
同路径、同大小的文件替换不会复用旧像素。生产纹理的资源 I/O、解码与指纹计算
均不在 engine 线程执行。

每个 engine 持有一个 `Rc<TexturePool>`；renderer 持有 `Rc` lease，pool
只用 `Weak` 索引。键包含固定采样策略：linear/repeat 与 linear/clamp 使用
不同 GL 纹理对象。绘制不再修改共享上传的 sampler 状态，维持现有 OpenGL 3.0
要求，无需 OpenGL 3.3 sampler 对象。

engine 将共享域绑定到实际 EGL 上下文及其生命周期 token。当前上下文不同、
上下文 owner 已销毁、共享域关闭或退役结果不明确时，后续纹理操作会被拒绝。
新 engine/上下文使用新共享域；驱动复用 native handle 不能使旧 owner 重新有效。
独立调用 `PreparedScene::initialize` 延续原有契约：调用方必须保持上下文 current，
直到 renderer 完成清理与销毁。

## 分配与事务

GPU 逻辑预算仍为 512 MiB。在 candidate 上传前，engine 对所有保留的 current、
candidate、rollback 及未初始化输出预留计算只读键并集。每种不可变采样策略只计
一次，各代私有资源分别计费，退役上传在 GL 删除成功前仍计费。算术溢出及同键
分配尺寸冲突均拒绝继续；被拒绝的计划不会分配 candidate 或驱逐 current。

scratch、命名 target、完整帧 target、bloom、几何、script、audio 及已接受状态
保持私有。resize 保留旧私有布局，并在新布局分配成功前计入完整替换布局。
共享不改变 first-frame ACK、commit、原子 assignment 保存、finalize、取消与
rollback 的顺序。

最后一个 lease 释放时只把 GL 对象加入退役队列，由所属 current 上下文在
renderer cleanup 时处理。删除成功后才取消计费；不明确的 GL 失败将隔离共享域，
避免重试旧数字对象 ID。存在 live lease 时关闭会报错，分配账目保留至上下文销毁。

CPU 解码存储另有保守的 512 MiB 保留分配上限。按真实 RGBA `Vec` capacity
求和，只按 `PreparedScene` Arc 身份在 session 和保留的 request/rollback source
之间去重。内容相同但独立准备的 vector 分别计费，因此 GPU 共享不会暗中放宽
原先的解码内存上界。原有单次准备、资源读取、script 与 audio 限制也保留。
该指标只覆盖保留的解码 RGBA，不代表进程总 RSS、decoder scratch 或驱动 VRAM。

## 诊断与验证

`EngineStatus.scene_gpu_budget` 包含上下文代次、存活不可变字节、私有/预留字节、
不可变预留、退役字节、当前逻辑计划上界、分配数与 lease 数。上界是快照，
不是实测 VRAM 或历史峰值。`scene_decoded_texture_bytes` 表示保留的解码 RGBA
capacity。每输出的 `scene_payload_bytes` 包含其引用的不可变并集与各代私有资源，
直接累加多个输出可能重复计算同一上传。

CPU 回归覆盖资源替换与来源、元数据/动画、sampler 计划、真实 vector capacity、
并集计费、溢出与保留旧布局的 resize 峰值。小型自主构造的离屏回归覆盖共享 ID
及像素、采样策略隔离、像素变更、初始化失败、预检拒绝、独立释放、错误 current
上下文中的退役及新上下文共享域。

显式执行共享 fixture，不写入真实桌面状态：

```bash
EGL_PLATFORM=surfaceless LIBGL_ALWAYS_SOFTWARE=1 LP_NUM_THREADS=2 CARGO_BUILD_JOBS=1 \
  cargo test -p lwe-engine scene::renderer::sharing_tests::offscreen_ \
  -- --ignored --test-threads=1
```

软件 renderer 的离屏结果验证 GL 所有权及像素行为，不代表原生 GPU 内存用量、
可见 Wayland 播放、广泛 Scene 支持、休眠恢复，也不能证明仍超出 CPU 保留预算
或私有 GPU 预算的大场景兼容。测试此分支时应保持原长期桌面验收的源码和进程不变。
