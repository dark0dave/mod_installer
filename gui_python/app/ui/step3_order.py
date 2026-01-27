from __future__ import annotations

import re
from dataclasses import dataclass

from PySide6.QtCore import Qt
from PySide6.QtGui import QStandardItem, QStandardItemModel
from PySide6.QtWidgets import (
    QAbstractItemView,
    QFrame,
    QLabel,
    QListView,
    QTabWidget,
    QVBoxLayout,
    QWidget,
)

from .page_frame import PageFrame


class _ReorderListView(QListView):
    """Live-reordering drag/drop: rows shift while dragging (no overwrite, no vanish)."""

    def __init__(self) -> None:
        super().__init__()
        self._drag_row: int | None = None
        self._in_move = False

    def startDrag(self, supported_actions) -> None:  # type: ignore[override]
        idx = self.currentIndex()
        self._drag_row = idx.row() if idx.isValid() else None
        try:
            super().startDrag(supported_actions)
        finally:
            self._drag_row = None
            self._in_move = False

    def dragMoveEvent(self, event) -> None:  # type: ignore[override]
        if self._drag_row is None or self._in_move:
            event.acceptProposedAction()
            return

        model = self.model()
        if not isinstance(model, QStandardItemModel):
            super().dragMoveEvent(event)
            return

        pos = event.position().toPoint()
        idx = self.indexAt(pos)
        if not idx.isValid():
            if model.rowCount() == 0:
                event.acceptProposedAction()
                return
            last_idx = model.index(model.rowCount() - 1, 0)
            last_rect = self.visualRect(last_idx)
            if pos.y() > last_rect.bottom():
                idx = last_idx
            else:
                event.acceptProposedAction()
                return

        src = self._drag_row
        dst = idx.row()
        if pos.y() > self.visualRect(idx).bottom():
            dst += 1
        if dst == src:
            event.acceptProposedAction()
            return

        self._in_move = True
        try:
            items = model.takeRow(src)
            if dst > src:
                dst -= 1
            model.insertRow(dst, items)
            self._drag_row = dst
            self.setCurrentIndex(model.index(dst, 0))
        finally:
            self._in_move = False

        event.acceptProposedAction()

    def dropEvent(self, event) -> None:  # type: ignore[override]
        # We already moved rows during dragMoveEvent; suppress default dropMimeData logic.
        event.setDropAction(Qt.IgnoreAction)
        event.accept()


_LOG_LINE_RE = re.compile(r"~(?P<tp2>[^~]+)~\s+#\d+\s+#(?P<cid>\d+)", re.IGNORECASE)


@dataclass(slots=True)
class _ListBundle:
    host: QWidget
    view: QListView
    model: QStandardItemModel
    original_lines: list[str]


