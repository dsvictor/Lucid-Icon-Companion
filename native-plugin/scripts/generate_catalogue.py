#!/usr/bin/env python3
"""Convert lucide-static SVG assets into SVG path data for the WASM plugin."""

import json
import math
import pathlib
import sys
import xml.etree.ElementTree as ET


def number(element, name, default=0.0):
    try:
        value = float(element.attrib.get(name, default))
    except (TypeError, ValueError) as error:
        raise ValueError(f"invalid {name} attribute") from error
    if not math.isfinite(value):
        raise ValueError(f"non-finite {name} attribute")
    return value


def path_for(element):
    tag = element.tag.rsplit("}", 1)[-1]

    if tag == "path":
        return element.attrib.get("d", "").strip()

    if tag == "line":
        x1, y1 = number(element, "x1"), number(element, "y1")
        x2, y2 = number(element, "x2"), number(element, "y2")
        return f"M{x1:g} {y1:g}L{x2:g} {y2:g}"

    if tag in ("polyline", "polygon"):
        raw = element.attrib.get("points", "").replace(",", " ").split()
        if len(raw) < 4 or len(raw) % 2:
            return ""
        try:
            values = [float(value) for value in raw]
        except ValueError:
            return ""
        if not all(math.isfinite(value) for value in values):
            return ""
        pairs = list(zip(values[0::2], values[1::2]))
        path = "M" + " ".join(f"{x:g} {y:g}" for x, y in pairs)
        return path + ("Z" if tag == "polygon" else "")

    if tag == "rect":
        x, y = number(element, "x"), number(element, "y")
        width, height = number(element, "width"), number(element, "height")
        if width <= 0 or height <= 0:
            return ""

        rx = number(element, "rx", number(element, "ry"))
        ry = number(element, "ry", rx)
        rx = min(max(rx, 0.0), width / 2.0)
        ry = min(max(ry, 0.0), height / 2.0)

        if rx == 0 or ry == 0:
            return f"M{x:g} {y:g}h{width:g}v{height:g}h{-width:g}Z"

        return (
            f"M{x + rx:g} {y:g}"
            f"h{width - 2 * rx:g}"
            f"q{rx:g} 0 {rx:g} {ry:g}"
            f"v{height - 2 * ry:g}"
            f"q0 {ry:g} {-rx:g} {ry:g}"
            f"h{-width + 2 * rx:g}"
            f"q{-rx:g} 0 {-rx:g} {-ry:g}"
            f"v{-height + 2 * ry:g}"
            f"q0 {-ry:g} {rx:g} {-ry:g}Z"
        )

    if tag in ("circle", "ellipse"):
        cx, cy = number(element, "cx"), number(element, "cy")
        rx = number(element, "r" if tag == "circle" else "rx")
        ry = rx if tag == "circle" else number(element, "ry")
        if rx <= 0 or ry <= 0:
            return ""
        k = 0.5522847498
        return (
            f"M{cx-rx:g} {cy:g}"
            f"C{cx-rx:g} {cy-k*ry:g} {cx-k*rx:g} {cy-ry:g} {cx:g} {cy-ry:g}"
            f"C{cx+k*rx:g} {cy-ry:g} {cx+rx:g} {cy-k*ry:g} {cx+rx:g} {cy:g}"
            f"C{cx+rx:g} {cy+k*ry:g} {cx+k*rx:g} {cy+ry:g} {cx:g} {cy+ry:g}"
            f"C{cx-k*rx:g} {cy+ry:g} {cx-rx:g} {cy+k*ry:g} {cx-rx:g} {cy:g}Z"
        )

    return ""


def build_catalogue(source):
    source = pathlib.Path(source)
    if not source.is_dir():
        raise SystemExit(f"Lucide SVG directory not found: {source}")

    icons = []
    for svg_file in sorted(source.glob("*.svg")):
        try:
            root = ET.parse(svg_file).getroot()
            path_data = " ".join(
                path for path in (path_for(element) for element in root.iter()) if path
            )
        except (ET.ParseError, OSError, ValueError) as error:
            print(f"Skipping {svg_file.name}: {error}", file=sys.stderr)
            continue

        if path_data:
            icons.append({"name": svg_file.stem, "path": path_data})

    if not icons:
        raise SystemExit(
            f"No usable Lucide icons found in {source}. "
            "Check the installed lucide-static package layout."
        )
    return icons


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: generate_catalogue.py ICONS_DIRECTORY OUTPUT_JSON")

    icons = build_catalogue(sys.argv[1])
    output = pathlib.Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps({"icons": icons}, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print(f"Bundled {len(icons)} Lucide icons into {output}")


if __name__ == "__main__":
    main()
