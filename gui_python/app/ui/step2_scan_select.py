from __future__ import annotations

from dataclasses import dataclass
from concurrent.futures import ThreadPoolExecutor, as_completed
import os
import re
import subprocess
from pathlib import Path
from urllib.parse import quote_plus

from PySide6.QtCore import (
    Qt,
    QSortFilterProxyModel,
    QModelIndex,
    QObject,
    Signal,
    QThread,
    QRegularExpression,
    QUrl,
    QSignalBlocker,
    QEvent,
)
from PySide6.QtGui import (
    QBrush,
    QColor,
    QDesktopServices,
    QStandardItem,
    QStandardItemModel,
)
from PySide6.QtWidgets import (
    QAbstractItemView,
    QCheckBox,
    QFrame,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPushButton,
    QPlainTextEdit,
    QSizePolicy,
    QHeaderView,
    QSplitter,
    QTabWidget,
    QTreeView,
    QVBoxLayout,
    QWidget,
)

from .page_frame import PageFrame


class _IssueFilterProxy(QSortFilterProxyModel):
    def __init__(self) -> None:
        super().__init__()
        self._show_only_issues = False

    def set_show_only_issues(self, on: bool) -> None:
        self._show_only_issues = bool(on)
        self.invalidateFilter()

    def filterAcceptsRow(self, source_row: int, source_parent: QModelIndex) -> bool:  # type: ignore[override]
        if not super().filterAcceptsRow(source_row, source_parent):
            return False
        if not self._show_only_issues:
            return True
        src = self.sourceModel()
        if src is None:
            return True
        idx = src.index(source_row, 0, source_parent)
        if not idx.isValid():
            return True
        item = src.itemFromIndex(idx)
        if item is None:
            return True
        issue = item.data(_ISSUE_ROLE)
        if issue in ("missing", "conflict"):
            return True
        for r in range(item.rowCount()):
            if self._has_issue(item.child(r, 0)):
                return True
        return False

    def _has_issue(self, item: QStandardItem | None) -> bool:
        if item is None:
            return False
        issue = item.data(_ISSUE_ROLE)
        if issue in ("missing", "conflict"):
            return True
        for r in range(item.rowCount()):
            if self._has_issue(item.child(r, 0)):
                return True
        return False


@dataclass(frozen=True, slots=True)
class ComponentDetails:
    mod_rel: str
    tp2_abs: str
    component_name: str
    component_id: int
    version: str | None = None
    designated: int | None = None
    game_allowed: str | None = None
    allowed_games: tuple[str, ...] = ()
    dependencies: tuple[str, ...] = ()
    conflicts: tuple[str, ...] = ()


_DETAILS_ROLE = Qt.UserRole + 10
_ORDER_ROLE = Qt.UserRole + 11
_ISSUE_ROLE = Qt.UserRole + 12  # "ok" | "missing" | "conflict"


@dataclass(frozen=True, slots=True)
class _ScanConfig:
    weidu_exe: str
    mods_dir: str
    bgee_dir: str
    bg2ee_dir: str
    mode: int  # 0=BGEE, 1=BG2EE, 2=EET


@dataclass(frozen=True, slots=True)
class _ScannedMod:
    tp2_rel: str
    tp2_abs: str
    components: tuple[
        tuple[int, str, str | None, tuple[str, ...], tuple[str, ...], tuple[str, ...]],
        ...,
    ]  # (id, name, version, deps, allowed_games, conflicts)


