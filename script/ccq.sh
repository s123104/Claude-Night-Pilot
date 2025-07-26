#!/usr/bin/env zsh
set -euo pipefail

# ===== 色彩定義 =====
RESET=$(tput sgr0)        BOLD=$(tput bold)
GREEN=$(tput setaf 2)     YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)      CYAN=$(tput setaf 6)  RED=$(tput setaf 1)

# ===== 自己的絕對路徑 =====
SCRIPT="$(command -v ccq)"

# ===== 路徑設定 =====
BASE="$HOME/.ccq"; mkdir -p "$BASE"
SCHED="$BASE/schedules.json"; [[ -f $SCHED ]] || echo '[]' > "$SCHED"
COOL="$BASE/cooldown_log.json"; [[ -f $COOL ]] || echo '[]' > "$COOL"
AGENTS="$HOME/Library/LaunchAgents"
LABEL_PREFIX="com.$USER.ccq"

# ===== 公用函式 =====
say(){ print -P "%{$1%}$2%{$RESET%}"; }

json_log(){ # $1=file $2=status $3=note
  local file=$1 status=$2 note=$3
  if command -v sponge &>/dev/null; then
    jq --arg t "$(date -Iseconds)" --arg s "$status" --arg n "$note" \
       '. += [{time:$t,status:$s,note:$n}]' "$file" | sponge "$file"
  else
    jq --arg t "$(date -Iseconds)" --arg s "$status" --arg n "$note" \
       '. += [{time:$t,status:$s,note:$n}]' "$file" > "$file.tmp" \
    && mv "$file.tmp" "$file"
  fi
}

sort_schedules(){
  if command -v sponge &>/dev/null; then
    jq -S 'sort_by(.time)' "$SCHED" | sponge "$SCHED"
  else
    jq -S 'sort_by(.time)' "$SCHED" > "$SCHED.tmp" && mv "$SCHED.tmp" "$SCHED"
  fi
}

# ===== launchd 操作 =====
plist_path(){ echo "$AGENTS/$LABEL_PREFIX.$1.plist"; }
reload_plist(){
  launchctl unload -w "$1" &>/dev/null || true
  launchctl load   -w "$1"
}

