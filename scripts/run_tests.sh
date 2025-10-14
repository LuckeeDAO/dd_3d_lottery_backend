#!/bin/bash

# DD 3D 彩票智能合约测试运行脚本
# 运行所有测试并生成测试报告

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 函数定义
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "cargo 未安装，请先安装 Rust"
        exit 1
    fi
    
    log_success "依赖检查完成"
}

# 运行单元测试
run_unit_tests() {
    log_info "运行单元测试..."
    
    cargo test --lib --verbose
    
    if [ $? -eq 0 ]; then
        log_success "单元测试通过"
    else
        log_error "单元测试失败"
        exit 1
    fi
}

# 运行集成测试
run_integration_tests() {
    log_info "运行集成测试..."
    
    # 基础集成测试
    cargo test --test integration --verbose
    
    if [ $? -eq 0 ]; then
        log_success "基础集成测试通过"
    else
        log_error "基础集成测试失败"
        exit 1
    fi
}

# 运行奖金分配测试
run_reward_tests() {
    log_info "运行奖金分配测试..."
    
    cargo test --test reward_distribution_tests --verbose
    
    if [ $? -eq 0 ]; then
        log_success "奖金分配测试通过"
    else
        log_error "奖金分配测试失败"
        exit 1
    fi
}

# 运行端到端测试
run_e2e_tests() {
    log_info "运行端到端测试..."
    
    cargo test --test end_to_end_tests --verbose
    
    if [ $? -eq 0 ]; then
        log_success "端到端测试通过"
    else
        log_error "端到端测试失败"
        exit 1
    fi
}

# 运行性能测试
run_performance_tests() {
    log_info "运行性能测试..."
    
    cargo test --test performance_tests --verbose
    
    if [ $? -eq 0 ]; then
        log_success "性能测试通过"
    else
        log_warning "性能测试失败（可能由于环境限制）"
    fi
}

# 运行安全测试
run_security_tests() {
    log_info "运行安全测试..."
    
    cargo test --test security_tests --verbose
    
    if [ $? -eq 0 ]; then
        log_success "安全测试通过"
    else
        log_error "安全测试失败"
        exit 1
    fi
}

# 运行所有测试
run_all_tests() {
    log_info "运行所有测试..."
    
    cargo test --verbose
    
    if [ $? -eq 0 ]; then
        log_success "所有测试通过"
    else
        log_error "部分测试失败"
        exit 1
    fi
}

# 生成测试报告
generate_test_report() {
    log_info "生成测试报告..."
    
    # 创建测试报告目录
    mkdir -p test_reports
    
    # 运行测试并生成报告
    cargo test --verbose 2>&1 | tee test_reports/test_output.log
    
    # 统计测试结果
    local total_tests=$(grep -c "test " test_reports/test_output.log || echo "0")
    local passed_tests=$(grep -c "test result: ok" test_reports/test_output.log || echo "0")
    local failed_tests=$((total_tests - passed_tests))
    
    # 生成HTML报告
    cat > test_reports/test_report.html << EOF
<!DOCTYPE html>
<html>
<head>
    <title>DD 3D 彩票智能合约测试报告</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .summary { background-color: #e8f5e8; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .error { background-color: #ffe8e8; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .success { color: green; }
        .failure { color: red; }
    </style>
</head>
<body>
    <div class="header">
        <h1>DD 3D 彩票智能合约测试报告</h1>
        <p>生成时间: $(date)</p>
    </div>
    
    <div class="summary">
        <h2>测试摘要</h2>
        <p>总测试数: $total_tests</p>
        <p class="success">通过测试: $passed_tests</p>
        <p class="failure">失败测试: $failed_tests</p>
    </div>
    
    <div class="error">
        <h2>测试详情</h2>
        <pre>$(cat test_reports/test_output.log)</pre>
    </div>
</body>
</html>
EOF

    log_success "测试报告已生成: test_reports/test_report.html"
}

# 显示帮助信息
show_help() {
    echo "DD 3D 彩票智能合约测试运行脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help              显示帮助信息"
    echo "  -u, --unit              只运行单元测试"
    echo "  -i, --integration       只运行集成测试"
    echo "  -r, --reward            只运行奖金分配测试"
    echo "  -e, --end-to-end       只运行端到端测试"
    echo "  -p, --performance     只运行性能测试"
    echo "  -s, --security         只运行安全测试"
    echo "  -a, --all              运行所有测试（默认）"
    echo "  --report               生成测试报告"
    echo ""
    echo "示例:"
    echo "  $0 --all --report     运行所有测试并生成报告"
    echo "  $0 --unit             只运行单元测试"
    echo "  $0 --performance     只运行性能测试"
}

# 主函数
main() {
    local run_unit=false
    local run_integration=false
    local run_reward=false
    local run_e2e=false
    local run_performance=false
    local run_security=false
    local run_all=true
    local generate_report=false
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -u|--unit)
                run_unit=true
                run_all=false
                shift
                ;;
            -i|--integration)
                run_integration=true
                run_all=false
                shift
                ;;
            -r|--reward)
                run_reward=true
                run_all=false
                shift
                ;;
            -e|--end-to-end)
                run_e2e=true
                run_all=false
                shift
                ;;
            -p|--performance)
                run_performance=true
                run_all=false
                shift
                ;;
            -s|--security)
                run_security=true
                run_all=false
                shift
                ;;
            -a|--all)
                run_all=true
                shift
                ;;
            --report)
                generate_report=true
                shift
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 检查依赖
    check_dependencies
    
    # 运行测试
    if [ "$run_all" = true ]; then
        run_all_tests
    else
        if [ "$run_unit" = true ]; then
            run_unit_tests
        fi
        
        if [ "$run_integration" = true ]; then
            run_integration_tests
        fi
        
        if [ "$run_reward" = true ]; then
            run_reward_tests
        fi
        
        if [ "$run_e2e" = true ]; then
            run_e2e_tests
        fi
        
        if [ "$run_performance" = true ]; then
            run_performance_tests
        fi
        
        if [ "$run_security" = true ]; then
            run_security_tests
        fi
    fi
    
    # 生成测试报告
    if [ "$generate_report" = true ]; then
        generate_test_report
    fi
    
    log_success "测试完成！"
}

# 运行主函数
main "$@"
