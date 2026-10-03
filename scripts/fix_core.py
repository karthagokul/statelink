# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later

from pathlib import Path
import re

path = Path("crates/statelink-core/src/bus.rs")
text = path.read_text()

pattern = re.compile(
    r"\.filter_map\(\|\(topic, record\)\| \{\s*"
    r"\(record\.active_session == Some\(session_id\)\)\.then\(\|\| topic\.clone\(\)\)\s*"
    r"\}\)"
)
replacement = (
    ".filter(|(_, record)| record.active_session == Some(session_id))\n"
    "            .map(|(topic, _)| topic.clone())"
)
text, count = pattern.subn(replacement, text, count=1)
if count != 1:
    raise SystemExit(f"expected one disconnect filter_map match, found {count}")

needle = "    fn declare(\n"
replacement = (
    "    // DECLARE mirrors the protocol fields explicitly; keeping these arguments visible\n"
    "    // avoids a duplicate internal transfer type while preserving the wire contract.\n"
    "    #[allow(clippy::too_many_arguments)]\n"
    "    fn declare(\n"
)
if needle not in text:
    raise SystemExit("declare function not found")
text = text.replace(needle, replacement, 1)

path.write_text(text)
