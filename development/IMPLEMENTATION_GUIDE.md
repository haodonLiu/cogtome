  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  COGTOME 实现指南（傻瓜版）
  ══════════════════════════
  ▌ 目标读者：代码能力较弱的模型/开发者 目标：根据本文档，能够独立实现 COGTOME 的 Phase 1 全部功能 范围：src/main.rs、src/discovery.rs、src/context.rs、src/engine.rs 不用直接写真实代码，所有逻辑用详细伪代码描述
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  一、项目总览
  1.1 这是什么东西
  COGTOME 是一个用 Rust 编写的命令行程序。它像一个小型操作系统：
  • 用户在命令行输入 cogtome run xxx --input '{...}'
  • 程序扫描磁盘上的 skills/ 目录，找到匹配的领域包（Complex）
  • 按照 Complex → Structure → Motif → Unit 四层模型逐层执行
  • 每个 Unit 是一个独立的外部程序（fork + exec），通过 stdin 输入 JSON，通过 stdout 输出 JSON
  • 最终把执行结果打印到终端
  1.2 项目文件结构
  cogtome/
  ├── Cargo.toml              # Rust 项目配置
  ├── skills/                 # 运行时扫描的业务逻辑目录
  │   ├── units/              # 原子执行体（外部可执行程序）
  │   ├── motifs/             # 编排逻辑（YAML 文件）
  │   ├── structures/         # 业务封装（manifest.yaml）
  │   └── text-processing/    # Complex 示例（SKILL.md）
  └── src/
      ├── main.rs             # CLI 入口，用 clap 解析参数
      ├── discovery.rs        # 磁盘扫描，加载目录结构
      ├── context.rs          # 执行上下文，变量解析
      └── engine.rs           # 执行引擎，fork 进程、调度 Motif
  1.3 Cargo.toml 依赖
  [package]
  name = "cogtome"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  clap = { version = "4.5", features = ["derive"] }
  tokio = { version = "1.37", features = ["full"] }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  serde_yaml = "0.9"
  anyhow = "1.0"
  ▌ 注意：所有 I/O（文件读写、进程执行）都用 tokio 的异步 API，不要用标准库的阻塞 API。
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  二、核心数据模型（先定义，后实现）
  2.1 discovery.rs 中的模型
  // Unit 定义
  struct UnitDef {
      name: String,
      path: PathBuf,              // 可执行文件绝对路径
      skill_md_path: Option<PathBuf>,
  }

  // Motif 定义
  struct MotifDef {
      name: String,
      path: PathBuf,              // YAML 文件路径
      raw_yaml: String,           // 整个文件原始内容
  }

  // manifest.yaml 解析后的结构
  struct Manifest {
      name: String,
      #[serde(rename = "type")]
      type_field: String,         // 固定值 "structure"
      motifs: Vec<String>,
      input_schema: serde_json::Value,
      output_schema: serde_json::Value,
      resources: serde_json::Value,
  }

  // Structure 定义
  struct StructureDef {
      name: String,
      path: PathBuf,              // 目录路径
      manifest: Manifest,
  }

  // Complex 中引用的 Structure
  struct StructureRef {
      name: String,
      path: String,
      summary: Option<String>,
      scenarios: Vec<String>,
      weight: f64,
  }

  // Complex 配置
  struct ComplexConfig {
      default_timeout: u64,       // 默认 30
      log_retention: String,      // 如 "1d"
  }

  // Complex 定义
  struct ComplexDef {
      name: String,
      path: PathBuf,
      description: String,        // 必须非空，否则不参与发现
      structures: Vec<StructureRef>,
      config: ComplexConfig,
  }

  // 全局索引
  struct SkillIndex {
      units: HashMap<String, UnitDef>,
      motifs: HashMap<String, MotifDef>,
      structures: HashMap<String, StructureDef>,
      complexes: HashMap<String, ComplexDef>,
  }
  2.2 context.rs 中的模型
  // Unit 执行后的结果
  struct StepState {
      output: serde_json::Value,  // stdout 解析后的 JSON
      exit_code: i32,
  }

  // 执行上下文（可 Clone）
  struct ExecContext {
      params: serde_json::Value,                    // 用户原始参数
      steps: HashMap<String, StepState>,            // 已执行步骤的结果
      env_vars: HashMap<String, String>,            // 环境变量快照
  }
  2.3 engine.rs 中的模型
  // Motif 解析后的结构
  struct ParsedMotif {
      name: String,
      flow: Vec<FlowStep>,
      return_fields: HashMap<String, String>,   // key=字段名, value=模板表达式
      foreach_block: Option<ForeachBlock>,
  }

  struct FlowStep {
      name: String,
      unit: String,
      input: serde_json::Value,       // 模板 JSON
      condition: Option<String>,      // if 条件
  }

  struct ForeachBlock {
      over: String,                   // 数组表达式，如 "${params.files}"
      as_var: String,                 // 迭代变量名
      max_iterations: usize,          // 默认 50，硬上限 500
      on_error: String,               // "fail_fast" 或 "continue"
      flow: Vec<FlowStep>,
      aggregate: AggregateConfig,
  }

  struct AggregateConfig {
      mode: String,                   // "array", "object", "sum", "join"
      map_template: Option<serde_json::Value>,
      key_template: Option<String>,
      sum_expr: Option<String>,
      join_expr: Option<String>,
      separator: Option<String>,
  }

  // Structure 执行结果
  struct ExecutionResult {
      output: serde_json::Value,
      logs: Vec<StepLog>,
  }

  struct StepLog {
      step_name: String,
      unit_name: String,
      input: serde_json::Value,
      output: serde_json::Value,
      exit_code: i32,
      duration_ms: u64,
  }
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  三、模块逐一实现
  3.1 discovery.rs —— 磁盘扫描与发现
  职责：启动时扫描 skills/ 目录，构建 SkillIndex。
  3.1.1 build_index(skills_root) 入口函数
  async fn build_index(skills_root: &Path) -> Result<SkillIndex> {
      let mut index = SkillIndex::new();

      // 1. 扫描 units/
      let units_dir = skills_root.join("units");
      if units_dir.exists() {
          for entry in fs::read_dir(units_dir).await? {
              let entry = entry?;
              if entry.file_type().await?.is_dir() {
                  let name = entry.file_name().to_string_lossy().to_string();
                  let bin_path = entry.path().join("bin").join(&name);
                  let skill_md = entry.path().join("SKILL.md");
                  index.units.insert(name.clone(), UnitDef {
                      name,
                      path: bin_path,
                      skill_md_path: if skill_md.exists() { Some(skill_md) } else { None },
                  });
              }
          }
      }

      // 2. 扫描 motifs/
      let motifs_dir = skills_root.join("motifs");
      if motifs_dir.exists() {
          for entry in fs::read_dir(motifs_dir).await? {
              let entry = entry?;
              let path = entry.path();
              if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                  let name = path.file_stem().unwrap().to_string_lossy().to_string();
                  let raw = fs::read_to_string(&path).await?;
                  index.motifs.insert(name.clone(), MotifDef { name, path, raw_yaml: raw });
              }
          }
      }

      // 3. 扫描 structures/
      let structures_dir = skills_root.join("structures");
      if structures_dir.exists() {
          for entry in fs::read_dir(structures_dir).await? {
              let entry = entry?;
              if entry.file_type().await?.is_dir() {
                  let manifest_path = entry.path().join("manifest.yaml");
                  if manifest_path.exists() {
                      let raw = fs::read_to_string(&manifest_path).await?;
                      let manifest: Manifest = serde_yaml::from_str(&raw)?;
                      let name = manifest.name.clone();
                      index.structures.insert(name.clone(), StructureDef {
                          name,
                          path: entry.path(),
                          manifest,
                      });
                  }
              }
          }
      }

      // 4. 扫描 Complex（skills 根目录下直接子目录中的 SKILL.md）
      for entry in fs::read_dir(skills_root).await? {
          let entry = entry?;
          if entry.file_type().await?.is_dir() {
              let skill_md = entry.path().join("SKILL.md");
              if skill_md.exists() {
                  let raw = fs::read_to_string(&skill_md).await?;
                  let front_matter = extract_front_matter(&raw);
                  let complex: ComplexDef = serde_yaml::from_str(&front_matter)?;

                  // 关键纪律：description 必须非空
                  if !complex.description.trim().is_empty() {
                      index.complexes.insert(complex.name.clone(), complex);
                  }
              }
          }
      }

      Ok(index)
  }
  3.1.2 extract_front_matter(content) 辅助函数
  fn extract_front_matter(content: &str) -> String {
      let lines: Vec<&str> = content.lines().collect();
      let mut start = None;
      let mut end = None;

      for (i, line) in lines.iter().enumerate() {
          if line.trim() == "---" {
              if start.is_none() {
                  start = Some(i + 1);
              } else {
                  end = Some(i);
                  break;
              }
          }
      }

      match (start, end) {
          (Some(s), Some(e)) => lines[s..e].join("\n"),
          _ => content.to_string(), // 容错：找不到分隔符就返回全部
      }
  }
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  3.2 context.rs —— 变量解析与表达式引擎
  职责：把模板字符串 "${params.text}" 解析为实际 JSON 值。
  3.2.1 resolve_expression(ctx, expr) 入口
  fn resolve_expression(ctx: &ExecContext, expr: &str) -> Result<serde_json::Value> {
      let trimmed = expr.trim();

      // 情况1：纯 "${...}" 包裹
      if trimmed.starts_with("${") && trimmed.ends_with('}') {
          let inner = &trimmed[2..trimmed.len()-1];
          return evaluate_inner(ctx, inner.trim());
      }

      // 情况2：混合模板，如 "hello ${name}"
      // 找到所有 ${...}，替换为字符串值，最后返回 String
      let mut result = expr.to_string();
      for cap in find_all_braces(expr) {
          let val = evaluate_inner(ctx, &cap.inner)?;
          result = result.replace(&cap.full, &json_to_string(&val));
      }
      Ok(Value::String(result))
  }
  3.2.2 evaluate_inner(ctx, inner) 核心解析
  fn evaluate_inner(ctx: &ExecContext, inner: &str) -> Result<serde_json::Value> {
      let parts: Vec<&str> = inner.split('.').collect();

      match parts[0] {
          "params" => navigate_json(&ctx.params, &parts[1..]),

          "steps" => {
              let step_name = parts[1];
              let state = ctx.steps.get(step_name)
                  .ok_or_else(|| anyhow!("Step '{}' not found", step_name))?;

              if parts.get(2) == Some(&"exit_code") {
                  return Ok(json!(state.exit_code));
              }
              // parts[2] == "output"
              navigate_json(&state.output, &parts[3..])
          }

          "env" => {
              let key = parts[1];
              let val = ctx.env_vars.get(key).cloned().unwrap_or_default();
              Ok(Value::String(val))
          }

          "item" => {
              // foreach 把当前项注入到 params["__item"]
              navigate_json(&ctx.params, &["__item"])
          }

          _ => Err(anyhow!("Unknown variable prefix: {}", parts[0]))
      }
  }
  3.2.3 navigate_json(root, path_parts) JSON 导航
  fn navigate_json(root: &Value, path_parts: &[&str]) -> Result<Value> {
      let mut current = root.clone();

      for part in path_parts {
          // 处理数组索引，如 "numbers[0]" 或 "numbers[-1]"
          if let Some(open) = part.find('[') {
              let field = &part[..open];
              let close = part.rfind(']').ok_or_else(|| anyhow!("Missing ]"))?;
              let idx_str = &part[open+1..close];
              let idx: i32 = idx_str.parse()?;

              if !field.is_empty() {
                  current = current.get(field)
                      .ok_or_else(|| anyhow!("Field not found: {}", field))?
                      .clone();
              }

              let arr = current.as_array()
                  .ok_or_else(|| anyhow!("Not an array"))?;
              let actual = if idx < 0 { arr.len() as i32 + idx } else { idx };
              current = arr.get(actual as usize)
                  .ok_or_else(|| anyhow!("Index out of bounds"))?
                  .clone();
          } else {
              current = current.get(part)
                  .ok_or_else(|| anyhow!("Field not found: {}", part))?
                  .clone();
          }
      }

      Ok(current)
  }
  3.2.4 resolve_json_template(ctx, template) 递归解析模板 JSON
  fn resolve_json_template(ctx: &ExecContext, template: &Value) -> Result<Value> {
      match template {
          Value::String(s) => resolve_expression(ctx, s),

          Value::Object(map) => {
              let mut new = Map::new();
              for (k, v) in map {
                  new.insert(k.clone(), resolve_json_template(ctx, v)?);
              }
              Ok(Value::Object(new))
          }

          Value::Array(arr) => {
              let mut new = Vec::new();
              for item in arr {
                  new.push(resolve_json_template(ctx, item)?);
              }
              Ok(Value::Array(new))
          }

          // Number, Bool, Null 直接返回
          other => Ok(other.clone()),
      }
  }
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  3.3 engine.rs —— 执行引擎
  3.3.1 UnitRunner 结构体与 run() 方法
  struct UnitRunner {
      index: Arc<SkillIndex>,
      timeout_ms: u64,
  }

  impl UnitRunner {
      async fn run(&self, unit_name: &str, input: Value, ctx: &ExecContext) -> Result<StepState> {
          // 1. 查找 Unit
          let unit_def = self.index.units.get(unit_name)
              .ok_or_else(|| anyhow!("Unit not found: {}", unit_name))?;

          if !unit_def.path.exists() {
              return Err(anyhow!("Unit binary not found at {:?}", unit_def.path));
          }

          // 2. 构造环境变量
          let mut cmd = tokio::process::Command::new(&unit_def.path);
          cmd.env("COGTOME_UNIT_MODE", "1")           // 禁止嵌套调用
             .env("COGTOME_EXECUTION_ID", generate_uuid())
             .env("COGTOME_TRACE_ID", ctx.env_vars.get("COGTOME_TRACE_ID").unwrap_or(&"".to_string()))
             .env("COGTOME_LOG_LEVEL", "info")
             .env("COGTOME_TIMEOUT_MS", self.timeout_ms.to_string())
             .stdin(Stdio::piped())
             .stdout(Stdio::piped())
             .stderr(Stdio::piped());

          // 3. 启动子进程
          let mut child = cmd.spawn()?;

          // 4. 写入 stdin JSON
          let input_bytes = serde_json::to_vec(&input)?;
          if let Some(mut stdin) = child.stdin.take() {
              stdin.write_all(&input_bytes).await?;
              stdin.shutdown().await?;  // 必须关闭，让子进程知道输入结束
          }

          // 5. 等待输出，带超时
          let result = tokio::time::timeout(
              Duration::from_millis(self.timeout_ms),
              child.wait_with_output()
          ).await;

          match result {
              Ok(Ok(output)) => {
                  let exit_code = output.status.code().unwrap_or(-1);
                  let stdout = String::from_utf8_lossy(&output.stdout);
                  let stderr = String::from_utf8_lossy(&output.stderr);

                  if exit_code != 0 {
                      return Err(anyhow!("Unit exited with code {}: {}", exit_code, stderr));
                  }

                  let parsed: Value = serde_json::from_str(&stdout)?;
                  Ok(StepState { output: parsed, exit_code })
              }

              Ok(Err(e)) => Err(anyhow!("Failed to run unit: {}", e)),

              Err(_) => {
                  child.kill().await.ok();
                  Err(anyhow!("Unit timed out after {}ms", self.timeout_ms))
              }
          }
      }
  }
  3.3.2 MotifEngine 与 YAML Motif 执行
  struct MotifEngine {
      unit_runner: Arc<UnitRunner>,
      index: Arc<SkillIndex>,
  }

  impl MotifEngine {
      async fn run_motif(&self, motif_name: &str, ctx: &mut ExecContext) -> Result<Value> {
          let motif_def = self.index.motifs.get(motif_name)
              .ok_or_else(|| anyhow!("Motif not found: {}", motif_name))?;

          let parsed: ParsedMotif = serde_yaml::from_str(&motif_def.raw_yaml)?;

          // 如果有 foreach，走 foreach 逻辑
          if let Some(foreach) = parsed.foreach_block {
              return self.run_foreach(foreach, ctx).await;
          }

          // 普通串行 flow
          for step in &parsed.flow {
              // if 条件判断
              if let Some(cond) = &step.condition {
                  let val = resolve_expression(ctx, cond)?;
                  if !is_truthy(&val) {
                      continue;  // 跳过此 step
                  }
              }

              // 解析 input 模板
              let resolved_input = resolve_json_template(ctx, &step.input)?;

              // 执行 Unit
              let state = self.unit_runner.run(&step.unit, resolved_input, ctx).await?;
              ctx.steps.insert(step.name.clone(), state);
          }

          // 构造 return 值
          let mut result = Map::new();
          for (key, expr) in &parsed.return_fields {
              result.insert(key.clone(), resolve_expression(ctx, expr)?);
          }
          Ok(Value::Object(result))
      }
  }
  is_truthy(value) 真值判断
  fn is_truthy(value: &Value) -> bool {
      match value {
          Value::Bool(b) => *b,
          Value::Null => false,
          Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
          Value::String(s) => !s.is_empty() && s != "false" && s != "0",
          Value::Array(a) => !a.is_empty(),
          Value::Object(o) => !o.is_empty(),
      }
  }
  run_foreach() 完整逻辑
  async fn run_foreach(&self, foreach: ForeachBlock, ctx: &mut ExecContext) -> Result<Value> {
      // 1. 解析 over 为数组
      let over_val = resolve_expression(ctx, &foreach.over)?;
      let items = over_val.as_array()
          .ok_or_else(|| anyhow!("foreach.over must resolve to an array"))?;

      // 2. 空数组 → 返回对应类型的空值
      if items.is_empty() {
          return Ok(match foreach.aggregate.mode.as_str() {
              "array" => Value::Array(vec![]),
              "object" => Value::Object(Map::new()),
              "sum" => json!(0),
              "join" => Value::String("".to_string()),
              _ => Value::Null,
          });
      }

      // 3. max_iterations 安全检查（硬上限 500）
      let max_iter = foreach.max_iterations.min(500);
      if items.len() > max_iter {
          return Err(anyhow!(
              "Foreach attempted {} iterations (limit: {}). \
               Hint: Increase max_iterations in cogtome.toml or ask Agent to batch process.",
              items.len(), max_iter
          ));
      }

      // 4. 保存外部 steps 快照（隔离 foreach 内部修改）
      let snapshot = ctx.steps.clone();
      let mut aggregate_results: Vec<Value> = Vec::new();

      // 5. 迭代执行
      for (idx, item) in items.iter().enumerate() {
          let mut sub_ctx = ExecContext {
              params: ctx.params.clone(),
              steps: snapshot.clone(),  // 从快照开始，不继承之前迭代的 steps
              env_vars: ctx.env_vars.clone(),
          };

          // 注入迭代变量到 params
          if let Value::Object(ref mut map) = sub_ctx.params {
              map.insert("__item".to_string(), item.clone());
              map.insert("__index".to_string(), json!(idx));
          }

          let mut success = true;
          let mut error_msg = None;

          for step in &foreach.flow {
              // if 条件
              if let Some(cond) = &step.condition {
                  if !is_truthy(&resolve_expression(&sub_ctx, cond)?) {
                      continue;
                  }
              }

              let input = resolve_json_template(&sub_ctx, &step.input)?;

              match self.unit_runner.run(&step.unit, input, &sub_ctx).await {
                  Ok(state) => { sub_ctx.steps.insert(step.name.clone(), state); }
                  Err(e) => {
                      success = false;
                      error_msg = Some(e.to_string());
                      if foreach.on_error == "fail_fast" {
                          return Err(anyhow!("Foreach iteration {} failed: {}", idx, e));
                      }
                      break;  // continue 模式：中断当前 iteration，进入 aggregate
                  }
              }
          }

          // 6. 收集 aggregate
          if !success && foreach.on_error == "continue" {
              let mut entry = Map::new();
              entry.insert("__error".to_string(), Value::String(error_msg.unwrap_or_default()));
              aggregate_results.push(Value::Object(entry));
          } else if success {
              if let Some(template) = &foreach.aggregate.map_template {
                  aggregate_results.push(resolve_json_template(&sub_ctx, template)?);
              }
          }
      }

      // 7. 按 mode 聚合
      match foreach.aggregate.mode.as_str() {
          "array" => Ok(Value::Array(aggregate_results)),

          "object" => {
              let mut obj = Map::new();
              for (i, entry) in aggregate_results.into_iter().enumerate() {
                  obj.insert(i.to_string(), entry);
              }
              Ok(Value::Object(obj))
          }

          "sum" => {
              let total: f64 = aggregate_results.iter()
                  .filter_map(|v| v.as_f64()).sum();
              Ok(json!(total))
          }

          "join" => {
              let sep = foreach.aggregate.separator.as_deref().unwrap_or("");
              let parts: Vec<String> = aggregate_results.iter()
                  .filter_map(|v| v.as_str().map(|s| s.to_string()))
                  .collect();
              Ok(Value::String(parts.join(sep)))
          }

          _ => Ok(Value::Null),
      }
  }
  3.3.3 StructureExecutor
  struct StructureExecutor {
      motif_engine: Arc<MotifEngine>,
      index: Arc<SkillIndex>,
  }

  impl StructureExecutor {
      async fn run_structure(&self, name: &str, params: Value) -> Result<ExecutionResult> {
          let def = self.index.structures.get(name)
              .ok_or_else(|| anyhow!("Structure not found: {}", name))?;

          // 1. 检查 required 字段（简化版 input_schema 校验）
          if let Some(required) = def.manifest.input_schema.get("required").and_then(|v| v.as_array()) {
              for field in required {
                  let f = field.as_str().unwrap_or("");
                  if params.get(f).is_none() {
                      return Err(anyhow!("Missing required input field: {}", f));
                  }
              }
          }

          // 2. 初始化上下文
          let mut ctx = ExecContext {
              params,
              steps: HashMap::new(),
              env_vars: std::env::vars().collect(),
          };

          // 3. 依次执行所有 Motif
          let mut final_output = Value::Null;
          for motif_name in &def.manifest.motifs {
              final_output = self.motif_engine.run_motif(motif_name, &mut ctx).await?;
          }

          Ok(ExecutionResult {
              output: final_output,
              logs: vec![],  // Phase 1 可简化日志
          })
      }
  }
  3.3.4 ComplexExecutor
  struct ComplexExecutor {
      structure_executor: Arc<StructureExecutor>,
      index: Arc<SkillIndex>,
  }

  impl ComplexExecutor {
      async fn run_complex(&self, name: &str, params: Value) -> Result<ExecutionResult> {
          let def = self.index.complexes.get(name)
              .ok_or_else(|| anyhow!("Complex not found: {}", name))?;

          let first = def.structures.first()
              .ok_or_else(|| anyhow!("Complex has no structures"))?;

          // Phase 1 直接执行第一个 Structure
          self.structure_executor.run_structure(&first.name, params).await
      }
  }
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  3.4 main.rs —— CLI 入口
  3.4.1 CLI 参数定义（clap derive）
  #[derive(Parser)]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }

  #[derive(Subcommand)]
  enum Commands {
      Discover,
      Unit(UnitArgs),
      Motif(MotifArgs),
      Structure(StructureArgs),
      Run(RunArgs),
  }

  #[derive(Args)]
  struct RunArgs {
      complex_name: String,
      #[arg(short, long)]
      input: Option<String>,
      #[arg(long)]
      stdin: bool,
  }

  #[derive(Args)]
  struct UnitArgs {
      #[command(subcommand)]
      command: UnitCommands,
  }

  #[derive(Subcommand)]
  enum UnitCommands {
      List,
      Run(UnitRunArgs),
  }

  #[derive(Args)]
  struct UnitRunArgs {
      unit_name: String,
      #[arg(short, long)]
      input: Option<String>,
      #[arg(long)]
      stdin: bool,
  }
  3.4.2 main() 函数
  #[tokio::main]
  async fn main() -> Result<()> {
      let cli = Cli::parse();

      // 1. 确定 skills 根目录
      let skills_root = std::env::var("COGTOME_SKILLS_PATH")
          .map(PathBuf::from)
          .unwrap_or_else(|| PathBuf::from("./skills"));

      // 2. 构建索引
      let index = discovery::build_index(&skills_root).await?;
      let index = Arc::new(index);

      // 3. 初始化引擎（层层 Arc 共享）
      let unit_runner = Arc::new(UnitRunner {
          index: index.clone(),
          timeout_ms: 30000,
      });
      let motif_engine = Arc::new(MotifEngine {
          unit_runner: unit_runner.clone(),
          index: index.clone(),
      });
      let structure_exec = Arc::new(StructureExecutor {
          motif_engine: motif_engine.clone(),
          index: index.clone(),
      });
      let complex_exec = ComplexExecutor {
          structure_executor: structure_exec.clone(),
          index: index.clone(),
      };

      // 4. 命令分发
      match cli.command {
          Commands::Discover => {
              for (name, c) in &index.complexes {
                  println!("{}  {}", name, c.description.lines().next().unwrap_or(""));
              }
          }

          Commands::Unit(args) => match args.command {
              UnitCommands::List => {
                  for name in index.units.keys() { println!("{}", name); }
              }
              UnitCommands::Run(r) => {
                  let input = parse_input(&r.input, r.stdin)?;
                  let ctx = ExecContext {
                      params: input.clone(),
                      steps: HashMap::new(),
                      env_vars: std::env::vars().collect(),
                  };
                  let result = unit_runner.run(&r.unit_name, input, &ctx).await?;
                  println!("{}", serde_json::to_string_pretty(&result.output)?);
              }
          }

          Commands::Motif(args) => {
              let input = parse_input(&args.input, args.stdin)?;
              let mut ctx = ExecContext {
                  params: input,
                  steps: HashMap::new(),
                  env_vars: std::env::vars().collect(),
              };
              let result = motif_engine.run_motif(&args.motif_name, &mut ctx).await?;
              println!("{}", serde_json::to_string_pretty(&result)?);
          }

          Commands::Structure(args) => {
              let input = parse_input(&args.input, args.stdin)?;
              let result = structure_exec.run_structure(&args.structure_name, input).await?;
              println!("{}", serde_json::to_string_pretty(&result.output)?);
          }

          Commands::Run(args) => {
              let input = parse_input(&args.input, args.stdin)?;
              let result = complex_exec.run_complex(&args.complex_name, input).await?;
              println!("{}", serde_json::to_string_pretty(&result.output)?);
          }
      }

      Ok(())
  }
  3.4.3 parse_input() 辅助函数
  async fn parse_input(arg: &Option<String>, use_stdin: bool) -> Result<Value> {
      if use_stdin {
          let mut buf = String::new();
          tokio::io::stdin().read_to_string(&mut buf).await?;
          return Ok(serde_json::from_str(&buf)?);
      }
      if let Some(s) = arg {
          return Ok(serde_json::from_str(s)?);
      }
      Ok(Value::Object(Map::new()))  // 默认空对象
  }
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  四、边界情况处理清单
   场景                                预期行为
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   skills/ 目录不存在                  build_index 返回空 SkillIndex，不报错
   Unit 没有 +x 权限                   Command::spawn 报错，向上传播
   Unit 输出非合法 JSON                serde_json::from_str 失败，返回错误
   Unit 返回非 0 退出码                返回错误，包含 stderr 内容
   Unit 超时                           timeout 触发，kill 子进程，返回超时错误
   Motif 引用不存在的 Unit             index.units.get() 返回 None，报错
   Motif 的 if 条件解析失败            视为 false，跳过 step，打印 warning
   Motif 的 return 引用不存在的 step   抛错，明确编程错误
   foreach.over 为 null 或非数组       视为空数组，不报错
   foreach 长度超过 max_iterations     抛 MaxIterationsExceeded
   foreach + fail_fast + 迭代失败      不产出 aggregate，直接抛错
   foreach + continue + 迭代失败       __error 记录，aggregate 中该项保留
   aggregate.map 引用不存在的字段      静默填充 null，打印 warning
   Structure 缺少 required 字段        执行前抛错，不进入 Motif
   Complex 没有 description            build_index 跳过，不加入索引
   Complex 没有 structures             run_complex 报错
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  五、执行流程时序图
  cogtome run text-processing --input '{"text":"hello"}'

  [main.rs]
    │
    ▼ 解析 CLI → RunArgs
    ▼ build_index("./skills")
  [discovery.rs]
    ├── 扫描 units/ → index.units
    ├── 扫描 motifs/ → index.motifs
    ├── 扫描 structures/ → index.structures
    └── 扫描 SKILL.md → index.complexes
    │
    ▼ 返回 SkillIndex
  [main.rs]
    ▼ 初始化 UnitRunner → MotifEngine → StructureExecutor → ComplexExecutor
    ▼ complex_exec.run_complex("text-processing", params)
  [ComplexExecutor]
    ▼ 找 Complex → 取第一个 Structure "text-pipeline"
    ▼ structure_exec.run_structure("text-pipeline", params)
  [StructureExecutor]
    ▼ 校验 required: ["text"] ✓
    ▼ 初始化 ExecContext { params, steps: {} }
    ▼ 执行 manifest.motifs[0] = "text-transform"
    ▼ motif_engine.run_motif("text-transform", ctx)
  [MotifEngine]
    ▼ 解析 YAML → ParsedMotif
    ▼ flow[0]: name="upper", unit="text-uppercase"
        ▼ resolve_json_template(ctx, {"text": "${params.text}"}) → {"text": "hello"}
        ▼ unit_runner.run("text-uppercase", {"text":"hello"}, ctx)
  [UnitRunner]
            ▼ Command::new(".../bin/text-uppercase").envs(...).spawn()
            ▼ stdin.write({"text":"hello"})
            ▼ wait_with_output(timeout=30s)
            ▼ stdout = {"result": "HELLO"}
            ▼ 解析 JSON → StepState
        ◄ 返回 StepState
        ▼ ctx.steps.insert("upper", state)
    ▼ flow[1]: name="rev", unit="text-reverse"
        ▼ 同上 → ctx.steps.insert("rev", {"result":"olleh"})
    ▼ 解析 return:
        upper = "${steps.upper.output.result}" → "HELLO"
        reversed = "${steps.rev.output.result}" → "olleh"
    ▼ 返回 {"upper": "HELLO", "reversed": "olleh"}
  ◄ 逐层返回
  [main.rs]
    ▼ serde_json::to_string_pretty()
    ▼ 打印到 stdout
  ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
  六、Phase 1 必须实现 vs 不需要
   功能                                                                             Phase 1
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   CLI 参数解析（clap）                                                             ✅
   discover / unit list/run / motif list/run / structure list/run / run <complex>   ✅
   Discovery 四级扫描                                                               ✅
   YAML Motif 解析（flow, return）                                                  ✅
   foreach 循环（over, as, flow, aggregate, max_iterations, on_error）              ✅
   表达式引擎（${params} / ${steps} / ${env} / ${item} / [索引]）                   ✅
   if 条件跳过                                                                      ✅
   Unit fork/exec + stdin/stdout JSON + 超时                                        ✅
   COGTOME_UNIT_MODE=1 注入                                                         ✅
   foreach.parallel: true                                                           ❌
   Python/Shell Motif                                                               ❌
   Complex 智能选择 Structure                                                       ❌
   Complex 私有 Unit                                                                ❌
   daemon / logs / pack                                                             ❌
   Schema 深度类型校验                                                              ❌
   表达式函数 filter() / map()                                                      ❌