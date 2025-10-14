#!/bin/bash

# DD 3D 彩票智能合约部署脚本
# 基于 CosmWasm 的部署自动化脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置变量
CHAIN_ID="archway-1"
NODE_URL="https://rpc.archway.io:443"
GAS_PRICES="0.025uarch"
GAS_ADJUSTMENT="1.3"
ADMIN_ADDRESS=""
SERVICE_FEE_RATE="0.1"
MIN_BET_AMOUNT="1000"
MAX_BET_AMOUNT="1000000"

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
    
    if ! command -v archwayd &> /dev/null; then
        log_error "archwayd 未安装，请先安装 Archway CLI"
        exit 1
    fi
    
    if ! command -v cosmwasm-opt &> /dev/null; then
        log_warning "cosmwasm-opt 未安装，将跳过 WASM 优化"
    fi
    
    log_success "依赖检查完成"
}

# 构建合约
build_contract() {
    log_info "构建合约..."
    
    # 清理之前的构建
    cargo clean
    
    # 构建合约
    cargo build --release --target wasm32-unknown-unknown
    
    if [ $? -eq 0 ]; then
        log_success "合约构建成功"
    else
        log_error "合约构建失败"
        exit 1
    fi
    
    # 优化 WASM（如果可用）
    if command -v cosmwasm-opt &> /dev/null; then
        log_info "优化 WASM..."
        cosmwasm-opt target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm \
            -o target/wasm32-unknown-unknown/release/dd_3d_lottery_optimized.wasm
        
        if [ $? -eq 0 ]; then
            log_success "WASM 优化完成"
            WASM_FILE="target/wasm32-unknown-unknown/release/dd_3d_lottery_optimized.wasm"
        else
            log_warning "WASM 优化失败，使用原始文件"
            WASM_FILE="target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm"
        fi
    else
        WASM_FILE="target/wasm32-unknown-unknown/release/dd_3d_lottery.wasm"
    fi
}

# 上传合约代码
upload_contract() {
    log_info "上传合约代码..."
    
    # 检查 WASM 文件是否存在
    if [ ! -f "$WASM_FILE" ]; then
        log_error "WASM 文件不存在: $WASM_FILE"
        exit 1
    fi
    
    # 上传合约
    CODE_ID=$(archwayd tx wasm store "$WASM_FILE" \
        --from "$ADMIN_ADDRESS" \
        --gas auto \
        --gas-adjustment "$GAS_ADJUSTMENT" \
        --gas-prices "$GAS_PRICES" \
        --chain-id "$CHAIN_ID" \
        --node "$NODE_URL" \
        --yes \
        --output json | jq -r '.logs[0].events[0].attributes[0].value')
    
    if [ "$CODE_ID" = "null" ] || [ -z "$CODE_ID" ]; then
        log_error "合约上传失败"
        exit 1
    fi
    
    log_success "合约上传成功，Code ID: $CODE_ID"
    echo "$CODE_ID" > .code_id
}

