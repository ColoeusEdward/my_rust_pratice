#!/usr/bin/env bash
# trigger-ubuntu-build.sh
# 用 gh 触发 Ubuntu 打包(ubuntu-build.yml)，轮询运行状态，成功则弹窗提醒。
#
# 用法:
#   ./trigger-ubuntu-build.sh [分支] [profile]
#   分支    默认: 当前所在分支 (git rev-parse --abbrev-ref HEAD)
#   profile 默认: release   (可选 release / debug)
#
# 前置条件:
#   - gh 已登录且拥有 workflow 权限
#   - ubuntu-build.yml 已存在于“默认分支(main)”与“目标分支”上(否则无法被触发)
#
# 注意: 本脚本面向 Windows Git-Bash 环境，弹窗使用 powershell.exe。

set -euo pipefail

WORKFLOW="ubuntu-build.yml"
REF="${1:-$(git rev-parse --abbrev-ref HEAD)}"
PROFILE="${2:-release}"
POLL_INTERVAL=10   # 轮询间隔(秒)

echo "==> 工作流: $WORKFLOW"
echo "==> 目标分支: $REF"
echo "==> 构建类型: $PROFILE"

# 0) 基本检查
command -v gh >/dev/null 2>&1 || { echo "错误: 未找到 gh 命令"; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "错误: gh 未登录"; exit 1; }

# 1) 记录触发前该工作流的最新 run ID，用于识别本次新产生的 run
before_id="$(gh run list --workflow="$WORKFLOW" -L 1 --json databaseId \
              --jq '.[0].databaseId // 0' 2>/dev/null || echo 0)"
echo "==> 触发前最新 run ID: $before_id"

# 2) 触发工作流
echo "==> 正在触发..."
gh workflow run "$WORKFLOW" --ref "$REF" -f profile="$PROFILE"

# 3) 轮询直到出现比 before_id 更新的 run ID(即本次触发产生的 run)
echo "==> 等待新的运行出现..."
run_id=""
for _ in $(seq 1 30); do   # 最多等 ~30*3=90s
  candidate="$(gh run list --workflow="$WORKFLOW" -L 1 --json databaseId \
                --jq '.[0].databaseId // 0' 2>/dev/null || echo 0)"
  if [ "$candidate" != "0" ] && [ "$candidate" != "$before_id" ]; then
    run_id="$candidate"
    break
  fi
  sleep 3
done

if [ -z "$run_id" ]; then
  echo "错误: 超时未检测到新的运行。请确认目标分支与默认分支上都存在 $WORKFLOW。"
  exit 1
fi
echo "==> 本次运行 ID: $run_id"
echo "==> 链接: $(gh run view "$run_id" --json url --jq .url)"

# 4) 轮询运行状态直到完成
echo "==> 轮询状态(每 ${POLL_INTERVAL}s)..."
while true; do
  read -r status conclusion < <(gh run view "$run_id" \
        --json status,conclusion --jq '"\(.status) \(.conclusion // "")"')
  echo "    status=$status conclusion=$conclusion"
  if [ "$status" = "completed" ]; then
    break
  fi
  sleep "$POLL_INTERVAL"
done

# 5) 根据结论提醒
notify() {  # $1=标题 $2=正文 $3=图标码(64=信息,16=错误)
  powershell.exe -NoProfile -Command \
    "\$ws = New-Object -ComObject WScript.Shell; \
     \$ws.Popup('$2', 0, '$1', 0 + $3) | Out-Null" >/dev/null 2>&1 || \
    echo "(弹窗失败，忽略)"
}

if [ "$conclusion" = "success" ]; then
  echo "==> ✅ 打包成功"
  notify "Ubuntu 打包成功" "分支 $REF ($PROFILE) 打包成功! run #$run_id" 64
  exit 0
else
  echo "==> ❌ 打包未成功: $conclusion"
  notify "Ubuntu 打包失败" "分支 $REF ($PROFILE) 结论: $conclusion (run #$run_id)" 16
  exit 1
fi
