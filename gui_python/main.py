from __future__ import annotations

import sys
import os.path
from pathlib import Path

from PySide6.QtWidgets import QApplication

from app.ui.wizard_window import WizardWindow


def _load_qss(app: QApplication) -> None:
    qss_path = Path(
        os.path.join(Path(__file__).resolve().parent, "app", "resources", "theme.qss")
    )
    if qss_path.exists():
        app.setStyleSheet(qss_path.read_text(encoding="utf-8"))


def main() -> int:
    app = QApplication(sys.argv)
    app.setOrganizationName("mod_installer")
    app.setApplicationName("gui")
    _load_qss(app)

    w = WizardWindow()
    w.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