class Step2ScanSelectPage(PageFrame):
    def __init__(self) -> None:
        super().__init__("Step 2 — Select Components")
        self._search: QLineEdit
        self._status: QLabel

        self._tabs = QTabWidget()
        self._tab_main = self._build_one_tree()
        self._tab_bgee = self._build_one_tree()

        self._tabs.addTab(self._tab_main.host, "BGEE")
        self._tabs.addTab(self._tab_bgee.host, "BG2EE")
        self._tabs.setTabEnabled(1, False)
        self._tabs.tabBar().hide()
        self._tabs.currentChanged.connect(lambda _i: self._apply_game_compatibility())
        self._tabs.currentChanged.connect(
            lambda _i: self._fit_columns(self._active_tree().view)
        )
        self._tabs.currentChanged.connect(lambda _i: self._update_issue_summary())

        self._details_text = QPlainTextEdit()
        self._details_text.setReadOnly(True)
        self._details_text.setLineWrapMode(QPlainTextEdit.WidgetWidth)

        self._scan_cfg: _ScanConfig | None = None
        self._scan_thread: QThread | None = None
        self._scan_worker: _ScanWorker | None = None
        self._check_seq = 0
        self._check_sync = False
        self._issue_counts = {"bgee": (0, 0), "bg2ee": (0, 0)}

        self.set_body(self._build())
        self._details_text.setPlainText("")

    def set_scan_config(
        self,
        *,
        weidu_exe: str,
        mods_dir: str,
        bgee_dir: str,
        bg2ee_dir: str,
        mode: int,
    ) -> None:
        self._scan_cfg = _ScanConfig(
            weidu_exe=weidu_exe.strip(),
            mods_dir=mods_dir.strip(),
            bgee_dir=bgee_dir.strip(),
            bg2ee_dir=bg2ee_dir.strip(),
            mode=int(mode),
        )

    # Public hook (wire this from Step1 later)
    def set_game_mode(self, mode: int) -> None:
        # 0=BGEE, 1=BG2EE, 2=EET
        is_eet = mode == 2
        self._tabs.tabBar().setVisible(is_eet)
        self._tabs.setTabEnabled(1, is_eet or mode == 1)
        self._tabs.setCurrentIndex(1 if mode == 1 else 0)
        self._sync_details_from_current(self._active_tree().view.currentIndex())
        self._apply_game_compatibility()

    def _build(self) -> QWidget:
        host = QWidget()
        root = QVBoxLayout(host)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(10)

        root.addLayout(self._build_search_row())
        root.addLayout(self._build_actions_row())

        split = QSplitter(Qt.Horizontal)
        split.setChildrenCollapsible(False)

        left_panel = QFrame()
        left_panel.setObjectName("Panel")
        left_lay = QVBoxLayout(left_panel)
        left_lay.setContentsMargins(10, 10, 10, 10)
        self._issue_summary = QLabel("Issues: Missing 0 / Conflicts 0")
        self._issue_summary.setStyleSheet("color: #bdbdbd;")
        left_lay.addWidget(self._issue_summary)
        left_lay.addWidget(self._tabs, 1)

        right_panel = QFrame()
        right_panel.setObjectName("Panel")
        right_lay = QVBoxLayout(right_panel)
        right_lay.setContentsMargins(10, 10, 10, 10)
        right_lay.setSpacing(10)

        title = QLabel("Component Details:")
        title.setStyleSheet("font-weight: 600;")
        right_lay.addWidget(title)

        right_lay.addLayout(self._build_details_actions_row())
        right_lay.addWidget(self._details_text, 1)

        split.addWidget(left_panel)
        split.addWidget(right_panel)
        split.setStretchFactor(0, 3)
        split.setStretchFactor(1, 2)

        root.addWidget(split, 1)

        self._status = QLabel("Ready.")
        self._status.setStyleSheet("color: #bdbdbd;")
        root.addWidget(self._status)

        self._wire_models()
        return host

    def _build_search_row(self) -> QHBoxLayout:
        row = QHBoxLayout()
        row.setSpacing(10)
        row.addWidget(QLabel("Search:"))

        self._search = QLineEdit()
        self._search.setPlaceholderText("Type to filter components…")
        row.addWidget(self._search, 1)
        return row

    def _build_actions_row(self) -> QHBoxLayout:
        row = QHBoxLayout()
        row.setSpacing(10)

        self._btn_scan = QPushButton("Scan Mods Folder")
        row.addWidget(self._btn_scan)
        self._toggle_issues = QCheckBox("Show only issues")
        row.addWidget(self._toggle_issues)
        row.addStretch(1)

        self._btn_scan.clicked.connect(self._on_scan_clicked)
        self._toggle_issues.toggled.connect(self._on_issues_filter_changed)
        return row

    def _build_details_actions_row(self) -> QHBoxLayout:
        row = QHBoxLayout()
        row.setSpacing(10)

        self._btn_readme = QPushButton("Readme")
        self._btn_web = QPushButton("Web")
        for b in (self._btn_readme, self._btn_web):
            b.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
            row.addWidget(b)
        row.addStretch(1)
        return row

    def _build_one_tree(self) -> "_TreeBundle":
        model = QStandardItemModel()
        model.setHorizontalHeaderLabels(["Mod / Component", "ID", "Status"])

        proxy = _IssueFilterProxy()
        proxy.setSourceModel(model)
        proxy.setFilterKeyColumn(0)
        proxy.setRecursiveFilteringEnabled(True)
        proxy.setDynamicSortFilter(True)

        view = QTreeView()
        view.setModel(proxy)
        view.setAlternatingRowColors(True)
        view.setUniformRowHeights(False)
        view.setWordWrap(True)
        view.setTextElideMode(Qt.ElideNone)
        view.setAllColumnsShowFocus(True)
        view.setSelectionBehavior(QAbstractItemView.SelectRows)
        view.setSelectionMode(QAbstractItemView.SingleSelection)
        view.setEditTriggers(QAbstractItemView.NoEditTriggers)
        view.setIndentation(18)
        view.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        view.installEventFilter(self)
        header = view.header()
        header.setStretchLastSection(False)
        header.setCascadingSectionResizes(True)
        header.setSectionResizeMode(0, QHeaderView.Stretch)
        header.setSectionResizeMode(1, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(2, QHeaderView.ResizeToContents)

        host = QWidget()
        lay = QVBoxLayout(host)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.addWidget(view, 1)

        return _TreeBundle(host=host, view=view, model=model, proxy=proxy)

    def _wire_models(self) -> None:
        self._search.textChanged.connect(self._on_filter_changed)

        self._tab_main.view.selectionModel().currentChanged.connect(
            self._on_current_changed
        )
        self._tab_bgee.view.selectionModel().currentChanged.connect(
            self._on_current_changed
        )
        self._tab_main.model.itemChanged.connect(self._on_item_changed)
        self._tab_bgee.model.itemChanged.connect(self._on_item_changed)
        self._btn_readme.clicked.connect(self._on_readme_clicked)
        self._btn_web.clicked.connect(self._on_web_clicked)

    def _active_tree(self) -> "_TreeBundle":
        return self._tab_bgee if self._tabs.currentIndex() == 1 else self._tab_main

    def eventFilter(self, obj: QObject, event: QEvent) -> bool:  # type: ignore[override]
        if isinstance(obj, QTreeView) and event.type() == QEvent.Resize:
            self._fit_columns(obj)
        return super().eventFilter(obj, event)

    def _on_filter_changed(self, text: str) -> None:
        bundle = self._active_tree()
        q = text.strip()
        if not q:
            bundle.proxy.setFilterRegularExpression(QRegularExpression())
            bundle.view.collapseAll()
            self._refresh_dependency_status()
            return

        rx = QRegularExpression(re.escape(q), QRegularExpression.CaseInsensitiveOption)
        bundle.proxy.setFilterRegularExpression(rx)
        bundle.view.expandAll()
        self._refresh_dependency_status()

    def _on_issues_filter_changed(self, on: bool) -> None:
        bundle = self._active_tree()
        if isinstance(bundle.proxy, _IssueFilterProxy):
            bundle.proxy.set_show_only_issues(on)
            bundle.view.expandAll()
        self._refresh_dependency_status()

    def _on_current_changed(self, current: QModelIndex, _prev: QModelIndex) -> None:
        self._sync_details_from_current(current)

    def _on_item_changed(self, item: QStandardItem) -> None:
        if self._check_sync:
            return
        if item.column() != 0 or not item.isCheckable():
            return

        # Parent (mod) toggled: apply to all children.
        if item.rowCount() > 0:
            state = item.checkState()
            if state == Qt.PartiallyChecked:
                # Auto-tristate update from children; never cascade from this.
                return

            any_checked = False
            any_unchecked = False
            for r in range(item.rowCount()):
                child = item.child(r, 0)
                if child is None or not child.isCheckable():
                    continue
                if child.checkState() == Qt.Checked:
                    any_checked = True
                elif child.checkState() == Qt.Unchecked:
                    any_unchecked = True

            # If the parent state already matches its children, this is not a user toggle.
            if state == Qt.Checked and not any_unchecked:
                return
            if state == Qt.Unchecked and not any_checked:
                return

            game_token = self._current_game_token()
            self._check_sync = True
            try:
                for r in range(item.rowCount()):
                    child = item.child(r, 0)
                    if child is None or not child.isCheckable():
                        continue
                    details = child.data(_DETAILS_ROLE)
                    if isinstance(details, ComponentDetails):
                        if not self._is_game_allowed(details.allowed_games, game_token):
                            if child.checkState() != Qt.Unchecked:
                                child.setCheckState(Qt.Unchecked)
                            child.setData(None, _ORDER_ROLE)
                            continue

                    if child.checkState() != state:
                        child.setCheckState(state)

                    if state == Qt.Checked:
                        if not isinstance(child.data(_ORDER_ROLE), int):
                            self._check_seq += 1
                            child.setData(self._check_seq, _ORDER_ROLE)
                    elif state == Qt.Unchecked:
                        child.setData(None, _ORDER_ROLE)
            finally:
                self._check_sync = False
            return

        # Leaf (component) toggled: stamp/clear order.
        if item.checkState() == Qt.Checked:
            details = item.data(_DETAILS_ROLE)
            if isinstance(details, ComponentDetails):
                game_token = self._current_game_token()
                if not self._is_game_allowed(details.allowed_games, game_token):
                    self._check_sync = True
                    try:
                        item.setCheckState(Qt.Unchecked)
                        item.setData(None, _ORDER_ROLE)
                    finally:
                        self._check_sync = False
                    return
            if not isinstance(item.data(_ORDER_ROLE), int):
                self._check_sync = True
                try:
                    self._check_seq += 1
                    item.setData(self._check_seq, _ORDER_ROLE)
                finally:
                    self._check_sync = False
        elif item.checkState() == Qt.Unchecked:
            if item.data(_ORDER_ROLE) is not None:
                self._check_sync = True
                try:
                    item.setData(None, _ORDER_ROLE)
                finally:
                    self._check_sync = False

        # Update parent tristate.
        parent = item.parent()
        if parent is None or not parent.isCheckable():
            return

        checked = 0
        unchecked = 0
        for r in range(parent.rowCount()):
            child = parent.child(r, 0)
            if child is None or not child.isCheckable():
                continue
            if child.checkState() == Qt.Checked:
                checked += 1
            elif child.checkState() == Qt.Unchecked:
                unchecked += 1

        self._check_sync = True
        try:
            if checked and unchecked:
                parent.setCheckState(Qt.PartiallyChecked)
            elif checked and not unchecked:
                parent.setCheckState(Qt.Checked)
            else:
                parent.setCheckState(Qt.Unchecked)
        finally:
            self._check_sync = False

        self._refresh_dependency_status()

    def _current_game_token(self) -> str:
        return "bg2ee" if self._tabs.currentIndex() == 1 else "bgee"

    def _sync_details_from_current(self, proxy_index: QModelIndex) -> None:
        bundle = self._active_tree()
        if not proxy_index.isValid():
            self._details_text.setPlainText("")
            return

        src_index = bundle.proxy.mapToSource(proxy_index)
        item = bundle.model.itemFromIndex(src_index)
        if item is None:
            self._details_text.setPlainText("")
            return

        details = item.data(_DETAILS_ROLE)
        if not isinstance(details, ComponentDetails):
            self._details_text.setPlainText("")
            return

        lines: list[str] = []
        lines.append(f"Mod Name: {details.mod_rel}")
        lines.append(f"Component Name: {details.component_name}")
        lines.append(f"Component ID: {details.component_id}")
        lines.append(f"TP2 Path: {details.tp2_abs}")
        if details.version:
            lines.append(f"Version: {details.version}")
        if details.designated is not None:
            lines.append(f"Designated: {details.designated}")
        if details.game_allowed:
            lines.append(f"Game Allowed: {details.game_allowed}")
        lines.append("")
        lines.append("Dependencies (Requires):")
        if details.dependencies:
            for d in details.dependencies:
                lines.append(f"  • {d}")
        else:
            lines.append("  • None")
        lines.append("")
        lines.append("Conflicts:")
        if details.conflicts:
            for c in details.conflicts:
                lines.append(f"  • {c}")
        else:
            lines.append("  • None")

        self._details_text.setPlainText("\n".join(lines))

    def _current_details(self) -> ComponentDetails | None:
        bundle = self._active_tree()
        idx = bundle.view.currentIndex()
        if not idx.isValid():
            return None
        src_index = bundle.proxy.mapToSource(idx)
        item = bundle.model.itemFromIndex(src_index)
        if item is None:
            return None
        details = item.data(_DETAILS_ROLE)
        return details if isinstance(details, ComponentDetails) else None

    def _mod_root_from_details(self, details: ComponentDetails) -> Path | None:
        if self._scan_cfg is not None and self._scan_cfg.mods_dir:
            base = Path(self._scan_cfg.mods_dir)
            try:
                tp2 = base / Path(details.mod_rel)
                if tp2.exists():
                    return tp2.parent
            except OSError:
                pass
        try:
            tp2_abs = Path(details.tp2_abs)
            if tp2_abs.exists():
                return tp2_abs.parent
        except OSError:
            return None
        return None

    def _find_readme(self, mod_root: Path) -> Path | None:
        for d in (mod_root, mod_root.parent):
            if d is None or not d.is_dir():
                continue
            try:
                for p in d.iterdir():
                    if not p.is_file():
                        continue
                    if p.name.casefold() == "readme.txt":
                        return p
            except OSError:
                continue

        def score(p: Path) -> tuple[int, str]:
            name = p.name.casefold()
            ext = p.suffix.casefold()
            pri = 0
            if name.startswith("readme"):
                pri -= 3
            if "readme" in name:
                pri -= 2
            if ext in {".txt", ".md", ".rtf", ".html", ".htm", ".pdf"}:
                pri -= 1
            return (pri, name)

        candidates: list[Path] = []
        for d in (mod_root, mod_root.parent):
            if d is None or not d.is_dir():
                continue
            try:
                for p in d.iterdir():
                    if not p.is_file():
                        continue
                    if "readme" not in p.name.casefold():
                        continue
                    candidates.append(p)
            except OSError:
                continue

        if not candidates:
            return None
        candidates.sort(key=score)
        return candidates[0]

    def _on_readme_clicked(self) -> None:
        details = self._current_details()
        if details is None:
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText("Select a component to open its readme.")
            return
        mod_root = self._mod_root_from_details(details)
        if mod_root is None:
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText("Cannot locate mod folder for readme.")
            return
        readme = self._find_readme(mod_root)
        if readme is None:
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText("No readme found near this mod.")
            return
        if not QDesktopServices.openUrl(QUrl.fromLocalFile(str(readme))):
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText("Failed to open readme.")

    def _on_web_clicked(self) -> None:
        details = self._current_details()
        tp2_name = ""
        if details is not None:
            tp2_name = Path(details.mod_rel).name
        if not tp2_name:
            tp2_name = "Modname.tp2"
        query = quote_plus(tp2_name)
        google = QUrl(f"https://www.google.com/search?q={query}")
        fandom = QUrl(
            f"https://baldursgate.fandom.com/wiki/Special:Search?query={query}"
        )
        if not QDesktopServices.openUrl(google):
            QDesktopServices.openUrl(fandom)

    def _on_scan_clicked(self) -> None:
        if self._scan_cfg is None:
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText("Scan not configured. Set paths in Step 1 first.")
            return
        if self._scan_thread is not None:
            try:
                if self._scan_thread.isRunning():
                    return
            except RuntimeError:
                self._scan_thread = None
                self._scan_worker = None
        self._start_scan(self._scan_cfg)

    def _on_select_all(self) -> None:
        bundle = self._active_tree()
        self._set_all_checks(bundle.model, Qt.Checked)
        self._status.setStyleSheet("color: #bdbdbd;")
        self._status.setText("Selected all components.")

    def _on_deselect_all(self) -> None:
        bundle = self._active_tree()
        self._set_all_checks(bundle.model, Qt.Unchecked)
        self._status.setStyleSheet("color: #bdbdbd;")
        self._status.setText("Deselected all components.")

    def _on_auto_select_deps(self) -> None:
        # Placeholder: later use metadata/graph to auto-check required deps.
        self._status.setStyleSheet("color: #bdbdbd;")
        self._status.setText("Auto-select dependencies: not implemented yet.")

    def _set_all_checks(self, model: QStandardItemModel, state: Qt.CheckState) -> None:
        root = model.invisibleRootItem()
        for i in range(root.rowCount()):
            mod_item = root.child(i, 0)
            if mod_item is None:
                continue
            for r in range(mod_item.rowCount()):
                comp_item = mod_item.child(r, 0)
                if comp_item is not None and comp_item.isCheckable():
                    comp_item.setCheckState(state)
        self._refresh_dependency_status()

    def get_checked_weidulog_lines(self) -> tuple[list[str], list[str]]:
        return (
            self._checked_lines_for(self._tab_main.model),
            self._checked_lines_for(self._tab_bgee.model),
        )

    def _checked_lines_for(self, model: QStandardItemModel) -> list[str]:
        picked: list[tuple[int, str]] = []
        root = model.invisibleRootItem()
        for i in range(root.rowCount()):
            mod_item = root.child(i, 0)
            if mod_item is None:
                continue
            for r in range(mod_item.rowCount()):
                comp_item = mod_item.child(r, 0)
                if comp_item is None or not comp_item.isCheckable():
                    continue
                if comp_item.checkState() != Qt.Checked:
                    continue
                details = comp_item.data(_DETAILS_ROLE)
                if not isinstance(details, ComponentDetails):
                    continue
                tp2 = details.mod_rel.replace("/", "\\")
                ver = f": {details.version}" if details.version else ""
                line = rf"~{tp2}~ #0 #{details.component_id} // {details.component_name}{ver}"

                order = comp_item.data(_ORDER_ROLE)
                try:
                    order_i = int(order) if order is not None else 1_000_000_000
                except Exception:
                    order_i = 1_000_000_000
                picked.append((order_i, line))
        picked.sort(key=lambda t: t[0])
        return [line for _ord, line in picked]

    def _start_scan(self, cfg: _ScanConfig) -> None:
        self._btn_scan.setEnabled(False)
        self._status.setStyleSheet("color: #d7ba7d;")
        self._status.setText("Scanning...")

        self._scan_thread = QThread(self)
        self._scan_worker = _ScanWorker(cfg)
        self._scan_worker.moveToThread(self._scan_thread)

        self._scan_thread.started.connect(self._scan_worker.run)
        self._scan_worker.progress.connect(self._on_scan_progress)
        self._scan_worker.finished.connect(self._on_scan_finished)

        self._scan_worker.finished.connect(self._scan_thread.quit)
        self._scan_worker.finished.connect(self._scan_worker.deleteLater)
        self._scan_thread.finished.connect(self._scan_thread.deleteLater)

        self._scan_thread.start()

    def _on_scan_progress(self, text: str) -> None:
        self._status.setStyleSheet("color: #d7ba7d;")
        self._status.setText(text)

    def _on_scan_finished(self, results_obj: object, error: str) -> None:
        self._btn_scan.setEnabled(True)
        self._scan_thread = None
        self._scan_worker = None

        if error:
            self._status.setStyleSheet("color: #f48771;")
            self._status.setText(f"Scan failed: {error}")
            return

        results_map = results_obj if isinstance(results_obj, dict) else {}
        bgee = results_map.get("bgee", [])
        bg2 = results_map.get("bg2ee", [])
        errors = results_map.get("_errors", 0)

        if not isinstance(bgee, list):
            bgee = []
        if not isinstance(bg2, list):
            bg2 = []
        if not isinstance(errors, int):
            errors = 0

        self._apply_scan_results(bgee, bg2)

        mode = int(self._scan_cfg.mode) if self._scan_cfg is not None else 2
        shown = (
            len(bgee)
            if mode == 0
            else (len(bg2) if mode == 1 else max(len(bgee), len(bg2)))
        )

        self._status.setStyleSheet("color: #bdbdbd;")
        if errors:
            self._status.setText(
                f"Scan complete. Mods found: {shown} (errors: {errors})"
            )
        else:
            self._status.setText(f"Scan complete. Mods found: {shown}")

    def _apply_scan_results(
        self, bgee: list[_ScannedMod], bg2ee: list[_ScannedMod]
    ) -> None:
        self._check_seq = 0
        self._check_sync = True
        try:
            self._tab_main.model.clear()
            self._tab_main.model.setHorizontalHeaderLabels(
                ["Mod / Component", "ID", "Status"]
            )
            self._populate_model(self._tab_main.model, bgee)
            self._sort_top_level_mods(self._tab_main.model)
            self._tab_main.view.collapseAll()
            self._fit_columns(self._tab_main.view)

            self._tab_bgee.model.clear()
            self._tab_bgee.model.setHorizontalHeaderLabels(
                ["Mod / Component", "ID", "Status"]
            )
            self._populate_model(self._tab_bgee.model, bg2ee)
            self._sort_top_level_mods(self._tab_bgee.model)
            self._tab_bgee.view.collapseAll()
            self._fit_columns(self._tab_bgee.view)
        finally:
            self._check_sync = False

        self._sync_details_from_current(self._active_tree().view.currentIndex())
        self._refresh_dependency_status()

    def _populate_model(
        self, model: QStandardItemModel, results: list[_ScannedMod]
    ) -> None:
        mod_map: dict[str, QStandardItem] = {}
        for mod in results:
            mod_key, mod_label = self._eet_group_key_label(mod.tp2_rel)

            mod_item = mod_map.get(mod_key)
            if mod_item is None:
                mod_item = QStandardItem(mod_label)
                f = mod_item.font()
                f.setBold(True)
                f.setPointSizeF(f.pointSizeF() + 1.5)
                mod_item.setFont(f)
                mod_item.setCheckable(True)
                mod_item.setAutoTristate(True)
                mod_item.setCheckState(Qt.Unchecked)
                mod_item.setEditable(False)

                id_item = QStandardItem("")
                id_item.setEditable(False)
                status_item = QStandardItem("")
                status_item.setEditable(False)

                model.appendRow([mod_item, id_item, status_item])
                mod_map[mod_key] = mod_item

            for cid, name, ver, deps, allowed, conflicts in mod.components:
                comp_item = QStandardItem(name)
                comp_item.setCheckable(True)
                comp_item.setCheckState(Qt.Unchecked)
                comp_item.setEditable(False)
                comp_item.setData("ok", _ISSUE_ROLE)

                cid_item = QStandardItem(str(cid))
                cid_item.setTextAlignment(Qt.AlignRight | Qt.AlignVCenter)
                cid_item.setEditable(False)

                status_item = QStandardItem("")
                status_item.setEditable(False)

                details = ComponentDetails(
                    mod_rel=mod.tp2_rel.replace("\\", "/"),
                    tp2_abs=mod.tp2_abs,
                    component_name=name,
                    component_id=cid,
                    version=ver,
                    game_allowed=self._format_allowed_games(allowed),
                    allowed_games=allowed,
                    dependencies=deps,
                    conflicts=conflicts,
                )
                comp_item.setData(details, _DETAILS_ROLE)

                mod_item.appendRow([comp_item, cid_item, status_item])

    def _eet_group_key_label(self, tp2_rel: str) -> tuple[str, str]:
        # Only group EET_end and EET_gui under EET (and EET itself).
        rel = tp2_rel.replace("\\", "/").lower()
        if (
            rel.endswith("eet/eet.tp2")
            or rel.endswith("eet_end/eet_end.tp2")
            or rel.endswith("eet_gui/eet_gui.tp2")
        ):
            return ("EET", "EET")
        stem = Path(tp2_rel).stem
        parent = Path(tp2_rel).parent.name
        label = stem if not parent else f"{stem} ({parent})"
        return (tp2_rel.lower(), label)

    def _sort_top_level_mods(self, model: QStandardItemModel) -> None:
        root = model.invisibleRootItem()
        rows: list[list[QStandardItem]] = []
        while root.rowCount() > 0:
            rows.append(root.takeRow(0))

        def key(row: list[QStandardItem]) -> str:
            if not row:
                return ""
            return (row[0].text() or "").casefold()

        rows.sort(key=key)
        for row in rows:
            root.appendRow([item for item in row if item is not None])

    def _refresh_dependency_status(self) -> None:
        if self._check_sync:
            return
        missing_color = QBrush(QColor("#d18616"))
        conflict_color = QBrush(QColor("#f48771"))
        self._check_sync = True
        try:
            checked: set[tuple[str, int]] = set()
            for model in (self._tab_main.model, self._tab_bgee.model):
                blocker = QSignalBlocker(model)
                _ = blocker
                root = model.invisibleRootItem()
                for i in range(root.rowCount()):
                    mod_item = root.child(i, 0)
                    if mod_item is None:
                        continue
                    for r in range(mod_item.rowCount()):
                        comp_item = mod_item.child(r, 0)
                        if comp_item is None or not comp_item.isCheckable():
                            continue
                        if comp_item.checkState() != Qt.Checked:
                            continue
                        details = comp_item.data(_DETAILS_ROLE)
                        if not isinstance(details, ComponentDetails):
                            continue
                        key = (
                            self._norm_tp2(details.mod_rel),
                            int(details.component_id),
                        )
                        checked.add(key)

            for bundle in (self._tab_main, self._tab_bgee):
                model = bundle.model
                view = bundle.view
                blocker = QSignalBlocker(model)
                _ = blocker
                selected_src = set()
                sel = view.selectionModel()
                if sel is not None:
                    for idx in sel.selectedRows(0):
                        try:
                            selected_src.add(bundle.proxy.mapToSource(idx))
                        except Exception:
                            continue

                missing_count = 0
                conflict_count = 0
                root = model.invisibleRootItem()
                for i in range(root.rowCount()):
                    mod_item = root.child(i, 0)
                    if mod_item is None:
                        continue
                    for r in range(mod_item.rowCount()):
                        comp_item = mod_item.child(r, 0)
                        if comp_item is None or not comp_item.isCheckable():
                            continue
                        src_idx = model.indexFromItem(comp_item)
                        is_selected = src_idx in selected_src
                        if comp_item.checkState() != Qt.Checked and not is_selected:
                            comp_item.setBackground(QBrush())
                            id_item = mod_item.child(r, 1)
                            status_item = mod_item.child(r, 2)
                            if id_item is not None:
                                id_item.setBackground(QBrush())
                            if status_item is not None:
                                status_item.setBackground(QBrush())
                                status_item.setText("")
                            comp_item.setData("ok", _ISSUE_ROLE)
                            continue
                        details = comp_item.data(_DETAILS_ROLE)
                        if not isinstance(details, ComponentDetails):
                            comp_item.setBackground(QBrush())
                            continue
                        missing = False
                        conflict = False
                        for dep in details.dependencies:
                            dep_key = self._parse_dep_key(dep)
                            if dep_key is not None:
                                if dep_key not in checked:
                                    missing = True
                                    break
                                continue
                            if dep.startswith("file:"):
                                path = dep[5:].strip()
                                if path and not self._game_file_exists(bundle, path):
                                    missing = True
                                    break
                                continue
                            if dep.startswith("res:"):
                                res = dep[4:].strip()
                                if res and not self._resource_exists(bundle, res):
                                    missing = True
                                    break
                                continue
                            if dep.startswith("prog:"):
                                prog = dep[5:].strip()
                                if prog and not self._prog_exists(prog):
                                    missing = True
                                    break
                                continue
                        for con in details.conflicts:
                            con_key = self._parse_dep_key(con)
                            if con_key is None:
                                continue
                            if con_key in checked:
                                conflict = True
                                break
                        id_item = mod_item.child(r, 1)
                        status_item = mod_item.child(r, 2)
                        if conflict:
                            conflict_count += 1
                            comp_item.setBackground(conflict_color)
                            if id_item is not None:
                                id_item.setBackground(conflict_color)
                            if status_item is not None:
                                status_item.setBackground(conflict_color)
                                status_item.setText("Conflict")
                            comp_item.setData("conflict", _ISSUE_ROLE)
                        elif missing:
                            missing_count += 1
                            comp_item.setBackground(missing_color)
                            if id_item is not None:
                                id_item.setBackground(missing_color)
                            if status_item is not None:
                                status_item.setBackground(missing_color)
                                status_item.setText("Missing")
                            comp_item.setData("missing", _ISSUE_ROLE)
                        else:
                            comp_item.setBackground(QBrush())
                            if id_item is not None:
                                id_item.setBackground(QBrush())
                            if status_item is not None:
                                status_item.setBackground(QBrush())
                                status_item.setText("")
                            comp_item.setData("ok", _ISSUE_ROLE)
                key = "bg2ee" if bundle is self._tab_bgee else "bgee"
                self._issue_counts[key] = (missing_count, conflict_count)
        finally:
            self._check_sync = False
        self._update_issue_summary()

    def _apply_game_compatibility(self) -> None:
        disabled_brush = QBrush(QColor("#6f6f6f"))

        self._check_sync = True
        try:
            bundles = (
                (self._tab_main, "bgee"),
                (self._tab_bgee, "bg2ee"),
            )
            for bundle, game_token in bundles:
                model = bundle.model
                blocker = QSignalBlocker(model)
                _ = blocker
                root = model.invisibleRootItem()
                for i in range(root.rowCount()):
                    mod_item = root.child(i, 0)
                    if mod_item is None:
                        continue
                    any_enabled = False
                    for r in range(mod_item.rowCount()):
                        comp_item = mod_item.child(r, 0)
                        id_item = mod_item.child(r, 1)
                        if comp_item is None or not comp_item.isCheckable():
                            continue
                        details = comp_item.data(_DETAILS_ROLE)
                        if not isinstance(details, ComponentDetails):
                            continue

                        allowed = details.allowed_games
                        ok = self._is_game_allowed(allowed, game_token)
                        if not ok:
                            if comp_item.checkState() == Qt.Checked:
                                comp_item.setCheckState(Qt.Unchecked)
                            comp_item.setFlags(
                                comp_item.flags() & ~Qt.ItemIsUserCheckable
                            )
                            comp_item.setForeground(disabled_brush)
                            if id_item is not None:
                                id_item.setForeground(disabled_brush)
                        else:
                            comp_item.setFlags(
                                comp_item.flags() | Qt.ItemIsUserCheckable
                            )
                            comp_item.setForeground(QBrush())
                            comp_item.setData(None, Qt.ForegroundRole)
                            any_enabled = True
                            if id_item is not None:
                                id_item.setForeground(QBrush())
                                id_item.setData(None, Qt.ForegroundRole)

                    if any_enabled:
                        mod_item.setForeground(QBrush())
                        mod_item.setData(None, Qt.ForegroundRole)
                    else:
                        mod_item.setForeground(disabled_brush)
                        id_top = model.item(mod_item.row(), 1)
                        if id_top is not None:
                            id_top.setForeground(disabled_brush)
        finally:
            self._check_sync = False

    def _is_game_allowed(self, allowed: tuple[str, ...], game_token: str) -> bool:
        if not allowed:
            return True
        if game_token in allowed:
            return True
        # EET-compatible components should be allowed on both BGEE and BG2EE tabs.
        if "eet" in allowed and game_token in {"bgee", "bg2ee"}:
            return True
        return False

    def _format_allowed_games(self, allowed: tuple[str, ...]) -> str | None:
        if not allowed:
            return None
        label_map = {"bgee": "BGEE", "bg2ee": "BG2EE", "eet": "EET", "iwdee": "IWD:EE"}
        labels = [label_map.get(a, a.upper()) for a in allowed]
        return " / ".join(labels)

    def _game_token_matches(self, game_token: str, raw_game: str) -> bool:
        t = raw_game.strip().lower().strip("~").strip('"').strip("'")
        if t in {"bgee", "bg1ee", "bg1"}:
            return game_token == "bgee"
        if t in {"bg2ee", "bg2"}:
            return game_token == "bg2ee"
        if t in {"eet"}:
            return game_token in {"bgee", "bg2ee"}
        if t in {"iwdee", "iwd-ee", "iwd_ee"}:
            return False
        return False

    def _game_file_exists(self, bundle: "_TreeBundle", rel_path: str) -> bool:
        if self._scan_cfg is None:
            return False
        base = (
            self._scan_cfg.bgee_dir
            if bundle is self._tab_main
            else self._scan_cfg.bg2ee_dir
        )
        if not base:
            return False
        p = Path(base) / rel_path
        try:
            return p.exists()
        except OSError:
            return False

    def _resource_exists(self, bundle: "_TreeBundle", resref: str) -> bool:
        if self._scan_cfg is None:
            return False
        base = (
            self._scan_cfg.bgee_dir
            if bundle is self._tab_main
            else self._scan_cfg.bg2ee_dir
        )
        if not base:
            return False
        override = Path(base) / "override"
        try:
            if not override.is_dir():
                return False
            rx = re.compile(rf"^{re.escape(resref)}\\.[A-Za-z0-9]+$", re.I)
            for p in override.iterdir():
                if p.is_file() and rx.match(p.name):
                    return True
        except OSError:
            return False
        return False

    def _prog_exists(self, prog: str) -> bool:
        from shutil import which

        if which(prog):
            return True
        if self._scan_cfg and self._scan_cfg.weidu_exe:
            try:
                p = Path(self._scan_cfg.weidu_exe).parent / prog
                return p.exists()
            except OSError:
                return False
        return False

    def _norm_tp2(self, tp2: str) -> str:
        return tp2.replace("\\", "/").strip().lower()

    def _parse_dep_key(self, dep: str) -> tuple[str, int] | None:
        s = dep.strip()
        m = re.match(r"^(?P<tp2>[^#]+)#(?P<cid>\d+)$", s)
        if not m:
            return None
        tp2 = self._norm_tp2(m.group("tp2"))
        try:
            cid = int(m.group("cid"))
        except ValueError:
            return None
        return (tp2, cid)

    def _fit_columns(self, view: QTreeView) -> None:
        view.resizeColumnToContents(1)
        view.resizeColumnToContents(2)
        header = view.header()
        id_w = header.sectionSize(1)
        total = view.viewport().width()
        margin = 12
        status_w = header.sectionSize(2)
        col0 = max(200, total - id_w - status_w - margin)
        view.setColumnWidth(0, col0)

    def _update_issue_summary(self) -> None:
        key = "bg2ee" if self._tabs.currentIndex() == 1 else "bgee"
        missing, conflict = self._issue_counts.get(key, (0, 0))
        self._issue_summary.setText(f"Issues: Missing {missing} / Conflicts {conflict}")

    def _load_placeholder_data(self, model: QStandardItemModel) -> None:
        def add_mod(name: str) -> QStandardItem:
            mod_item = QStandardItem(name)
            mod_item.setCheckable(False)
            mod_item.setEditable(False)
            mod_item.setFont(mod_item.font())  # keep default; theme controls the look

            id_item = QStandardItem("")
            id_item.setEditable(False)

            model.appendRow([mod_item, id_item])
            return mod_item

        def add_comp(mod_item: QStandardItem, label: str, cid: int) -> None:
            comp_item = QStandardItem(label)
            comp_item.setCheckable(True)
            comp_item.setCheckState(Qt.Unchecked)
            comp_item.setEditable(False)

            id_item = QStandardItem(str(cid))
            id_item.setTextAlignment(Qt.AlignRight | Qt.AlignVCenter)
            id_item.setEditable(False)

            details = ComponentDetails(
                mod_rel="stratagems/setup-stratagems.tp2",
                tp2_abs=r"D:\Mods\Stratagems\setup-stratagems.tp2",
                component_name=label,
                component_id=cid,
                version="35.21",
                designated=cid,
                game_allowed="BG2EE / EET",
                dependencies=("dw_som/dw_som.tp2 #1600",),
                conflicts=(),
            )
            comp_item.setData(details, _DETAILS_ROLE)

            mod_item.appendRow([comp_item, id_item])

        m1 = add_mod("STRATAGEMS")
        add_comp(
            m1,
            "Install in batch mode (ask about all components before starting installation).",
            100,
        )
        add_comp(m1, "Include arcane spells from Icewind Dale: Enhanced Edition.", 1500)
        add_comp(m1, "Use Baldur's Gate-style Insect Plague and Creeping Doom.", 1600)

        m2 = add_mod("SOUTHERNEDGE")
        add_comp(m2, "Core component.", 0)


class _ScanWorker(QObject):
    progress = Signal(str)
    finished = Signal(object, str)  # (results: list[_ScannedMod], error)

    def __init__(self, cfg: _ScanConfig) -> None:
        super().__init__()
        self._cfg = cfg

    def run(self) -> None:
        try:
            results = self._scan()
        except Exception as e:
            self.finished.emit([], str(e))
            return
        self.finished.emit(results, "")

    def _scan(self) -> dict[str, list[_ScannedMod]]:
        cfg = self._cfg
        mods_dir = Path(cfg.mods_dir)
        weidu_exe = Path(cfg.weidu_exe)

        bgee_dir = Path(cfg.bgee_dir) if cfg.bgee_dir else None
        bg2ee_dir = Path(cfg.bg2ee_dir) if cfg.bg2ee_dir else None

        if not mods_dir.is_dir():
            raise RuntimeError(f"Mods folder not found: {mods_dir}")
        if not weidu_exe.is_file():
            raise RuntimeError(f"WeiDU exe not found: {weidu_exe}")

        if cfg.mode == 0 and (bgee_dir is None or not bgee_dir.is_dir()):
            raise RuntimeError("BGEE game folder not set/found.")
        if cfg.mode == 1 and (bg2ee_dir is None or not bg2ee_dir.is_dir()):
            raise RuntimeError("BG2EE game folder not set/found.")
        if cfg.mode == 2:
            if bgee_dir is None or not bgee_dir.is_dir():
                raise RuntimeError("BGEE game folder not set/found (EET).")
            if bg2ee_dir is None or not bg2ee_dir.is_dir():
                raise RuntimeError("BG2EE game folder not set/found (EET).")

        tp2s = self._iter_tp2_files(mods_dir, max_depth=3)

        targets: list[tuple[str, Path]] = []
        if cfg.mode in (0, 2) and bgee_dir is not None:
            targets.append(("bgee", bgee_dir))
        if cfg.mode in (1, 2) and bg2ee_dir is not None:
            targets.append(("bg2ee", bg2ee_dir))

        out: dict[str, list[_ScannedMod]] = {"bgee": [], "bg2ee": []}

        total = len(tp2s) * max(1, len(targets))
        done = 0

        max_workers = min(4, os.cpu_count() or 1)

        for key, game_dir in targets:
            use_lang = self._detect_game_use_lang(game_dir)

            def scan_one(tp2: Path) -> _ScannedMod:
                pref_lang = self._detect_mod_language_index(tp2)
                lang_count = self._count_mod_languages(tp2)

                if self._tp2_has_no_components(tp2):
                    return _ScannedMod(tp2_rel="", tp2_abs=str(tp2), components=tuple())

                best_idx = int(pref_lang) if pref_lang is not None else 0
                try:
                    text = self._run_list_components(
                        weidu_exe, game_dir, use_lang, best_idx, tp2
                    )
                    comps = self._parse_list_components(text)
                except Exception:
                    comps = []
                if not comps:
                    comps = self._fallback_components_from_tp2(tp2, pref_lang)
                best_undef = self._count_undefined(comps)

                if lang_count > 1 and best_undef:
                    for idx in range(min(lang_count, 6)):
                        if idx == best_idx:
                            continue
                        t2 = self._run_list_components(
                            weidu_exe, game_dir, use_lang, idx, tp2
                        )
                        c2 = self._parse_list_components(t2)
                        u2 = self._count_undefined(c2)
                        if u2 < best_undef or (
                            u2 == best_undef and len(c2) > len(comps)
                        ):
                            best_idx = idx
                            comps = c2
                            best_undef = u2

                use_index = self._looks_like_index_ids([cid for cid, _n, _v in comps])
                deps_map = self._extract_component_deps(
                    tp2, use_index=use_index, game_token=key
                )
                conflict_map = self._extract_component_conflicts(
                    tp2, use_index=use_index, game_token=key
                )
                allowed_map = self._extract_component_allowed_games(
                    tp2, use_index=use_index
                )
                label_map = self._extract_component_labels(tp2, use_index=use_index)

                rel = self._normalize_tp2_rel(mods_dir, tp2)

                def fix_name(cid: int, name: str) -> str:
                    base = (name or "").strip()
                    if base.upper().startswith("UNDEFINED STRING"):
                        resolved = self._resolve_undefined_name(tp2, pref_lang, base)
                        if resolved:
                            return resolved
                    if not base or base == ":" or base.startswith(":"):
                        return label_map.get(cid, f"Component {cid}")
                    return name

                with_deps = [
                    (
                        cid,
                        fix_name(cid, name),
                        ver,
                        tuple(deps_map.get(cid, ())),
                        tuple(allowed_map.get(cid, ())),
                        tuple(conflict_map.get(cid, ())),
                    )
                    for cid, name, ver in comps
                ]
                return _ScannedMod(
                    tp2_rel=rel, tp2_abs=str(tp2), components=tuple(with_deps)
                )

            results: list[_ScannedMod] = []
            with ThreadPoolExecutor(max_workers=max_workers) as ex:
                futs = {ex.submit(scan_one, tp2): tp2 for tp2 in tp2s}
                error_count = 0
                for fut in as_completed(futs):
                    tp2 = futs[fut]
                    done += 1
                    self.progress.emit(f"Scanning {done}/{total}: {tp2.name}")
                    try:
                        res = fut.result()
                        if res.components:
                            results.append(res)
                    except Exception:
                        error_count += 1

            results.sort(key=lambda m: m.tp2_rel.casefold())
            out[key] = results

            if error_count:
                out["_errors"] = out.get("_errors", 0) + error_count

        return out

    def _resolve_undefined_name(
        self, tp2: Path, lang_idx: int | None, text: str
    ) -> str | None:
        m = re.search(r"@\s*(\d+)", text)
        if not m:
            return None
        try:
            strref = int(m.group(1))
        except ValueError:
            return None
        tra_path = self._pick_setup_tra(tp2, lang_idx)
        if tra_path is not None:
            val = self._read_tra_string(tra_path, strref)
            if val:
                return val

        # Fallback: try any referenced .tra paths in the TP2.
        for rel in self._extract_tra_paths(tp2):
            for base in (tp2.parent, tp2.parent.parent, tp2.parent.parent.parent):
                try:
                    p = (base / rel).resolve()
                    if not p.is_file():
                        continue
                except OSError:
                    continue
                val = self._read_tra_string(p, strref)
                if val:
                    return val

        # Last resort: scan for setup.tra files under the mod directory.
        for p in self._find_setup_tra_candidates(tp2):
            val = self._read_tra_string(p, strref)
            if val:
                return val
        # Fallback: use inline BEGIN comment if present (e.g., "BEGIN @0 /* Name */").
        return self._find_begin_comment(tp2, strref)

    def _pick_setup_tra(self, tp2: Path, lang_idx: int | None) -> Path | None:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        text = re.sub(r"/\*.*?\*/", "", raw, flags=re.S)
        text = re.sub(r"//.*", "", text)
        lines = text.splitlines()
        rx_lang = re.compile(r"^\s*LANGUAGE\b(.*)$", flags=re.I)
        rx_q = re.compile(r"~([^~]+)~|\"([^\"]+)\"|'([^']+)'")

        entries: list[list[str]] = []
        i = 0
        while i < len(lines):
            m = rx_lang.match(lines[i])
            if not m:
                i += 1
                continue
            tail = (m.group(1) or "").strip()
            flat: list[str] = []
            if tail:
                for a, b, c in rx_q.findall(tail):
                    val = a or b or c
                    if val:
                        flat.append(val)
            j = i + 1
            while j < len(lines) and len(flat) < 4:
                line = lines[j].strip()
                if line:
                    for a, b, c in rx_q.findall(line):
                        val = a or b or c
                        if val:
                            flat.append(val)
                j += 1
            entries.append(flat)
            i = j

        if not entries:
            return None

        idx = lang_idx if isinstance(lang_idx, int) else 0
        if idx < 0 or idx >= len(entries):
            idx = 0
        paths = entries[idx][2:] if len(entries[idx]) >= 3 else []
        setup_candidates = [p for p in paths if p.lower().endswith("setup.tra")]
        if not setup_candidates:
            setup_candidates = paths
        for rel in setup_candidates:
            rel = self._expand_mod_folder_ref(rel, tp2)
            rel = rel.replace("\\", "/")
            for base in (tp2.parent, tp2.parent.parent, tp2.parent.parent.parent):
                try:
                    p = (base / rel).resolve()
                    if p.is_file():
                        return p
                except OSError:
                    continue
        return None

    def _expand_mod_folder_ref(self, rel: str, tp2: Path) -> str:
        # Expand %MOD_FOLDER% -> folder containing the TP2 (its name).
        return re.sub(r"%\\s*MOD_FOLDER\\s*%", tp2.parent.name, rel, flags=re.I)

    def _find_setup_tra_candidates(self, tp2: Path) -> list[Path]:
        base = tp2.parent
        max_depth = 3
        base_parts = len(base.resolve().parts)
        found: list[Path] = []

        for root, dirs, files in os.walk(base):
            root_p = Path(root).resolve()
            depth = len(root_p.parts) - base_parts
            if depth > max_depth:
                dirs[:] = []
                continue

            for fn in files:
                if fn.lower() != "setup.tra":
                    continue
                try:
                    p = (Path(root) / fn).resolve()
                except OSError:
                    continue
                found.append(p)

        return found

    def _find_begin_comment(self, tp2: Path, strref: int) -> str | None:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        rx = re.compile(rf"^\\s*BEGIN\\s+@{strref}\\b(.*)$", flags=re.I)
        for line in raw.splitlines():
            m = rx.match(line)
            if not m:
                continue
            tail = (m.group(1) or "").strip()
            if not tail:
                return None
            m_block = re.search(r"/\\*\\s*(.*?)\\s*\\*/", tail)
            if m_block:
                val = (m_block.group(1) or "").strip()
                if val:
                    return val
            m_line = re.search(r"//\\s*(.*)$", tail)
            if m_line:
                val = (m_line.group(1) or "").strip()
                if val:
                    return val
            return None
        return None

    def _read_tra_string(self, tra_path: Path, strref: int) -> str | None:
        try:
            raw = tra_path.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tra_path.read_text(errors="replace")
        rx = re.compile(
            rf"^\s*@{strref}\s*=\s*(~([^~]*)~|\"([^\"]*)\"|'([^']*)')", flags=re.M
        )
        m = rx.search(raw)
        if not m:
            return None
        val = m.group(2) or m.group(3) or m.group(4) or ""
        return val.strip() or None

    def _extract_component_labels(
        self, tp2: Path, *, use_index: bool
    ) -> dict[int, str]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        labels: dict[int, str] = {}
        blocks = re.split(r"^\s*BEGIN\b", raw, flags=re.M)
        for idx, block in enumerate(blocks[1:], start=0):
            if use_index:
                cid = idx
            else:
                m = re.search(r"\bDESIGNATED\s+(\d+)", block, flags=re.I)
                if not m:
                    m = re.search(r"\bDESIGNATED\s*=\s*(\d+)", block, flags=re.I)
                if not m:
                    continue
                try:
                    cid = int(m.group(1))
                except ValueError:
                    continue

            label = ""
            for line in block.splitlines():
                m = re.search(r"\bLABEL\b\s+(.+)", line, flags=re.I)
                if not m:
                    continue
                label = m.group(1)
                label = label.split("//", 1)[0].strip()
                label = label.strip().strip("~").strip('"').strip("'")
                break

            if label:
                labels[cid] = label

        return labels

    def _tp2_has_no_components(self, tp2: Path) -> bool:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")
        return re.search(r"^\s*BEGIN\b", raw, flags=re.M) is None

    def _iter_tp2_files(self, mods_dir: Path, max_depth: int) -> list[Path]:
        out: list[Path] = []
        base_parts = len(mods_dir.resolve().parts)

        for root, dirs, files in os.walk(mods_dir):
            root_p = Path(root).resolve()
            depth = len(root_p.parts) - base_parts
            if depth > max_depth:
                dirs[:] = []
                continue

            for fn in files:
                if not fn.lower().endswith(".tp2"):
                    continue
                p = (Path(root) / fn).resolve()
                out.append(p)

        out.sort(key=lambda p: str(p).lower())
        return out

    def _normalize_tp2_rel(self, mods_dir: Path, tp2: Path) -> str:
        rel = tp2.relative_to(mods_dir)
        parts = rel.parts
        parent_name = tp2.parent.name.casefold()

        cut = 0
        for i in range(0, len(parts) - 1):
            if parts[i].casefold() == parent_name:
                cut = i

        if cut > 0:
            rel = Path(*parts[cut:])

        return rel.as_posix()

    def _extract_component_deps(
        self, tp2: Path, *, use_index: bool, game_token: str
    ) -> dict[int, list[str]]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        deps: dict[int, list[str]] = {}
        blocks = re.split(r"^\s*BEGIN\b", raw, flags=re.M)
        for idx, block in enumerate(blocks[1:], start=0):
            if use_index:
                cid = idx
            else:
                m = re.search(r"\bDESIGNATED\s+(\d+)", block, flags=re.I)
                if not m:
                    m = re.search(r"\bDESIGNATED\s*=\s*(\d+)", block, flags=re.I)
                if not m:
                    continue
                try:
                    cid = int(m.group(1))
                except ValueError:
                    continue

            dep_list: list[str] = []
            for rx in (
                r"REQUIRE_COMPONENT\s+~([^~]+)~\s+~?(\d+)~?",
                r'REQUIRE_COMPONENT\s+"([^"]+)"\s+"?(\d+)"?',
                r"REQUIRE_COMPONENT\s+([^\s]+)\s+(\d+)",
                r"REQUIRE_COMPONENT\s+~([^~]+)~\s+DESIGNATED\s+(\d+)",
                r'REQUIRE_COMPONENT\s+"([^"]+)"\s+DESIGNATED\s+(\d+)',
                r"REQUIRE_COMPONENT\s+([^\s]+)\s+DESIGNATED\s+(\d+)",
                r"REQUIRE_COMPONENT_IN_GAME\s+~([^~]+)~\s+~?(\d+)~?\s+([^\s]+)",
                r'REQUIRE_COMPONENT_IN_GAME\s+"([^"]+)"\s+"?(\d+)"?\s+([^\s]+)',
                r"REQUIRE_COMPONENT_IN_GAME\s+([^\s]+)\s+(\d+)\s+([^\s]+)",
                r"COMPONENT_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'COMPONENT_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"COMPONENT_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
                r"MOD_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'MOD_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"MOD_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
                r"REQUIRE_FILE\s+~([^~]+)~",
                r'REQUIRE_FILE\s+"([^"]+)"',
                r"REQUIRE_RESOURCE\s+~([^~]+)~",
                r'REQUIRE_RESOURCE\s+"([^"]+)"',
                r"REQUIRE_PROG\s+~([^~]+)~",
                r'REQUIRE_PROG\s+"([^"]+)"',
            ):
                for dm in re.finditer(rx, block, flags=re.I):
                    if "REQUIRE_COMPONENT_IN_GAME" in rx:
                        dep_tp2 = dm.group(1).strip().replace("\\", "/")
                        dep_tp2 = dep_tp2.strip().strip("~").strip('"').strip("'")
                        dep_id = dm.group(2).strip()
                        game = dm.group(3).strip()
                        if not dep_tp2 or not dep_id:
                            continue
                        if not self._game_token_matches(game_token, game):
                            continue
                        dep_list.append(f"{dep_tp2}#{dep_id}")
                        continue
                    if (
                        "REQUIRE_COMPONENT" in rx
                        or "MOD_IS_INSTALLED" in rx
                        or "COMPONENT_IS_INSTALLED" in rx
                    ):
                        dep_tp2 = dm.group(1).strip().replace("\\", "/")
                        dep_tp2 = dep_tp2.strip().strip("~").strip('"').strip("'")
                        dep_id = dm.group(2).strip()
                        if not dep_tp2 or not dep_id:
                            continue
                        dep_list.append(f"{dep_tp2}#{dep_id}")
                        continue
                    if "REQUIRE_FILE" in rx:
                        path = dm.group(1).strip().strip("~").strip('"').strip("'")
                        if path:
                            dep_list.append(f"file:{path}")
                        continue
                    if "REQUIRE_RESOURCE" in rx:
                        res = dm.group(1).strip().strip("~").strip('"').strip("'")
                        if res:
                            dep_list.append(f"res:{res}")
                        continue
                    if "REQUIRE_PROG" in rx:
                        prog = dm.group(1).strip().strip("~").strip('"').strip("'")
                        if prog:
                            dep_list.append(f"prog:{prog}")
                        continue

            dep_pred, con_pred, file_pred, res_pred, prog_pred = (
                self._parse_predicate_refs(block)
            )
            dep_list.extend(dep_pred)
            dep_list.extend(file_pred)
            dep_list.extend(res_pred)
            dep_list.extend(prog_pred)
            if dep_list:
                deps[cid] = sorted(set(dep_list))

        return deps

    def _extract_component_conflicts(
        self, tp2: Path, *, use_index: bool, game_token: str
    ) -> dict[int, list[str]]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        conflicts: dict[int, list[str]] = {}
        blocks = re.split(r"^\s*BEGIN\b", raw, flags=re.M)
        for idx, block in enumerate(blocks[1:], start=0):
            if use_index:
                cid = idx
            else:
                m = re.search(r"\bDESIGNATED\s+(\d+)", block, flags=re.I)
                if not m:
                    m = re.search(r"\bDESIGNATED\s*=\s*(\d+)", block, flags=re.I)
                if not m:
                    continue
                try:
                    cid = int(m.group(1))
                except ValueError:
                    continue

            con_list: list[str] = []
            for rx in (
                r"FORBID_COMPONENT\s+~([^~]+)~\s+~?(\d+)~?",
                r'FORBID_COMPONENT\s+"([^"]+)"\s+"?(\d+)"?',
                r"FORBID_COMPONENT\s+([^\s]+)\s+(\d+)",
                r"FORBID_COMPONENT\s+~([^~]+)~\s+DESIGNATED\s+(\d+)",
                r'FORBID_COMPONENT\s+"([^"]+)"\s+DESIGNATED\s+(\d+)',
                r"FORBID_COMPONENT\s+([^\s]+)\s+DESIGNATED\s+(\d+)",
                r"FORBID_COMPONENT_IN_GAME\s+~([^~]+)~\s+~?(\d+)~?\s+([^\s]+)",
                r'FORBID_COMPONENT_IN_GAME\s+"([^"]+)"\s+"?(\d+)"?\s+([^\s]+)',
                r"FORBID_COMPONENT_IN_GAME\s+([^\s]+)\s+(\d+)\s+([^\s]+)",
            ):
                for dm in re.finditer(rx, block, flags=re.I):
                    if "FORBID_COMPONENT_IN_GAME" in rx:
                        con_tp2 = dm.group(1).strip().replace("\\", "/")
                        con_tp2 = con_tp2.strip().strip("~").strip('"').strip("'")
                        con_id = dm.group(2).strip()
                        game = dm.group(3).strip()
                        if not con_tp2 or not con_id:
                            continue
                        if not self._game_token_matches(game_token, game):
                            continue
                        con_list.append(f"{con_tp2}#{con_id}")
                        continue
                    con_tp2 = dm.group(1).strip().replace("\\", "/")
                    con_tp2 = con_tp2.strip().strip("~").strip('"').strip("'")
                    con_id = dm.group(2).strip()
                    if not con_tp2 or not con_id:
                        continue
                    con_list.append(f"{con_tp2}#{con_id}")

            dep_pred, con_pred, _file_pred, _res_pred, _prog_pred = (
                self._parse_predicate_refs(block)
            )
            con_list.extend(con_pred)
            if con_list:
                conflicts[cid] = sorted(set(con_list))

        return conflicts

    def _parse_predicate_refs(
        self, text: str
    ) -> tuple[list[str], list[str], list[str], list[str], list[str]]:
        deps: list[str] = []
        conflicts: list[str] = []
        files: list[str] = []
        resources: list[str] = []
        progs: list[str] = []
        for m in re.finditer(r"REQUIRE_PREDICATE\b([\s\S]{0,800})", text, flags=re.I):
            chunk = m.group(1)
            for rx in (
                r"COMPONENT_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'COMPONENT_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"COMPONENT_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
                r"MOD_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'MOD_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"MOD_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    tp2 = dm.group(1).strip().replace("\\", "/")
                    tp2 = tp2.strip().strip("~").strip('"').strip("'")
                    cid = dm.group(2).strip()
                    if not tp2 or not cid:
                        continue
                    deps.append(f"{tp2}#{cid}")
            for rx in (
                r"!\s*MOD_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'!\s*MOD_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"!\s*MOD_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
                r"NOT\s+MOD_IS_INSTALLED\s+~([^~]+)~\s+~?(\d+)~?",
                r'NOT\s+MOD_IS_INSTALLED\s+"([^"]+)"\s+"?(\d+)"?',
                r"NOT\s+MOD_IS_INSTALLED\s+([^\s]+)\s+(\d+)",
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    tp2 = dm.group(1).strip().replace("\\", "/")
                    tp2 = tp2.strip().strip("~").strip('"').strip("'")
                    cid = dm.group(2).strip()
                    if not tp2 or not cid:
                        continue
                    conflicts.append(f"{tp2}#{cid}")
            for rx in (
                r"FILE_EXISTS_IN_GAME\s+~([^~]+)~",
                r'FILE_EXISTS_IN_GAME\s+"([^"]+)"',
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    path = dm.group(1).strip().strip("~").strip('"').strip("'")
                    if path:
                        files.append(f"file:{path}")
            for rx in (
                r"REQUIRE_FILE\s+~([^~]+)~",
                r'REQUIRE_FILE\s+"([^"]+)"',
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    path = dm.group(1).strip().strip("~").strip('"').strip("'")
                    if path:
                        files.append(f"file:{path}")
            for rx in (
                r"REQUIRE_RESOURCE\s+~([^~]+)~",
                r'REQUIRE_RESOURCE\s+"([^"]+)"',
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    res = dm.group(1).strip().strip("~").strip('"').strip("'")
                    if res:
                        resources.append(f"res:{res}")
            for rx in (
                r"REQUIRE_PROG\s+~([^~]+)~",
                r'REQUIRE_PROG\s+"([^"]+)"',
            ):
                for dm in re.finditer(rx, chunk, flags=re.I):
                    prog = dm.group(1).strip().strip("~").strip('"').strip("'")
                    if prog:
                        progs.append(f"prog:{prog}")
        return (deps, conflicts, files, resources, progs)

    def _extract_component_allowed_games(
        self, tp2: Path, *, use_index: bool
    ) -> dict[int, tuple[str, ...]]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        allowed: dict[int, set[str]] = {}
        blocks = re.split(r"^\s*BEGIN\b", raw, flags=re.M)
        for idx, block in enumerate(blocks[1:], start=0):
            cid = idx
            if not use_index:
                m = re.search(r"\bDESIGNATED\s+(\d+)", block, flags=re.I)
                if not m:
                    m = re.search(r"\bDESIGNATED\s*=\s*(\d+)", block, flags=re.I)
                if m:
                    try:
                        cid = int(m.group(1))
                    except ValueError:
                        cid = idx

            games = self._parse_game_tokens(block)
            if not games:
                continue
            allowed.setdefault(cid, set()).update(games)

        return {k: tuple(sorted(v)) for k, v in allowed.items()}

    def _looks_like_index_ids(self, ids: list[int]) -> bool:
        if not ids:
            return False
        uniq = sorted(set(ids))
        if uniq[0] != 0:
            return False
        max_id = uniq[-1]
        if max_id > len(uniq) + 2:
            return False
        missing = 0
        present = set(uniq)
        for i in range(max_id + 1):
            if i not in present:
                missing += 1
        return missing <= max(2, len(uniq) // 10)

    def _parse_game_tokens(self, text: str) -> set[str]:
        # Only treat hard gates as "allowed games" (ignore ACTION_IF GAME_IS).
        out: set[str] = set()

        def add_token(raw: str) -> None:
            for tok in re.split(r"[\s,;/]+", raw.strip()):
                t = tok.strip().lower()
                if not t:
                    continue
                if t in {"bg1", "bg1ee", "bgee"}:
                    out.add("bgee")
                elif t in {"bg2", "bg2ee"}:
                    out.add("bg2ee")
                elif t in {"eet"}:
                    out.add("eet")
                elif t in {"iwdee", "iwd-ee", "iwd_ee"}:
                    out.add("iwdee")

        clean = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
        clean = re.sub(r"//.*", "", clean)

        # REQUIRE_COMPONENT_IN_GAME ... <game>
        for rx in (
            r"REQUIRE_COMPONENT_IN_GAME\s+[^\\n]+?\\s+~([^~]+)~",
            r"REQUIRE_COMPONENT_IN_GAME\s+[^\\n]+?\\s+\"([^\"]+)\"",
            r"REQUIRE_COMPONENT_IN_GAME\s+[^\\n]+?\\s+([A-Za-z0-9_-]+)",
        ):
            for m in re.finditer(rx, clean, flags=re.I):
                add_token(m.group(1) or "")

        # REQUIRE_PREDICATE ... GAME_IS ...
        rx_pred = re.compile(
            r"REQUIRE_PREDICATE[\\s\\S]*?(?:@\\d+|\"[^\"]+\"|~[^~]+~)", flags=re.I
        )
        rx_game = re.compile(r"\bGAME_IS\b\s*(?:~([^~]+)~|\"([^\"]+)\")", flags=re.I)
        for m in rx_pred.finditer(clean):
            chunk = m.group(0)
            for gm in rx_game.finditer(chunk):
                raw = gm.group(1) or gm.group(2) or ""
                add_token(raw)

        return out

    def _detect_game_use_lang(self, game_dir: Path) -> str | None:
        lang_root = game_dir / "lang"
        if not lang_root.is_dir():
            return None
        subs = [p for p in lang_root.iterdir() if p.is_dir()]
        if not subs:
            return None
        for p in subs:
            if p.name.lower() == "en_us":
                return p.name
        subs.sort(key=lambda p: p.name.lower())
        return subs[0].name

    def _detect_mod_language_index(self, tp2: Path) -> int | None:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        # Best-effort comment stripping (avoid false LANGUAGE hits).
        text = re.sub(r"/\*.*?\*/", "", raw, flags=re.S)
        text = re.sub(r"//.*", "", text)

        lines = text.splitlines()
        langs: list[tuple[str, str]] = []
        rx_lang = re.compile(r"^\s*LANGUAGE\b(.*)$", flags=re.I)
        rx_q = re.compile(r'~([^~]+)~|"([^"]+)"|\'([^\']+)\'')

        i = 0
        while i < len(lines):
            m = rx_lang.match(lines[i])
            if not m:
                i += 1
                continue

            tail = (m.group(1) or "").strip()
            quoted: list[str] = []
            if tail:
                quoted = [g for g in rx_q.findall(tail)]
            flat = []
            for a, b, c in quoted:
                val = a or b or c
                if val:
                    flat.append(val)

            j = i + 1
            while len(flat) < 2 and j < len(lines):
                line = lines[j].strip()
                if line:
                    for a, b, c in rx_q.findall(line):
                        val = a or b or c
                        if val:
                            flat.append(val)
                j += 1

            if len(flat) >= 2:
                disp = flat[0].strip()
                token_norm = flat[1].strip()
                token_u = token_norm.upper()
                langs.append((disp, token_u))

            i = j

        if not langs:
            return None

        for idx, (disp, token_u) in enumerate(langs):
            if "ENGLISH" in disp.upper():
                return idx
            if token_u in {
                "ENGLISH",
                "EN_US",
                "EN-GB",
                "EN_GB",
                "EN-UK",
                "EN_UK",
                "EN",
            }:
                return idx
            if token_u.startswith("EN_"):
                return idx

        return 0

    def _count_mod_languages(self, tp2: Path) -> int:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        text = re.sub(r"/\*.*?\*/", "", raw, flags=re.S)
        text = re.sub(r"//.*", "", text)

        count = 0
        for line in text.splitlines():
            if re.match(r"^\s*LANGUAGE\b", line, flags=re.I):
                count += 1
        return max(1, count)

    def _count_undefined(self, comps: list[tuple[int, str, str | None]]) -> int:
        n = 0
        for _cid, name, _ver in comps:
            if name.strip().upper().startswith("UNDEFINED"):
                n += 1
        return n

    def _run_list_components(
        self,
        weidu_exe: Path,
        game_dir: Path,
        use_lang: str | None,
        mod_lang: int | None,
        tp2: Path,
    ) -> str:
        argv = [str(weidu_exe), "--game", str(game_dir)]
        if use_lang:
            argv += ["--use-lang", use_lang]
        lang_idx = int(mod_lang) if mod_lang is not None else 0
        argv += ["--list-components", str(tp2), str(lang_idx)]
        workdir = self._pick_mod_workdir(tp2)
        cp = subprocess.run(
            argv,
            cwd=str(workdir),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=180,
        )
        try:
            return cp.stdout.decode("utf-8", errors="replace")
        except Exception:
            return cp.stdout.decode(errors="replace")

    def _pick_mod_workdir(self, tp2: Path) -> Path:
        # Heuristic: pick a cwd where relative TRA paths resolve.
        tra_paths = self._extract_tra_paths(tp2)
        if not tra_paths:
            return tp2.parent

        candidates = [tp2.parent, tp2.parent.parent, tp2.parent.parent.parent]
        best = tp2.parent
        best_hits = -1

        for c in candidates:
            hits = 0
            for rel in tra_paths:
                try:
                    if (c / rel).is_file():
                        hits += 1
                except OSError:
                    continue
            try:
                if (c / "lang").is_dir():
                    hits += 1
            except OSError:
                pass
            if hits > best_hits:
                best_hits = hits
                best = c

        return best if best_hits > 0 else tp2.parent

    def _extract_tra_paths(self, tp2: Path) -> list[Path]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        found: list[Path] = []
        seen: set[str] = set()

        for m in re.finditer(r"~([^~]+?\.tra)~", raw, flags=re.I):
            s = (m.group(1) or "").strip()
            s = self._expand_mod_folder_ref(s, tp2)
            s = s.replace("\\", "/")
            if not s or ":" in s:
                continue
            key = s.lower()
            if key in seen:
                continue
            seen.add(key)
            found.append(Path(s))
            if len(found) >= 20:
                break
        for m in re.finditer(r"\"([^\"]+?\.tra)\"", raw, flags=re.I):
            s = (m.group(1) or "").strip()
            s = self._expand_mod_folder_ref(s, tp2)
            s = s.replace("\\", "/")
            if not s or ":" in s:
                continue
            key = s.lower()
            if key in seen:
                continue
            seen.add(key)
            found.append(Path(s))
            if len(found) >= 20:
                break

        return found

    def _parse_list_components(self, text: str) -> list[tuple[int, str, str | None]]:
        out: list[tuple[int, str, str | None]] = []
        ver_rx = re.compile(r"\s*:\s*([0-9][0-9A-Za-z._-]*)\s*$")

        for raw in text.splitlines():
            line = raw.strip()
            if not line or "//" not in line or "#" not in line:
                continue

            prefix, comment = line.split("//", 1)
            ids = re.findall(r"#\s*(\d+)", prefix)
            if len(ids) < 2:
                continue

            cid = int(ids[-1])
            comment = comment.strip()

            ver: str | None = None
            name = comment
            mv = ver_rx.search(comment)
            if mv:
                ver = mv.group(1)
                name = comment[: mv.start()].rstrip()

            out.append((cid, name, ver))

        return out

    def _fallback_components_from_tp2(
        self, tp2: Path, lang_idx: int | None
    ) -> list[tuple[int, str, str | None]]:
        try:
            raw = tp2.read_text(encoding="utf-8", errors="replace")
        except Exception:
            raw = tp2.read_text(errors="replace")

        out: list[tuple[int, str, str | None]] = []
        label_map = self._extract_component_labels(tp2, use_index=False)
        blocks = re.split(r"^\s*BEGIN\b", raw, flags=re.M)

        rx_name = re.compile(r"~([^~]+)~|\"([^\"]+)\"|'([^']+)'")
        for idx, block in enumerate(blocks[1:], start=0):
            cid = idx
            m = re.search(r"\bDESIGNATED\s+(\d+)", block, flags=re.I)
            if not m:
                m = re.search(r"\bDESIGNATED\s*=\s*(\d+)", block, flags=re.I)
            if m:
                try:
                    cid = int(m.group(1))
                except ValueError:
                    cid = idx

            line = block.splitlines()[0] if block.splitlines() else ""
            name = ""
            m_at = re.search(r"@\s*(\d+)", line)
            if m_at:
                name = f"@{m_at.group(1)}"
            else:
                m_q = rx_name.search(line)
                if m_q:
                    name = (m_q.group(1) or m_q.group(2) or m_q.group(3) or "").strip()

            if name.startswith("@"):
                resolved = self._resolve_undefined_name(tp2, lang_idx, name)
                if resolved:
                    name = resolved

            if not name or name == ":" or name.startswith(":"):
                name = label_map.get(cid, "")

            if not name:
                name = f"Component {cid}"

            out.append((cid, name, None))

        return out


@dataclass(slots=True)
class _TreeBundle:
    host: QWidget
    view: QTreeView
    model: QStandardItemModel
    proxy: QSortFilterProxyModel
