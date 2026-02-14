from __future__ import annotations

import sys

from PySide6.QtCore import QProcess
from PySide6.QtGui import QTextCursor
from PySide6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPlainTextEdit,
    QPushButton,
    QVBoxLayout,
    QWidget,
    QProgressBar,
)

from .page_frame import PageFrame


class Step5InstallPage(PageFrame):
    def __init__(self) -> None:
        super().__init__("Step 5 — Installation Progress")

        self._status = QLabel("Ready.")
        self._status.setStyleSheet("color: #bdbdbd;")

        self._progress = QProgressBar()
        self._progress.setRange(0, 0)  # indeterminate
        self._progress.setVisible(False)

        self._btn_start = QPushButton("Start Install")
        self._btn_cancel = QPushButton("Cancel")
        self._btn_cancel.setEnabled(False)

        self._output = QPlainTextEdit()
        self._output.setReadOnly(True)
        self._output.setLineWrapMode(QPlainTextEdit.NoWrap)
        self._output.setPlaceholderText("Installer output will appear here…")

        self._stdin = QLineEdit()
        self._stdin.setPlaceholderText(
            "Type response (or leave empty) and press Enter…"
        )
        self._btn_send = QPushButton("Send")
        self._btn_send.setFixedWidth(90)
        self._stdin.setEnabled(False)
        self._btn_send.setEnabled(False)

        self._proc = QProcess(self)
        self._proc.setProcessChannelMode(QProcess.MergedChannels)

        self._last_argv: list[str] | None = None
        self._last_cwd: str | None = None

        self.set_body(self._build())
        self._wire()

    # Public hook (wire this from Step1/Step4 later)
    def start_placeholder(self) -> None:
        self.start_install(self._placeholder_command())

    def start_install(self, argv: list[str], cwd: str | None = None) -> None:
        if self._proc.state() != QProcess.NotRunning:
            return
        if not argv:
            self._append("ERROR: empty command.\n")
            return

        self._last_argv = list(argv)
        self._last_cwd = cwd

        if cwd:
            self._proc.setWorkingDirectory(cwd)
        else:
            self._proc.setWorkingDirectory("")

        self._output.clear()
        self._set_running(True)
        self._status.setText("Running…")
        if cwd:
            self._append(f"[CWD {cwd}]\n")
        self._append(f"$ {self._fmt_cmd(argv)}\n")

        self._proc.start(argv[0], argv[1:])

    def _build(self) -> QWidget:
        host = QWidget()
        root = QVBoxLayout(host)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(10)

        top = QFrame()
        top.setObjectName("Panel")
        top_lay = QHBoxLayout(top)
        top_lay.setContentsMargins(12, 12, 12, 12)
        top_lay.setSpacing(12)

        top_lay.addWidget(self._btn_start)
        top_lay.addWidget(self._btn_cancel)
        top_lay.addStretch(1)
        top_lay.addWidget(self._progress)

        root.addWidget(top)

        out_panel = QFrame()
        out_panel.setObjectName("Panel")
        out_lay = QVBoxLayout(out_panel)
        out_lay.setContentsMargins(12, 12, 12, 12)
        out_lay.setSpacing(10)
        out_lay.addWidget(self._output, 1)

        stdin_row = QHBoxLayout()
        stdin_row.setSpacing(10)
        stdin_row.addWidget(self._stdin, 1)
        stdin_row.addWidget(self._btn_send)
        out_lay.addLayout(stdin_row)
        root.addWidget(out_panel, 1)

        root.addWidget(self._status)
        return host

    def _wire(self) -> None:
        self._btn_start.clicked.connect(self._on_start_clicked)
        self._btn_cancel.clicked.connect(self._on_cancel_clicked)

        self._btn_send.clicked.connect(self._on_send_clicked)
        self._stdin.returnPressed.connect(self._on_send_clicked)

        self._proc.readyReadStandardOutput.connect(self._on_ready_read)
        self._proc.errorOccurred.connect(self._on_error)
        self._proc.finished.connect(self._on_finished)

    def _on_start_clicked(self) -> None:
        argv = self._last_argv or self._placeholder_command()
        self.start_install(argv, cwd=self._last_cwd)

    def _on_cancel_clicked(self) -> None:
        if self._proc.state() == QProcess.NotRunning:
            return
        self._append("\n[CANCEL]\n")
        self._proc.kill()

    def _on_send_clicked(self) -> None:
        if self._proc.state() == QProcess.NotRunning:
            return
        text = self._stdin.text()
        self._stdin.clear()

        # Allow "press Enter" prompts (empty line).
        if text:
            self._append(f"> {text}\n")
        else:
            self._append("> [ENTER]\n")

        data = (text + "\r\n").encode("utf-8", errors="replace")
        self._proc.write(data)

    def _on_ready_read(self) -> None:
        data = bytes(self._proc.readAllStandardOutput())
        if not data:
            return
        try:
            text = data.decode("utf-8", errors="replace")
        except Exception:
            text = data.decode(errors="replace")
        self._append(text)

    def _on_error(self, _err) -> None:
        self._set_running(False)
        self._append(f"\n[ERROR] {self._proc.errorString()}\n")
        self._status.setText("Failed to start process.")

    def _on_finished(self, exit_code: int, _status) -> None:
        self._set_running(False)
        self._append(f"\n[EXIT {exit_code}]\n")
        self._status.setText("Done.")

    def _set_running(self, running: bool) -> None:
        self._btn_start.setEnabled(not running)
        self._btn_cancel.setEnabled(running)
        self._progress.setVisible(running)
        self._stdin.setEnabled(running)
        self._btn_send.setEnabled(running)
        if running:
            self._stdin.setFocus()

    def _fmt_cmd(self, argv: list[str]) -> str:
        # For display/copy-paste into CMD; QProcess does not need manual quoting.
        if sys.platform.startswith("win"):
            import subprocess

            return subprocess.list2cmdline(argv)
        return " ".join(argv)

    def _append(self, text: str) -> None:
        self._output.moveCursor(QTextCursor.End)
        self._output.insertPlainText(text)
        self._output.moveCursor(QTextCursor.End)

    def _placeholder_command(self) -> list[str]:
        # Keeps output streaming into the UI (no separate visible CMD window).
        if sys.platform.startswith("win"):
            return [
                "cmd.exe",
                "/c",
                "echo ModInstaller placeholder... && ping -n 2 127.0.0.1 > nul && echo Done.",
            ]
        return ["sh", "-lc", "echo ModInstaller placeholder...; sleep 1; echo Done."]
