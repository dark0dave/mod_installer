from __future__ import annotations

from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QLabel,
    QPlainTextEdit,
    QPushButton,
    QTabWidget,
    QVBoxLayout,
    QWidget,
)

from .page_frame import PageFrame


class Step4ExportPage(PageFrame):
    install_requested = Signal()

    def __init__(self) -> None:
        super().__init__("Step 4 — Preview & Export")
        self._mode = 0  # 0=BGEE, 1=BG2EE, 2=EET
        self._bgee_log_dir = ""
        self._bg2ee_log_dir = ""

        self._preview_bgee = self._make_preview()
        self._preview_bg2ee = self._make_preview()

        self._tabs = QTabWidget()
        self._tabs.addTab(self._preview_bgee, "BGEE")
        self._tabs.addTab(self._preview_bg2ee, "BG2EE")
        self._tabs.tabBar().hide()

        self._btn_save = QPushButton("Save (weidu.log)")
        self._btn_install = QPushButton("Install")
        self._btn_install.setEnabled(False)

        self._status = QLabel("Ready.")
        self._status.setStyleSheet("color: #bdbdbd;")

        self.set_body(self._build())
        self._wire()
        self._load_placeholder_previews()

    # Public hook (wire this from Step1 later)
    def set_game_mode(self, mode: int) -> None:
        # 0=BGEE, 1=BG2EE, 2=EET
        self._mode = int(mode)
        self._tabs.tabBar().setVisible(self._mode == 2)
        self._tabs.setCurrentIndex(1 if self._mode == 1 else 0)

    # Public hook (wire this from Step3 later)
    def set_log_dirs(self, bgee_log_dir: str, bg2ee_log_dir: str) -> None:
        self._bgee_log_dir = (bgee_log_dir or "").strip()
        self._bg2ee_log_dir = (bg2ee_log_dir or "").strip()

    def set_preview_lines(self, bgee_lines: list[str], bg2ee_lines: list[str]) -> None:
        self._preview_bgee.setPlainText(self._join_weidulog(bgee_lines))
        self._preview_bg2ee.setPlainText(self._join_weidulog(bg2ee_lines))

    def _build(self) -> QWidget:
        host = QWidget()
        root = QVBoxLayout(host)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(10)

        preview_panel = QFrame()
        preview_panel.setObjectName("Panel")
        pv = QVBoxLayout(preview_panel)
        pv.setContentsMargins(12, 12, 12, 12)
        pv.setSpacing(10)

        actions = QHBoxLayout()
        actions.setSpacing(10)
        actions.addWidget(self._btn_save)
        actions.addWidget(self._btn_install)
        actions.addStretch(1)

        pv.addLayout(actions)
        pv.addWidget(self._tabs, 1)

        root.addWidget(preview_panel, 1)
        root.addWidget(self._status)

        return host

    def _wire(self) -> None:
        self._btn_save.clicked.connect(self._on_save_clicked)
        self._btn_install.clicked.connect(self._on_install_clicked)

    def _make_preview(self) -> QPlainTextEdit:
        w = QPlainTextEdit()
        w.setReadOnly(True)
        w.setLineWrapMode(QPlainTextEdit.NoWrap)
        w.setPlaceholderText("Preview will appear here…")
        return w

    def _load_placeholder_previews(self) -> None:
        self._preview_bgee.setPlainText(
            self._join_weidulog(
                [
                    r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #100 // (BGEE) Install in batch mode: 35.21",
                    r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #1500 // (BGEE) Include IWD arcane spells: 35.21",
                ]
            )
        )
        self._preview_bg2ee.setPlainText(
            self._join_weidulog(
                [
                    r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2010 // (BG2EE) Core changes: 35.21",
                    r"~SOUTHERNEDGE\SETUP-SOUTHERNEDGE.TP2~ #0 #0 // (BG2EE) Core component: 1.0",
                ]
            )
        )

    def _on_save_clicked(self) -> None:
        need_bgee = self._mode in (0, 2)
        need_bg2 = self._mode in (1, 2)

        if need_bgee and not self._bgee_log_dir:
            self._status.setText("Set BGEE log folder in Step 1 first.")
            return
        if need_bg2 and not self._bg2ee_log_dir:
            self._status.setText("Set BG2EE log folder in Step 1 first.")
            return

        from pathlib import Path

        bgee_dir = Path(self._bgee_log_dir)
        bg2_dir = Path(self._bg2ee_log_dir)

        try:
            if need_bgee:
                bgee_dir.mkdir(parents=True, exist_ok=True)
            if need_bg2:
                bg2_dir.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            self._status.setText(f"Failed to create log folder(s): {e}")
            return

        bgee_text = self._preview_bgee.toPlainText()
        bg2_text = self._preview_bg2ee.toPlainText()

        if need_bgee and not bgee_text.strip():
            self._status.setText("BGEE preview is empty.")
            return
        if need_bg2 and not bg2_text.strip():
            self._status.setText("BG2EE preview is empty.")
            return

        try:
            if need_bgee:
                (bgee_dir / "weidu.log").write_text(
                    bgee_text, encoding="utf-8", newline="\r\n"
                )
            if need_bg2:
                (bg2_dir / "weidu.log").write_text(
                    bg2_text, encoding="utf-8", newline="\r\n"
                )
        except OSError as e:
            self._status.setText(f"Failed to write weidu.log: {e}")
            return

        self._btn_install.setEnabled(True)
        if self._mode == 0:
            self._status.setText("Saved BGEE\\weidu.log")
        elif self._mode == 1:
            self._status.setText("Saved BG2EE\\weidu.log")
        else:
            self._status.setText("Saved BGEE\\weidu.log and BG2EE\\weidu.log")

    def _on_install_clicked(self) -> None:
        self.install_requested.emit()

    def _join_weidulog(self, lines: list[str]) -> str:
        return "\r\n".join(lines).strip() + ("\r\n" if lines else "")