class Step3OrderPage(PageFrame):
    def __init__(self) -> None:
        super().__init__(
            "Step 3 — Reorder Components",
            "Drag and drop to reorder.",
        )

        self._tabs = QTabWidget()
        self._main = self._build_one_list()
        self._bgee = self._build_one_list()

        self._tabs.addTab(self._main.host, "BGEE")
        self._tabs.addTab(self._bgee.host, "BG2EE")
        self._tabs.setTabEnabled(1, False)
        self._tabs.tabBar().hide()

        self._count = QLabel("0 component(s) in install order.")
        self._count.setStyleSheet("color: #bdbdbd;")

        self.set_body(self._build())
        self.set_install_lines([], [])

    # Public hook (wire this from Step1 later)
    def set_game_mode(self, mode: int) -> None:
        # 0=BGEE, 1=BG2EE, 2=EET
        is_eet = mode == 2
        self._tabs.setTabEnabled(1, is_eet or mode == 1)
        self._tabs.tabBar().setVisible(is_eet)
        self._tabs.setCurrentIndex(1 if mode == 1 else 0)
        self._sync_count()

    def _build(self) -> QWidget:
        host = QWidget()
        root = QVBoxLayout(host)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(10)

        list_panel = QFrame()
        list_panel.setObjectName("Panel")
        list_lay = QVBoxLayout(list_panel)
        list_lay.setContentsMargins(10, 10, 10, 10)
        list_lay.addWidget(self._tabs, 1)
        root.addWidget(list_panel, 1)

        # (Buttons removed; drag/drop only.)

        root.addWidget(self._count)

        # (No extra actions.)

        self._tabs.currentChanged.connect(lambda _i: self._sync_count())
        self._wire_count_signals(self._main.model)
        self._wire_count_signals(self._bgee.model)

        return host

    def _build_one_list(self) -> _ListBundle:
        model = QStandardItemModel()

        view = _ReorderListView()
        view.setModel(model)
        view.setAlternatingRowColors(True)
        view.setUniformItemSizes(True)
        view.setSelectionMode(QAbstractItemView.SingleSelection)
        view.setEditTriggers(QAbstractItemView.NoEditTriggers)
        view.setDragEnabled(True)
        view.setAcceptDrops(True)
        view.setDropIndicatorShown(True)
        view.setDragDropOverwriteMode(False)
        view.setDragDropMode(QAbstractItemView.InternalMove)
        view.setDefaultDropAction(Qt.MoveAction)

        host = QWidget()
        lay = QVBoxLayout(host)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.addWidget(view, 1)

        return _ListBundle(host=host, view=view, model=model, original_lines=[])

    def _wire_count_signals(self, model: QStandardItemModel) -> None:
        model.rowsInserted.connect(lambda _p, _a, _b: self._sync_count())
        model.rowsRemoved.connect(lambda _p, _a, _b: self._sync_count())
        model.rowsMoved.connect(lambda _p, _s, _e, _dp, _dr: self._sync_count())
        model.modelReset.connect(self._sync_count)

    def _active(self) -> _ListBundle:
        return self._bgee if self._tabs.currentIndex() == 1 else self._main

    def _sync_count(self) -> None:
        active = self._active()
        self._count.setText(f"{active.model.rowCount()} component(s) in install order.")

    def _on_reset(self) -> None:
        active = self._active()
        if not active.original_lines:
            return
        self._set_lines(active, list(active.original_lines))

    def _on_auto_sort(self) -> None:
        active = self._active()
        lines = self._get_lines(active)
        lines.sort(key=self._sort_key_for_line)
        self._set_lines(active, lines)

    def _sort_key_for_line(self, line: str) -> tuple[str, int, str]:
        m = _LOG_LINE_RE.search(line)
        if not m:
            return ("~", 10**9, line.casefold())
        tp2 = m.group("tp2").replace("\\", "/").casefold()
        try:
            cid = int(m.group("cid"))
        except ValueError:
            cid = 10**9
        return (tp2, cid, line.casefold())

    def _get_lines(self, bundle: _ListBundle) -> list[str]:
        out: list[str] = []
        for r in range(bundle.model.rowCount()):
            item = bundle.model.item(r, 0)
            if item is not None:
                out.append(item.text())
        return out

    def _set_lines(self, bundle: _ListBundle, lines: list[str]) -> None:
        bundle.model.clear()
        for s in lines:
            item = QStandardItem(s)
            item.setEditable(False)
            item.setFlags(item.flags() | Qt.ItemIsDragEnabled | Qt.ItemIsDropEnabled)
            bundle.model.appendRow(item)
        self._sync_count()

    def set_install_lines(self, bgee_lines: list[str], bg2ee_lines: list[str]) -> None:
        self._main.original_lines = list(bgee_lines)
        self._bgee.original_lines = list(bg2ee_lines)
        self._set_lines(self._main, list(bgee_lines))
        self._set_lines(self._bgee, list(bg2ee_lines))
        self._sync_count()

    def get_install_lines(self) -> tuple[list[str], list[str]]:
        return (self._get_lines(self._main), self._get_lines(self._bgee))

    def _load_placeholder_data(self, bundle: _ListBundle) -> None:
        lines = [
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #100 // Install in batch mode (ask about all components before starting installation). DO NOT USE WITH PROJECT INFINITY.: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #1500 // Include arcane spells from Icewind Dale: Enhanced Edition: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #1510 // Include divine spells from Icewind Dale: Enhanced Edition: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #1520 // Include bard songs from Icewind Dale: Enhanced Edition: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2000 // Install all spell tweaks (if you don't select this, you will be given a chance to choose by category): 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2010 // Core Stratagems spell-system changes (installed by default by any AI component): 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2030 // Changes to Restoration: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2040 // Changes to shapeshift spells: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2050 // Icewind Dale-inspired tweaks to Baldur's Gate/Baldur's Gate II spells: 35.21",
            r"~STRATAGEMS\SETUP-STRATAGEMS.TP2~ #0 #2060 // Rebalancings of slightly-too-powerful spells: 35.21",
            r"~SOUTHERNEDGE\SETUP-SOUTHERNEDGE.TP2~ #0 #0 // Core component: 1.0",
        ]
        bundle.original_lines = list(lines)
        self._set_lines(bundle, lines)
        if bundle.model.rowCount() > 0:
            bundle.view.setCurrentIndex(bundle.model.index(0, 0))
            bundle.view.scrollTo(
                bundle.model.index(0, 0), QAbstractItemView.PositionAtTop
            )
