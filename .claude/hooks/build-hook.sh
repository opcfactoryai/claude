#!/bin/bash
# DesignGPT build hook
# 每次 src/*.rs 修改后自动编译，通过后复制 exe 到根目录
# 编译失败则强制 Claude 修复，不通过不让过

echo -e "\n📌 DesignGPT Build Hook — $(date '+%Y-%m-%d %H:%M:%S')" >&2
echo "正在编译 cargo build --release ..." >&2

build_output=$(cargo build --release 2>&1)
status=$?

if [ $status -eq 0 ]; then
    cp -f target/release/Launcher.exe ./Launcher.exe
    exe_size=$(du -h ./Launcher.exe | cut -f1)
    echo -e "\n✅ 编译成功, Launcher.exe ($exe_size) 已复制到根目录\n" >&2
    exit 0
else
    echo -e "\n❌ 编译失败！退出码: $status" >&2
    echo -e "\n=== 编译错误详情 ===" >&2
    echo "$build_output" >&2
    echo -e "========================\n" >&2

    cat << EOF

[HOOK_CRITICAL_FEEDBACK]
编译失败！

错误信息如下：
$build_output

指令：
1. 分析上面的编译错误原因
2. 用 Edit 工具修复所有导致编译失败的文件
3. 修复后我会再次运行 build 验证
4. **必须让 cargo build --release 100% 成功才能停止**

请开始修复代码。
[/HOOK_CRITICAL_FEEDBACK]
EOF

    exit 2
fi