# 实例化合约
instantiate_contract() {
    log_info "实例化合约..."
    
    if [ ! -f ".code_id" ]; then
        log_error "Code ID 文件不存在，请先上传合约"
        exit 1
    fi
    
    CODE_ID=$(cat .code_id)
    
    # 构建实例化消息
    INSTANTIATE_MSG=$(cat <<EOF
{
    "admin": "$ADMIN_ADDRESS",
    "service_fee_rate": "$SERVICE_FEE_RATE",
    "min_bet_amount": "$MIN_BET_AMOUNT",
    "max_bet_amount": "$MAX_BET_AMOUNT"
}
EOF
)
    
    # 实例化合约
    CONTRACT_ADDRESS=$(archwayd tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" \
        --from "$ADMIN_ADDRESS" \
        --admin "$ADMIN_ADDRESS" \
        --label "DD 3D Lottery" \
        --gas auto \
        --gas-adjustment "$GAS_ADJUSTMENT" \
        --gas-prices "$GAS_PRICES" \
        --chain-id "$CHAIN_ID" \
        --node "$NODE_URL" \
        --yes \
        --output json | jq -r '.logs[0].events[0].attributes[0].value')
    
    if [ "$CONTRACT_ADDRESS" = "null" ] || [ -z "$CONTRACT_ADDRESS" ]; then
        log_error "合约实例化失败"
        exit 1
    fi
    
    log_success "合约实例化成功，地址: $CONTRACT_ADDRESS"
    echo "$CONTRACT_ADDRESS" > .contract_address
}

# 验证部署
verify_deployment() {
    log_info "验证部署..."
    
    if [ ! -f ".contract_address" ]; then
        log_error "合约地址文件不存在"
        exit 1
    fi
    
    CONTRACT_ADDRESS=$(cat .contract_address)
    
    # 查询合约信息
    CONTRACT_INFO=$(archwayd query wasm contract "$CONTRACT_ADDRESS" \
        --chain-id "$CHAIN_ID" \
        --node "$NODE_URL" \
        --output json)
    
    if [ $? -eq 0 ]; then
        log_success "合约验证成功"
        echo "合约地址: $CONTRACT_ADDRESS"
        echo "合约信息: $CONTRACT_INFO"
    else
        log_error "合约验证失败"
        exit 1
    fi
}

# 测试合约功能
test_contract() {
    log_info "测试合约功能..."
    
    if [ ! -f ".contract_address" ]; then
        log_error "合约地址文件不存在"
        exit 1
    fi
    
    CONTRACT_ADDRESS=$(cat .contract_address)
    
    # 查询当前阶段
    PHASE=$(archwayd query wasm contract-state smart "$CONTRACT_ADDRESS" \
        '{"get_current_phase": {}}' \
        --chain-id "$CHAIN_ID" \
        --node "$NODE_URL" \
        --output json | jq -r '.data.phase')
    
    log_success "当前阶段: $PHASE"
    
    # 查询系统配置
    CONFIG=$(archwayd query wasm contract-state smart "$CONTRACT_ADDRESS" \
        '{"get_config": {}}' \
        --chain-id "$CHAIN_ID" \
        --node "$NODE_URL" \
        --output json)
    
    log_success "系统配置: $CONFIG"
}

# 清理临时文件
cleanup() {
    log_info "清理临时文件..."
    rm -f .code_id .contract_address
    log_success "清理完成"
}

# 显示帮助信息
show_help() {
    echo "DD 3D 彩票智能合约部署脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help              显示帮助信息"
    echo "  -a, --admin ADDRESS     管理员地址 (必需)"
    echo "  -c, --chain-id ID       链ID (默认: archway-1)"
    echo "  -n, --node URL          节点URL (默认: https://rpc.archway.io:443)"
    echo "  -g, --gas-prices PRICE   Gas价格 (默认: 0.025uarch)"
    echo "  --min-bet AMOUNT        最小投注金额 (默认: 1000)"
    echo "  --max-bet AMOUNT        最大投注金额 (默认: 1000000)"
    echo "  --service-fee RATE      服务费率 (默认: 0.1)"
    echo "  --cleanup               部署后清理临时文件"
    echo ""
    echo "示例:"
    echo "  $0 --admin archway1..."
    echo "  $0 -a archway1... -c archway-testnet-1"
}

# 主函数
main() {
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -a|--admin)
                ADMIN_ADDRESS="$2"
                shift 2
                ;;
            -c|--chain-id)
                CHAIN_ID="$2"
                shift 2
                ;;
            -n|--node)
                NODE_URL="$2"
                shift 2
                ;;
            -g|--gas-prices)
                GAS_PRICES="$2"
                shift 2
                ;;
            --min-bet)
                MIN_BET_AMOUNT="$2"
                shift 2
                ;;
            --max-bet)
                MAX_BET_AMOUNT="$2"
                shift 2
                ;;
            --service-fee)
                SERVICE_FEE_RATE="$2"
                shift 2
                ;;
            --cleanup)
                CLEANUP=true
                shift
                ;;
            *)
                log_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 检查必需参数
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_error "管理员地址是必需的"
        show_help
        exit 1
    fi
    
    # 显示配置信息
    log_info "部署配置:"
    echo "  管理员地址: $ADMIN_ADDRESS"
    echo "  链ID: $CHAIN_ID"
    echo "  节点URL: $NODE_URL"
    echo "  Gas价格: $GAS_PRICES"
    echo "  最小投注金额: $MIN_BET_AMOUNT"
    echo "  最大投注金额: $MAX_BET_AMOUNT"
    echo "  服务费率: $SERVICE_FEE_RATE"
    echo ""
    
    # 执行部署流程
    check_dependencies
    build_contract
    upload_contract
    instantiate_contract
    verify_deployment
    test_contract
    
    log_success "部署完成！"
    
    # 清理临时文件
    if [ "$CLEANUP" = true ]; then
        cleanup
    fi
}

# 运行主函数
main "$@"
