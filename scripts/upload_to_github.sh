#!/bin/bash

# DD 3D Lottery Backend 项目自动上传到 GitHub 脚本
# 使用方法: ./scripts/upload_to_github.sh

set -e  # 遇到错误时退出

echo "🚀 开始上传 DD 3D Lottery Backend 项目到 GitHub..."

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 请在项目根目录运行此脚本"
    exit 1
fi

# 检查 Git 状态
echo "📋 检查 Git 状态..."
git status

# 添加所有修改的文件
echo "📝 添加所有修改的文件..."
git add .

# 提交更改
echo "💾 提交更改..."
git commit -m "feat: 完成 DD 3D Lottery Backend 智能合约开发

- 实现完整的3D彩票智能合约系统
- 支持三阶段投注机制 (承诺/揭秘/结算)
- 实现多号码选择和倍数投注功能
- 添加去中心化随机数生成机制
- 实现奖金分配和提取系统
- 支持安全机制和访问控制
- 升级到 CosmWasm 2.2.2 版本

核心功能:
- 三阶段投注系统 (承诺/揭秘/结算)
- 多号码选择和倍数投注
- 去中心化随机数生成
- 奖金池管理和分配
- 用户投注记录管理
- 中奖结果计算和验证

技术特性:
- CosmWasm 2.2.2 兼容
- 完整的错误处理机制
- 防重入攻击保护
- 访问控制和权限管理
- 详细的测试覆盖
- 生产就绪的部署脚本

安全特性:
- 防重入保护机制
- 输入验证和边界检查
- 访问控制和权限管理
- 紧急暂停功能
- 去中心化随机数安全
- 完整的审计日志

部署特性:
- 自动化部署脚本
- 环境配置管理
- 合约验证和测试
- 生产环境就绪
- 完整的文档和示例"

# 确认远程仓库设置
echo "🔗 确认远程仓库设置..."
git remote -v

# 推送代码到 GitHub
echo "⬆️  推送代码到 GitHub..."
git push -u origin main

echo "✅ 项目已成功上传到 GitHub!"

# 显示项目信息
echo ""
echo "📊 项目统计:"
echo "   - 总文件数: $(find . -type f | wc -l)"
echo "   - Rust代码行数: $(find . -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')"
echo "   - 测试文件: $(find . -name "*test*.rs" | wc -l)"
echo "   - 文档文件: $(find . -name "*.md" | wc -l)"
echo "   - 脚本文件: $(find . -name "*.sh" | wc -l)"
echo "   - 配置文件: $(find . -name "*.toml" -o -name "*.json" | wc -l)"

echo ""
echo "🎉 上传完成! 您现在可以访问 GitHub 仓库查看您的项目"
echo "📋 本次提交包含:"
echo "   - 完整的3D彩票智能合约实现"
echo "   - 三阶段投注机制"
echo "   - 多号码选择和倍数投注功能"
echo "   - 去中心化随机数生成"
echo "   - 安全机制和访问控制"
echo "   - 生产就绪的部署脚本"
echo "   - 完整的测试覆盖"
echo "   - 详细的文档和示例"
