#!/usr/bin/env python3
"""Convert lucide-static SVG files into the compact path catalogue embedded in the WASM plugin."""

import json
import math
import pathlib
import sys
import xml.etree.ElementTree as ET


def attr(element, name, default="0"):
    return element.attrib.get(name, default)


def path_for(element):
    tag = element.tag.rsplit("}", 1)[-1]
    if tag == "path":
        return element.attrib.get("d", "")
    if tag == "line":
        return f"M{attr(element,'x1')} {attr(element,'y1')}L{attr(element,'x2')} {attr(element,'y2')}"
    if tag in ("polyline", "polygon"):
        values = [float(v) for v in element.attrib.get("points", "").replace(",", " ").split()]
        if len(values) < 4:
            return ""
        pairs = list(zip(values[0::2], values[1::2]))
        d = "M" + " ".join(f"{x:g} {y:g}" for x, y in pairs) + " "
        return d + ("Z" if tag == "polygon" else "")
    if tag == "rect":
        x, y = float(attr(element, "x")), float(attr(element, "y"))
        w, h = float(attr(element, "width")), float(attr(element, "height"))
        if w <= 0 or h <= 0:
            return ""
        return f"M{x:g} {y:g}h{w:g}v{h:g}h{-w:g}Z"
    if tag in ("circle", "ellipse"):
        cx, cy = float(attr(element, "cx")), float(attr(element, "cy"))
        rx = float(attr(element, "r" if tag == "circle" else "rx"))
        ry = rx if tag == "circle" else float(attr(element, "ry"))
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


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: generate_catalogue.py ICONS_DIRECTORY OUTPUT_JSON")
    source = pathlib.Path(sys.argv[1])
    output = pathlib.Path(sys.argv[2])
    icons = []
    for svg_file in sorted(source.glob("*.svg")):
        try:
            root = ET.parse(svg_file).getroot()
        except (ET.ParseError, OSError):
            continue
        paths = [path_for(child) for child in root.iter()]
        path_data = " ".join(part for part in paths if part)
        if path_data:
            icons.append({"name": svg_file.stem, "path": path_data})
    output.write_text(json.dumps({"icons": icons}, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"Bundled {len(icons)} Lucide icons into {output}")


if __name__ == "__main__":
    main()
