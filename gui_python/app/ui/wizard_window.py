from __future__ import annotations

from PySide6.QtWidgets import (
    QHBoxLayout,
    QPushButton,
    QStackedWidget,
    QVBoxLayout,
    QWidget,
)

from .step1_setup import Step1SetupPage
from .step2_scan_select import Step2ScanSelectPage
from .step3_order import Step3OrderPage
from .step4_export import Step4ExportPage
from .step5_install import Step5InstallPage


class WizardWindow(QWidget):
    def __init__(self) -> None:
        super().__init__()
        self.setWindowTitle("WeiDU Log Builder")
        self.resize(1280, 780)

        self._stack = QStackedWidget()

        self._p1 = Step1SetupPage()
        self._p2 = Step2ScanSelectPage()
        self._p3 = Step3OrderPage()
        self._p4 = Step4ExportPage()
        self._p5 = Step5InstallPage()

        self._pages = [self._p1, self._p2, self._p3, self._p4, self._p5]
        for p in self._pages:
            self._stack.addWidget(p)

        self._p1.game_mode.currentIndexChanged.connect(self._on_mode_changed)
        self._p4.install_requested.connect(self._on_install_requested)
        self._on_mode_changed()

        self._back = QPushButton("◀ Back")
        self._next = QPushButton("Next ▶")
        self._back.clicked.connect(self._go_back)
        self._next.clicked.connect(self._go_next)

        footer = QHBoxLayout()
        footer.addStretch(1)
        footer.addWidget(self._back)
        footer.addWidget(self._next)

        root = QVBoxLayout(self)
        root.setContentsMargins(14, 10, 14, 12)
        root.addWidget(self._stack, 1)
        root.addLayout(footer)

        self._sync_nav()

    def _go_back(self) -> None:
        self._stack.setCurrentIndex(max(0, self._stack.currentIndex() - 1))
        self._sync_nav()

    def _go_next(self) -> None:
        idx = self._stack.currentIndex()

        if idx == 0:  # Step 1 -> Step 2
            mode = int(self._p1.game_mode.currentIndex())  # 0=BGEE, 1=BG2EE, 2=EET
            self._p2.set_scan_config(
                weidu_exe=self._p1.weidu_exe.text().strip(),
                mods_dir=self._p1.mods_dir.text().strip(),
                bgee_dir=self._p1.bgee_dir.text().strip(),
                bg2ee_dir=self._p1.bg2ee_dir.text().strip(),
                mode=mode,
            )

        if idx == 1:  # Step 2 -> Step 3
            bgee, bg2ee = self._p2.get_checked_weidulog_lines()
            self._p3.set_install_lines(bgee, bg2ee)

        if idx == 2:  # Step 3 -> Step 4
            bgee, bg2ee = self._p3.get_install_lines()
            self._p4.set_preview_lines(bgee, bg2ee)
            self._p4.set_log_dirs(self._p1.bgee_log.text(), self._p1.bg2ee_log.text())

        self._stack.setCurrentIndex(min(self._stack.count() - 1, idx + 1))
        self._sync_nav()

    def _build_modinstaller_argv(self, mode: int) -> list[str]:
        from pathlib import Path

        exe = self._p1.modinstaller_exe.text().strip()
        weidu = self._p1.weidu_exe.text().strip()
        mods = self._p1.mods_dir.text().strip()

        bgee_dir = self._p1.bgee_dir.text().strip()
        bg2_dir = self._p1.bg2ee_dir.text().strip()

        bgee_log_dir = self._p1.bgee_log.text().strip()
        bg2_log_dir = self._p1.bg2ee_log.text().strip()

        bgee_log_file = str(Path(bgee_log_dir) / "weidu.log") if bgee_log_dir else ""
        bg2_log_file = str(Path(bg2_log_dir) / "weidu.log") if bg2_log_dir else ""

        if not exe or not weidu or not mods:
            return []

        if mode == 0:  # BGEE
            if not bgee_dir or not bgee_log_file:
                return []
            return [
                exe,
                "-n",
                "-w",
                weidu,
                "-m",
                mods,
                "-g",
                bgee_dir,
                "-f",
                bgee_log_file,
                "-c",
            ]

        if mode == 1:  # BG2EE
            if not bg2_dir or not bg2_log_file:
                return []
            return [
                exe,
                "-n",
                "-w",
                weidu,
                "-m",
                mods,
                "-g",
                bg2_dir,
                "-f",
                bg2_log_file,
                "-c",
            ]

        # EET
        if not bgee_dir or not bg2_dir or not bgee_log_file or not bg2_log_file:
            return []
        return [
            exe,
            "-e",
            "-1",
            bgee_dir,
            "-y",
            bgee_log_file,
            "-2",
            bg2_dir,
            "-z",
            bg2_log_file,
            "-w",
            weidu,
            "-m",
            mods,
            "-c",
        ]

    def _on_install_requested(self) -> None:
        mode = int(self._p1.game_mode.currentIndex())  # 0=BGEE, 1=BG2EE, 2=EET

        argv = self._build_modinstaller_argv(mode)
        self._stack.setCurrentIndex(4)
        self._sync_nav()
        if not argv:
            self._p5._append(
                "ERROR: missing/invalid Step 1 paths for this Game mode.\n"
            )
            return

        from pathlib import Path

        cwd = str(Path(argv[0]).parent)
        self._p5.start_install(argv, cwd=cwd)

    def _on_mode_changed(self) -> None:
        mode = int(self._p1.game_mode.currentIndex())  # 0=BGEE, 1=BG2EE, 2=EET
        self._p2.set_game_mode(mode)
        self._p3.set_game_mode(mode)
        self._p4.set_game_mode(mode)

    def _sync_nav(self) -> None:
        idx = self._stack.currentIndex()
        self._back.setVisible(idx > 0)
        self._back.setEnabled(idx > 0)
        self._next.setVisible(idx != 3)  # hide on Step 4
        self._next.setEnabled(idx < self._stack.count() - 1)
