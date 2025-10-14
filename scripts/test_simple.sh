#!/bin/bash

# 简化的测试运行脚本
set -e

echo "运行基础测试..."

# 运行基础集成测试
cargo test --test integration test_instantiate --quiet

echo "基础测试通过！"

# 运行一个简单的功能测试
cargo test --test integration test_place_bet_commitment_phase --quiet

echo "功能测试通过！"

echo "所有测试完成！"
