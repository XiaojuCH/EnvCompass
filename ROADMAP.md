# EnvCompass Roadmap

路线图描述的是方向，不是已经实现的能力。当前 Preview 保持 Windows-first、本地优先、只读、无遥测。

## Near-term

- Conda-aware diagnosis（只诊断，不管理环境）
- 更多学生、科研与普通 Python/Node.js 项目的真实覆盖
- 评估有边界的 Rust-side Python import inference；在可靠性足够前不做正则猜测

## Future: EnvCompass Recipes

未来可研究 YOLO / Ultralytics、PyTorch GPU、OpenCV 与 Data Science 等经过验证的配置方案。

Recipes 必须与只读 diagnosis 清楚分离，需要用户明确确认，并优先创建隔离环境，而不是修改系统 Python、PATH 或现有项目。
