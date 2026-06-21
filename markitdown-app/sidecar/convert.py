#!/usr/bin/env python3
"""
MarkItDown Sidecar - 转换引擎桥接脚本
通过 stdin/stdout 与 Tauri Rust 后端通信，处理文件转换请求。

通信协议：
- 输入: 文件路径（命令行参数）
- 输出(stdout): Markdown 文本
- 进度(stderr): PROGRESS:<percent>:<message>
"""

import sys
import os
import json
import time
from pathlib import Path


def emit_progress(percent: int, message: str):
    """向 stderr 输出进度信息"""
    print(f"PROGRESS:{percent}:{message}", file=sys.stderr, flush=True)


def convert_file(file_path: str) -> str:
    """
    转换单个文件为 Markdown
    返回 Markdown 文本
    """
    emit_progress(5, "正在加载转换引擎...")

    try:
        from markitdown import MarkItDown
    except ImportError:
        emit_progress(0, "未找到 markitdown 库，请先安装: pip install markitdown")
        sys.exit(1)

    md = MarkItDown()
    file_path = Path(file_path)

    if not file_path.exists():
        emit_progress(0, f"文件不存在: {file_path}")
        sys.exit(1)

    file_name = file_path.name
    file_size = file_path.stat().st_size

    emit_progress(10, f"正在检测文件格式: {file_name}")
    emit_progress(20, f"文件大小: {file_size / 1024:.1f} KB")

    try:
        emit_progress(30, "正在转换...")
        result = md.convert(str(file_path))

        emit_progress(80, "正在格式化输出...")
        markdown_text = result.text_content

        if not markdown_text:
            markdown_text = f"*(文件 {file_name} 转换结果为空)*"

        emit_progress(95, "转换完成，正在输出...")

        # 输出到 stdout
        return markdown_text

    except Exception as e:
        emit_progress(0, f"转换失败: {str(e)}")
        sys.exit(1)


def convert_batch(file_paths: list[str]) -> str:
    """批量转换多个文件"""
    emit_progress(5, f"开始批量转换 {len(file_paths)} 个文件...")

    results = []
    for i, path in enumerate(file_paths):
        percent = int(5 + (i / len(file_paths)) * 90)
        emit_progress(percent, f"正在转换 ({i + 1}/{len(file_paths)}): {Path(path).name}")

        try:
            result = convert_file(path)
            results.append(f"## {Path(path).name}\n\n{result}\n\n---\n")
        except SystemExit:
            results.append(f"## {Path(path).name}\n\n*(转换失败)*\n\n---\n")

    emit_progress(95, "合并输出中...")
    return "\n".join(results)


def main():
    if len(sys.argv) < 2:
        print("用法: python convert.py <文件路径> [文件路径2 ...]", file=sys.stderr)
        sys.exit(1)

    file_paths = sys.argv[1:]

    if len(file_paths) == 1:
        result = convert_file(file_paths[0])
    else:
        result = convert_batch(file_paths)

    # 输出最终结果到 stdout
    print(result, flush=True)
    emit_progress(100, "完成")


if __name__ == "__main__":
    main()