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

## 📁 项目结构

```
backend/
├── src/                    # 源代码
│   ├── contract.rs         # 合约入口点
│   ├── execute.rs          # 执行消息处理
│   ├── query.rs            # 查询消息处理
│   ├── msg.rs              # 消息定义
│   ├── state.rs            # 状态管理
│   ├── error.rs            # 错误定义
│   ├── phase_manager.rs    # 阶段管理
│   ├── lottery_logic.rs    # 彩票逻辑
│   ├── reward_system.rs    # 奖励系统
│   └── lib.rs              # 库入口
├── tests/                  # 测试文件
├── scripts/                # 脚本文件
├── docs/                   # 项目文档
├── schema/                 # JSON Schema
├── Cargo.toml             # 项目配置
└── README.md              # 项目说明
```

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

## 📋 合约接口

### 实例化消息

```json
{
  "admin": "cosmwasm1...",
  "service_fee_rate": "0.1",
  "min_bet_amount": "1000",
  "max_bet_amount": "1000000",
  "bet_denom": "uusd",
  "pause_requested": false
}
```

### 执行消息

```json
{
  "place_bet": {
    "commitment_hash": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456"
  }
}
```

```json
{
  "reveal_random": {
    "lucky_numbers": [123, 123, 456, 456],
    "random_seed": "user_random_string"
  }
}
```

```json
{
  "settle_lottery": {}
}
```

### 查询消息

```json
{
  "get_current_session": {}
}
```

```json
{
  "get_participant_info": {
    "participant": "cosmwasm1..."
  }
}
```

## 🎯 三阶段系统

### 阶段划分

| 区块链高度 % 10000 | 阶段 | 功能 |
|-------------------|------|------|
| 0-5999 | 承诺阶段 | 用户投注并设置随机数 |
| 6000-8999 | 中奖揭秘阶段 | 用户揭秘随机数 |
| 9000-9999 | 结算阶段 | 计算中奖号码并分配奖金 |

### 投注规则

- 用户转移K个基础代币获得K个投注码，每个投注码对应一个幸运数字（K为任意正整数）
- 同时输入中奖随机数和幸运数字投注列表
- **重要**：中奖揭秘阶段提供的投注码数量必须等于承诺阶段转移的代币数量K
- **重要限制**：单个幸运号码最多只能出现1000次
- **投注限制**：最大投注金额为1,000,000个基础代币
- 幸运数字取值范围：0-999
- 系统限制投注金额范围：1,000-1,000,000个基础代币（基础代币的整数倍）
- 允许选择重复的幸运数字（通过重复投注实现）
- **投注机制**：投注金额K必须等于投注码总数

### 奖金分配算法

#### 基本规则
- 当期销售额的 90% 作为奖金池
- 10% 作为软件服务费
- 一等奖：如果用户的投注码中有几个对应的幸运数字等于中奖号码，就中奖几次
- 中奖号码是由所有用户设置的随机数计算而来的去中心化随机数
- 没有二等奖与三等奖，只有一个奖项，就是一等奖

#### 分配策略

**1. 固定奖金分配（优先策略）**
- **触发条件：** 奖金池金额 ≥ 中奖者数量 × 800个基础代币
- **分配方式：** 每名中奖者固定获得800个基础代币
- **优势：** 保证中奖者获得稳定的奖金

**2. 平分奖金分配（兜底策略）**
- **触发条件：** 奖金池金额 < 中奖者数量 × 800个基础代币
- **分配方式：** 所有中奖者平分奖金池
- **计算方式：** 使用整数除法，确保公平分配
- **余数处理：** 余数部分保留在合约资金池中

## 🔒 安全特性

- **防重入保护**：防止重入攻击
- **访问控制**：只有授权用户可以执行特定操作
- **输入验证**：所有输入都经过严格验证
- **溢出保护**：使用 SafeMath 防止整数溢出
- **去中心化随机数安全**：基于所有参与者随机数生成中奖号码

### 🎲 去中心化随机数安全机制

**核心设计理念**：随机数质量依赖用户输入，这是去中心化随机数系统的重要安全特性。

**安全优势**：
- **抗单点操控**：任何单一方都无法控制随机数结果
- **集体安全**：只有控制全部参与者才能控制随机数结果
- **公平透明**：所有参与者共同参与随机数生成过程
- **可验证性**：随机数生成过程完全透明，结果可验证

**技术实现**：
- 使用 `dd_algorithms_lib` 去中心化算法库
- 收集所有参与者的随机种子
- 通过去中心化算法计算最终中奖号码
- 确保没有任何单一方能够预测或操控结果

**安全保证**：
- 攻击者需要控制**所有参与者**才能操控随机数
- 随着参与者数量增加，操控难度呈指数级增长
- 即使部分参与者提供弱随机数，系统仍能保持整体安全性

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
   - [系统架构设计文档](docs/系统架构设计文档.md)
   - [部署指南文档](docs/部署指南文档.md)
   - [项目结构说明文档](docs/项目结构说明文档.md)
   - [技术文档](docs/技术文档.md)
   - [使用示例](docs/使用示例.md)
2. 搜索 [Issues](https://github.com/your-org/dd-3d-lottery/issues)
3. 创建新的 Issue

---

**DD 3D 彩票智能合约** - 构建公平透明的去中心化彩票系统 🎲
