from __future__ import annotations

from PySide6.QtWidgets import QFrame, QLabel, QVBoxLayout, QWidget


class PageFrame(QFrame):
    def __init__(
        self, title: str, subtitle: str | None = None, parent: QWidget | None = None
    ) -> None:
        super().__init__(parent)
        self._root = QVBoxLayout(self)
        self._root.setContentsMargins(26, 20, 26, 16)
        self._root.setSpacing(12)

        self._title = QLabel(title)
        self._title.setObjectName("PageTitle")
        self._root.addWidget(self._title)

        if subtitle:
            self._subtitle = QLabel(subtitle)
            self._subtitle.setObjectName("PageSubtitle")
            self._root.addWidget(self._subtitle)

        self._body_host = QFrame()
        self._body_host.setObjectName("Panel")
        self._body_layout = QVBoxLayout(self._body_host)
        self._body_layout.setContentsMargins(16, 14, 16, 14)
        self._body_layout.setSpacing(10)
        self._root.addWidget(self._body_host, 1)

    def set_body(self, widget: QWidget) -> None:
        while self._body_layout.count():
            item = self._body_layout.takeAt(0)
            w = item.widget()
            if w is not None:
                w.setParent(None)
        self._body_layout.addWidget(widget, 1)