generate_plist(){ # $1=HHMM $2=prompt $3=cwd
  local hhmm=$1 pr=$2 cwd=$3
  local lab="$LABEL_PREFIX.$hhmm"
  local pl=$(plist_path $hhmm)
  mkdir -p "$(dirname "$pl")"

  cat > "$pl" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$lab</string>

  <key>ProgramArguments</key>
  <array>
    <string>$SCRIPT</string>
    <string>run</string>
    <string><![CDATA[$pr]]></string>
    <string>$cwd</string>
  </array>

  <key>WorkingDirectory</key>
  <string>$cwd</string>

  <key>StartCalendarInterval</key>
  <dict>
    <key>Hour</key><integer>$((10#${hhmm:0:2}))</integer>
    <key>Minute</key><integer>$((10#${hhmm:2:2}))</integer>
  </dict>

  <key>StandardOutPath</key>
  <string>/tmp/$lab.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/$lab.log</string>
</dict>
</plist>
PLIST

  reload_plist "$pl"
}

# ===== 主流程 =====
case ${1:-help} in

  add)
    say $CYAN "➤ 新增排程（時間格式 HH:MM，Enter 結束）："
    while true; do
      read -r "t?時間 HH:MM: "
      [[ -z "$t" ]] && break

      echo -n "Prompt: "
      IFS= read -r p
      [[ -z "$p" ]] && break

      # 更新 JSON
      if command -v sponge &>/dev/null; then
        jq --arg t "${t//:/}" --arg p "$p" --arg cwd "$(pwd)" \
           '. += [{time:$t,prompt:$p,cwd:$cwd}]' "$SCHED" | sponge "$SCHED"
      else
        jq --arg t "${t//:/}" --arg p "$p" --arg cwd "$(pwd)" \
           '. += [{time:$t,prompt:$p,cwd:$cwd}]' "$SCHED" > "$SCHED.tmp" \
        && mv "$SCHED.tmp" "$SCHED"
      fi

      # 產生/更新 plist
      generate_plist "${t//:/}" "$p" "$(pwd)"
    done
    sort_schedules
    ;;

  edit)
    say $BLUE "已排程列表：" && $0 list
    read -r "i?要修改哪一筆序號: " || exit
    idx=$((i-1))
    oldtime=$(jq -r ".[$idx].time" "$SCHED")

    read -r "nt?新時間 HH:MM (留空保留 $oldtime): "
    nt=${nt:-$oldtime}

    echo -n "新 Prompt (留空保留舊 prompt): "
    IFS= read -r np
    [[ -z "$np" ]] && np=$(jq -r ".[$idx].prompt" "$SCHED")

    # 更新 JSON
    if command -v sponge &>/dev/null; then
      jq ".[${idx}] |= (.time=\"${nt//:/}\"|.prompt=\"${np}\")" "$SCHED" | sponge "$SCHED"
    else
      jq ".[${idx}] |= (.time=\"${nt//:/}\"|.prompt=\"${np}\")" "$SCHED" > "$SCHED.tmp" \
      && mv "$SCHED.tmp" "$SCHED"
    fi

    # 重寫 plist
    generate_plist "${nt//:/}" "$np" "$(jq -r .[$idx].cwd "$SCHED")"
    sort_schedules
    say $GREEN "✓ 已更新"
    ;;

  rm)
    $0 list
    read -r "i?刪除序號: " || exit
    idx=$((i-1))
    del_time=$(jq -r ".[$idx].time" "$SCHED")
    del_pl=$(plist_path $del_time)

    launchctl unload -w "$del_pl" &>/dev/null || true
    rm -f "$del_pl"

    if command -v sponge &>/dev/null; then
      jq "del(.[$idx])" "$SCHED" | sponge "$SCHED"
    else
      jq "del(.[$idx])" "$SCHED" > "$SCHED.tmp" && mv "$SCHED.tmp" "$SCHED"
    fi

    say $YELLOW "✗ 已移除 $del_time"
    ;;

  list)
    jq -c '.[]' "$SCHED" | while read -r row; do
      tm=$(jq -r .time <<< "$row")
      pr=$(jq -r .prompt <<< "$row")
      say $GREEN "${tm:0:2}:${tm:2:2}  ▶  ${pr:0:40}"
    done
    ;;

  run)  # 由 launchd 執行
    prompt="$2"; cwd="$3"
    cd "$cwd"
    out=$(echo "$prompt" | claude --dangerously-skip-permissions 2>&1) || true

    if echo "$out" | grep -qi "usage limit reached"; then
      json_log "$COOL" cooldown "$out"

      hhmm="${prompt_time:-unknown}"
      # 由 plist label 取得當前時間 tag
      # 可以從 $3 或環境衝入，或改為從 schedules.json 查找

      new=$(date -j -f "%H%M" "$hhmm" -v+1H +"%H%M")

      if command -v sponge &>/dev/null; then
        jq "(.[]|select(.time==\"$hhmm\")).time=\"$new\"" "$SCHED" | sponge "$SCHED"
      else
        jq "(.[]|select(.time==\"$hhmm\")).time=\"$new\"" "$SCHED" > "$SCHED.tmp" \
        && mv "$SCHED.tmp" "$SCHED"
      fi

      sort_schedules
      generate_plist "$new" "$prompt" "$cwd"
    else
      json_log "$COOL" success "run ok"
    fi
    ;;

  probe)
    out=$(echo ping | claude --dangerously-skip-permissions 2>&1) || true
    if echo "$out" | grep -qi "usage limit reached"; then
      json_log "$COOL" cooldown "$out"
    else
      json_log "$COOL" ok "-"
    fi
    ;;

  help|*)
    cat <<-HEREDOC
${BOLD}ccq — Claude Quick Queue${RESET}

用法：
  ccq add      互動新增排程
  ccq edit     修改既有排程
  ccq rm       刪除排程
  ccq list     顯示已排程清單
  ccq probe    立即測冷卻

執行前請確認系統時區為 Asia/Taipei。
HEREDOC
    ;;
esac
