#!/bin/bash

# DD 3D 彩票后端部署到 GitHub 脚本
# 用于将后端代码部署到独立的 GitHub 仓库

set -e

# 配置变量
PROJECT_NAME="dd-3d-lottery-backend"
GITHUB_USERNAME="your-username"  # 请替换为实际的 GitHub 用户名
GITHUB_REPO="https://github.com/${GITHUB_USERNAME}/${PROJECT_NAME}.git"
BACKEND_DIR="dd_3d_lottery_backend"
TEMP_DIR="/tmp/${PROJECT_NAME}_deploy"

echo "🚀 开始部署 DD 3D 彩票后端到 GitHub..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 请在 dd_3d_lottery_backend 目录中运行此脚本"
    exit 1
fi

# 检查 Git 状态
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  警告: 工作目录有未提交的更改"
    echo "请先提交所有更改:"
    echo "  git add ."
    echo "  git commit -m 'feat: prepare for deployment'"
    read -p "是否继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 创建临时目录
echo "📁 创建临时目录..."
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"
cd "$TEMP_DIR"

# 克隆或初始化仓库
if [ -d "$PROJECT_NAME" ]; then
    echo "📥 更新现有仓库..."
    cd "$PROJECT_NAME"
    git pull origin main
else
    echo "📥 克隆仓库..."
    git clone "$GITHUB_REPO" || {
        echo "❌ 无法克隆仓库，请确保仓库存在且可访问"
        echo "请先在 GitHub 上创建仓库: https://github.com/new"
        echo "仓库名称: $PROJECT_NAME"
        exit 1
    }
    cd "$PROJECT_NAME"
fi

# 清理现有文件（保留 .git）
echo "🧹 清理现有文件..."
find . -not -path './.git*' -not -name '.' -not -name '..' -delete

# 复制后端文件
echo "📋 复制后端文件..."
cp -r "/home/lc/luckee_dao/dd_3d_lottery/$BACKEND_DIR"/* .

# 创建独立的 README.md
echo "📝 创建独立的 README.md..."
cat > README.md << 'EOF'
# DD 3D 彩票智能合约 (后端)

基于 CosmWasm 的去中心化 3D 彩票智能合约系统。

**版本**: v0.1.0

## 🎯 功能特性

### 核心功能
- **三阶段投注系统**：承诺阶段、中奖揭秘阶段、结算阶段
- **自动阶段切换**：基于区块链高度自动切换阶段
- **公平随机数生成**：基于所有参与者随机数生成中奖号码
- **单一奖项设计**：只有一等奖，没有二等奖与三等奖
- **安全防护**：防重入攻击、访问控制、输入验证

### 技术特性
- **CosmWasm 2.2.2**：使用最新的 CosmWasm 框架
- **cw-storage-plus 2.x**：高效的存储管理
- **完整测试覆盖**：单元测试和集成测试
- **生产就绪**：包含部署脚本和 CI/CD 配置

## 🛠️ 快速开始

### 1. 环境要求

- Rust 1.70+
- CosmWasm CLI (wasmd)
- cosmwasm-opt (可选，用于优化)

### 2. 构建合约

```bash
# 构建合约
cargo build --release --target wasm32-unknown-unknown

# 优化 WASM (可选)
cosmwasm-opt target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm \
  -o target/wasm32-unknown-unknown/release/dd_3d_lottery_optimized.wasm
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test integration
```

### 4. 部署合约

```bash
# 使用部署脚本
./scripts/deploy.sh

# 或手动部署
wasmd tx wasm store target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm \
  --from <your-key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --chain-id <chain-id> \
  --node <rpc-url> \
  --yes
```

## 📁 项目结构

```
dd-3d-lottery-backend/
├── src/                    # 源代码
│   ├── contract.rs         # 合约入口点
│   ├── execute.rs          # 执行消息处理
│   ├── query.rs            # 查询消息处理
│   ├── msg.rs              # 消息定义
│   ├── state.rs            # 状态管理
│   ├── error.rs            # 错误定义
│   ├── phase_manager.rs    # 阶段管理
│   ├── lottery_logic.rs    # 彩票逻辑
│   └── reward_system.rs    # 奖励系统
├── tests/                  # 测试文件
├── scripts/                # 脚本文件
├── docs/                   # 项目文档
├── schema/                 # JSON Schema
├── Cargo.toml             # 项目配置
└── README.md              # 项目说明
```

## 🔒 安全特性

- **防重入保护**：防止重入攻击
- **访问控制**：基于角色的权限管理
- **输入验证**：严格的参数验证
- **溢出保护**：使用 SafeMath 防止整数溢出

## 🧪 测试

项目包含完整的测试覆盖：

- **单元测试**：测试各个模块的功能
- **集成测试**：测试完整的业务流程
- **安全测试**：测试安全机制
- **边界测试**：测试边界条件

运行测试：

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_phase_detection
cargo test test_lottery_logic
cargo test test_reward_distribution
```

## 🚀 部署

### 自动部署

使用提供的部署脚本：

```bash
./scripts/deploy.sh
```

### 手动部署

1. **上传合约代码**：
```bash
wasmd tx wasm store target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm \
  --from <your-key> \
  --gas auto \
  --gas-adjustment 1.3 \
  --chain-id <chain-id> \
  --node <rpc-url> \
  --yes
```

2. **实例化合约**：
```bash
wasmd tx wasm instantiate <code-id> '{"admin":"<admin-address>",...}' \
  --from <your-key> \
  --admin <admin-address> \
  --label "DD 3D Lottery" \
  --chain-id <chain-id> \
  --node <rpc-url> \
  --yes
```

## 📈 CI/CD

项目配置了完整的 CI/CD 流程：

- **代码格式检查**：确保代码风格一致
- **代码质量检查**：使用 clippy 进行静态分析
- **构建和测试**：自动构建和运行测试
- **WASM 优化**：自动优化 WASM 文件
- **安全扫描**：进行安全审计
- **文档生成**：自动生成文档

## 📚 文档

- `docs/` - 项目文档
- `schema/` - JSON Schema 文件
- 代码注释 - 详细的函数和结构体注释

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 支持

如果您遇到问题或有任何问题，请：

1. 查看 [文档](docs/)
2. 搜索 [Issues](https://github.com/your-org/dd-3d-lottery-backend/issues)
3. 创建新的 Issue

---

**DD 3D 彩票智能合约** - 构建公平透明的去中心化彩票系统 🎲
EOF

# 创建 .gitignore
echo "📝 创建 .gitignore..."
cat > .gitignore << 'EOF'
# Rust
/target/
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Environment
.env
.env.local
.env.production

# Build artifacts
dist/
build/
EOF

# 提交更改
echo "💾 提交更改..."
git add .
git commit -m "feat: deploy DD 3D lottery backend

- 完整的 CosmWasm 智能合约
- 三阶段投注系统
- 安全防护机制
- 完整测试覆盖
- 生产就绪配置"

# 推送到 GitHub
echo "🚀 推送到 GitHub..."
git push origin main

# 清理临时目录
echo "🧹 清理临时目录..."
cd /home/lc/luckee_dao/dd_3d_lottery
rm -rf "$TEMP_DIR"

echo "✅ 后端部署完成！"
echo "📦 仓库地址: $GITHUB_REPO"
echo "🔗 GitHub 页面: https://github.com/$GITHUB_USERNAME/$PROJECT_NAME"
echo ""
echo "下一步："
echo "1. 在 GitHub 上配置 Actions 进行 CI/CD"
echo "2. 设置环境变量和密钥"
echo "3. 配置自动部署到测试网络"
