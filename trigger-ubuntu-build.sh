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

# 通用重试：重试执行命令并捕获其 stdout。
# 用法: out="$(retry <最大次数> <间隔秒> <命令...>)"
# 命令 exit 0 视为成功(即使输出为空)，打印 stdout 并返回 0；
# 全部重试失败则返回最后一次的退出码。进度提示走 stderr，不污染捕获的 stdout。
retry() {
  local tries="$1" delay="$2"; shift 2
  local n=0 out rc
  while :; do
    n=$((n + 1))
    if out="$("$@" 2>/dev/null)"; then
      printf '%s' "$out"
      return 0
    fi
    rc=$?
    if [ "$n" -ge "$tries" ]; then
      return "$rc"
    fi
    echo "    (第 $n/$tries 次获取失败，${delay}s 后重试: $*)" >&2
    sleep "$delay"
  done
}

# 1) 记录触发前该工作流的最新 run ID，用于识别本次新产生的 run
before_id="$(retry 5 3 gh run list --workflow="$WORKFLOW" -L 1 --json databaseId \
              --jq '.[0].databaseId // 0' || echo 0)"
echo "==> 触发前最新 run ID: $before_id"

# 2) 触发工作流
#   瞬时网络错误(如 GraphQL EOF)会重试；但由于触发请求可能"服务端已成功、
#   响应途中断开"，每次失败后先查一次是否已出现比 before_id 更新的 run，
#   有则视为触发成功、不再重试，避免重复触发产生多个 run。
echo "==> 正在触发..."
triggered=0
for i in 1 2 3 4 5; do
  if gh workflow run "$WORKFLOW" --ref "$REF" -f profile="$PROFILE" >/dev/null 2>&1; then
    triggered=1
    break
  fi
  # 触发命令返回失败：确认是否其实已经产生了新 run
  check_id="$(retry 3 3 gh run list --workflow="$WORKFLOW" -L 1 --json databaseId \
                --jq '.[0].databaseId // 0' || echo 0)"
  if [ "$check_id" != "0" ] && [ "$check_id" != "$before_id" ]; then
    echo "    (触发命令报错，但已检测到新 run $check_id，视为触发成功)"
    triggered=1
    break
  fi
  if [ "$i" -lt 5 ]; then
    echo "    (第 $i/5 次触发失败且未产生新 run，3s 后重试)" >&2
    sleep 3
  fi
done
if [ "$triggered" -ne 1 ]; then
  echo "错误: 触发工作流失败(多次重试后仍失败)。请检查网络或 gh 登录状态。"
  exit 1
fi

# 3) 轮询直到出现比 before_id 更新的 run ID(即本次触发产生的 run)
echo "==> 等待新的运行出现..."
run_id=""
for _ in $(seq 1 30); do   # 最多等 ~30*3=90s
  candidate="$(retry 3 3 gh run list --workflow="$WORKFLOW" -L 1 --json databaseId \
                --jq '.[0].databaseId // 0' || echo 0)"
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
echo "==> 链接: $(retry 5 3 gh run view "$run_id" --json url --jq .url || echo '(获取链接失败)')"

# 4) 轮询运行状态直到完成
echo "==> 轮询状态(每 ${POLL_INTERVAL}s)..."
while true; do
  # 单次状态查询失败会自动重试；仍失败则本轮跳过，下个周期再试，不中断整个轮询
  line="$(retry 5 3 gh run view "$run_id" \
        --json status,conclusion --jq '"\(.status) \(.conclusion // "")"' || echo '')"
  if [ -z "$line" ]; then
    echo "    (状态查询暂时失败，${POLL_INTERVAL}s 后重试)"
    sleep "$POLL_INTERVAL"
    continue
  fi
  read -r status conclusion <<< "$line"
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

  # 6) 查询本次 run 产生的 artifact，输出可在浏览器打开的下载链接
  run_url="$(retry 5 3 gh run view "$run_id" --json url --jq .url || echo '')"
  repo="$(retry 5 3 gh repo view --json nameWithOwner --jq .nameWithOwner || echo '')"

  echo "==> Artifact 列表:"
  # artifacts API 返回 id/name/size；浏览器下载链接需自行拼接为
  # https://github.com/<repo>/actions/runs/<run_id>/artifacts/<artifact_id>
  artifact_lines="$(retry 5 3 gh api "repos/$repo/actions/runs/$run_id/artifacts" \
      --jq '.artifacts[] | "\(.id)\t\(.name)\t\(.size_in_bytes)"' || echo '')"

  first_artifact_url=""
  if [ -n "$artifact_lines" ]; then
    while IFS=$'\t' read -r aid aname asize; do
      [ -z "$aid" ] && continue
      aurl="https://github.com/$repo/actions/runs/$run_id/artifacts/$aid"
      printf '    - %s (%s bytes)\n      %s\n' "$aname" "$asize" "$aurl"
      [ -z "$first_artifact_url" ] && first_artifact_url="$aurl"
    done <<< "$artifact_lines"
    echo "==> 亦可用命令下载: gh run download $run_id"
  else
    echo "    (未查询到 artifact，可能工作流未上传或权限不足)"
  fi

  # 弹窗：优先展示 artifact 链接，无则退回 run 链接
  if [ -n "$first_artifact_url" ]; then
    notify "Ubuntu 打包成功" "分支 $REF ($PROFILE) 打包成功! Artifact: $first_artifact_url" 64
  else
    notify "Ubuntu 打包成功" "分支 $REF ($PROFILE) 打包成功! run #$run_id: $run_url" 64
  fi
  exit 0
else
  echo "==> ❌ 打包未成功: $conclusion"
  notify "Ubuntu 打包失败" "分支 $REF ($PROFILE) 结论: $conclusion (run #$run_id)" 16
  exit 1
fi
