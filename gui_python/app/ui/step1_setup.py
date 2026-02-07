from __future__ import annotations

from PySide6.QtCore import Qt, QSettings
from PySide6.QtWidgets import (
    QComboBox,
    QFileDialog,
    QGridLayout,
    QLabel,
    QLineEdit,
    QPushButton,
    QWidget,
)

from .page_frame import PageFrame


class Step1SetupPage(PageFrame):
    def __init__(self) -> None:
        super().__init__("Step 1 — Setup")
        self._settings = QSettings()
        self.set_body(self._build())
        self._load_settings()
        self._wire_settings()

    def _build(self) -> QWidget:
        host = QWidget()
        grid = QGridLayout(host)
        grid.setColumnStretch(1, 1)
        grid.setHorizontalSpacing(14)
        grid.setVerticalSpacing(12)

        r = 0
        grid.addWidget(QLabel("Game:"), r, 0, Qt.AlignRight)
        self.game_mode = QComboBox()
        self.game_mode.addItems(["BGEE", "BG2EE", "EET"])
        grid.addWidget(self.game_mode, r, 1, 1, 2)
        r += 1

        r = self._path_row_dir(grid, r, "Mods Folder:", "mods_dir")
        r = self._path_row_file(
            grid,
            r,
            "WeiDU Binary:",
            "weidu_exe",
            "Executables (*.exe);;All Files (*.*)",
        )

        r = self._path_row_file(
            grid,
            r,
            "Mod_Installer Binary:",
            "modinstaller_exe",
            "Executables (*.exe);;All Files (*.*)",
        )
        r = self._path_row_dir(grid, r, "BGEE Folder:", "bgee_dir")
        r = self._path_row_dir(grid, r, "BG2EE Folder:", "bg2ee_dir")

        self._sec_bgee_log = self._section_label("BGEE Log")
        grid.addWidget(self._sec_bgee_log, r, 0, 1, 3)
        r += 1
        r = self._path_row_dir(grid, r, "Log Folder:", "bgee_log")

        self._sec_bg2ee_log = self._section_label("BG2EE Log")
        grid.addWidget(self._sec_bg2ee_log, r, 0, 1, 3)
        r += 1
        r = self._path_row_dir(grid, r, "Log Folder:", "bg2ee_log")

        grid.setRowStretch(r, 1)
        return host

    def _path_row_dir(
        self, grid: QGridLayout, r: int, label: str, attr_name: str
    ) -> int:
        lbl = QLabel(label)
        grid.addWidget(lbl, r, 0, Qt.AlignRight)

        edit = QLineEdit()
        edit.setPlaceholderText("Select a folder…")
        setattr(self, attr_name, edit)

        btn = QPushButton("Browse…")
        btn.setFixedWidth(110)
        btn.clicked.connect(lambda _=False, e=edit, t=label: self._browse_dir(e, t))

        setattr(self, f"_{attr_name}_lbl", lbl)
        setattr(self, f"_{attr_name}_btn", btn)

        grid.addWidget(edit, r, 1)
        grid.addWidget(btn, r, 2)
        return r + 1

    def _path_row_file(
        self, grid: QGridLayout, r: int, label: str, attr_name: str, file_filter: str
    ) -> int:
        lbl = QLabel(label)
        grid.addWidget(lbl, r, 0, Qt.AlignRight)

        edit = QLineEdit()
        edit.setPlaceholderText("Select a file…")
        setattr(self, attr_name, edit)

        btn = QPushButton("Browse…")
        btn.setFixedWidth(110)
        btn.clicked.connect(
            lambda _=False, e=edit, t=label, f=file_filter: self._browse_file(e, t, f)
        )

        setattr(self, f"_{attr_name}_lbl", lbl)
        setattr(self, f"_{attr_name}_btn", btn)

        grid.addWidget(edit, r, 1)
        grid.addWidget(btn, r, 2)
        return r + 1

    def _browse_dir(self, edit: QLineEdit, title: str) -> None:
        start = edit.text().strip() or ""
        chosen = QFileDialog.getExistingDirectory(self, f"Select {title}", start)
        if chosen:
            edit.setText(chosen)
            self._save_settings()

    def _browse_file(self, edit: QLineEdit, title: str, file_filter: str) -> None:
        start = edit.text().strip() or ""
        chosen, _ = QFileDialog.getOpenFileName(
            self, f"Select {title}", start, file_filter
        )
        if chosen:
            edit.setText(chosen)
            self._save_settings()

    def _section_label(self, text: str) -> QLabel:
        lbl = QLabel(text)
        lbl.setStyleSheet("font-weight: 600; color: #e6e6e6; padding-top: 8px;")
        return lbl

    def _wire_settings(self) -> None:
        self.game_mode.currentIndexChanged.connect(self._on_game_changed)

        for attr in (
            "mods_dir",
            "weidu_exe",
            "modinstaller_exe",
            "bgee_dir",
            "bg2ee_dir",
            "bgee_log",
            "bg2ee_log",
        ):
            w = getattr(self, attr, None)
            if isinstance(w, QLineEdit):
                w.editingFinished.connect(self._save_settings)

    def _load_settings(self) -> None:
        self.game_mode.setCurrentIndex(int(self._settings.value("game_mode", 0)))
        self._apply_game_visibility()

        for attr in (
            "mods_dir",
            "weidu_exe",
            "modinstaller_exe",
            "bgee_dir",
            "bg2ee_dir",
            "bgee_log",
            "bg2ee_log",
        ):
            w = getattr(self, attr, None)
            if isinstance(w, QLineEdit):
                w.setText(str(self._settings.value(attr, "")))

    def _on_game_changed(self, _idx: int) -> None:
        self._apply_game_visibility()
        self._save_settings()

    def _apply_game_visibility(self) -> None:
        mode = int(self.game_mode.currentIndex())  # 0=BGEE, 1=BG2EE, 2=EET
        show_bgee = mode in (0, 2)
        show_bg2 = mode in (1, 2)

        self._set_row_visible("bgee_dir", show_bgee)
        self._set_row_visible("bgee_log", show_bgee)
        if hasattr(self, "_sec_bgee_log"):
            self._sec_bgee_log.setVisible(show_bgee)

        self._set_row_visible("bg2ee_dir", show_bg2)
        self._set_row_visible("bg2ee_log", show_bg2)
        if hasattr(self, "_sec_bg2ee_log"):
            self._sec_bg2ee_log.setVisible(show_bg2)

    def _set_row_visible(self, attr_name: str, visible: bool) -> None:
        edit = getattr(self, attr_name, None)
        lbl = getattr(self, f"_{attr_name}_lbl", None)
        btn = getattr(self, f"_{attr_name}_btn", None)
        if isinstance(edit, QWidget):
            edit.setVisible(visible)
        if isinstance(lbl, QWidget):
            lbl.setVisible(visible)
        if isinstance(btn, QWidget):
            btn.setVisible(visible)

    def _save_settings(self) -> None:
        self._settings.setValue("game_mode", int(self.game_mode.currentIndex()))

        for attr in (
            "mods_dir",
            "weidu_exe",
            "modinstaller_exe",
            "bgee_dir",
            "bg2ee_dir",
            "bgee_log",
            "bg2ee_log",
        ):
            w = getattr(self, attr, None)
            if isinstance(w, QLineEdit):
                self._settings.setValue(attr, w.text().strip())
