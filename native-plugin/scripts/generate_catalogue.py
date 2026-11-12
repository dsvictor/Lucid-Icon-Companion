#!/usr/bin/env python3
"""Bundle original Lucide SVG source and exact path geometry for FigCraft."""

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
        return f"M{number(element,'x1'):g} {number(element,'y1'):g}L{number(element,'x2'):g} {number(element,'y2'):g}"
    if tag in ("polyline", "polygon"):
        raw = element.attrib.get("points", "").replace(",", " ").split()
        if len(raw) < 4 or len(raw) % 2:
            return ""
        values = [float(value) for value in raw]
        if not all(math.isfinite(value) for value in values):
            return ""
        pairs = list(zip(values[0::2], values[1::2]))
        return "M" + " ".join(f"{x:g} {y:g}" for x, y in pairs) + ("Z" if tag == "polygon" else "")
    if tag == "rect":
        x, y = number(element, "x"), number(element, "y")
        w, h = number(element, "width"), number(element, "height")
        if w <= 0 or h <= 0:
            return ""
        rx = min(max(number(element, "rx", number(element, "ry")), 0), w / 2)
        ry = min(max(number(element, "ry", rx), 0), h / 2)
        if rx == 0 or ry == 0:
            return f"M{x:g} {y:g}h{w:g}v{h:g}h{-w:g}Z"
        return (f"M{x+rx:g} {y:g}h{w-2*rx:g}a{rx:g} {ry:g} 0 0 1 {rx:g} {ry:g}"
                f"v{h-2*ry:g}a{rx:g} {ry:g} 0 0 1 {-rx:g} {ry:g}"
                f"h{-w+2*rx:g}a{rx:g} {ry:g} 0 0 1 {-rx:g} {-ry:g}"
                f"v{-h+2*ry:g}a{rx:g} {ry:g} 0 0 1 {rx:g} {-ry:g}Z")
    if tag in ("circle", "ellipse"):
        cx, cy = number(element, "cx"), number(element, "cy")
        rx = number(element, "r" if tag == "circle" else "rx")
        ry = rx if tag == "circle" else number(element, "ry")
        if rx <= 0 or ry <= 0:
            return ""
        return (f"M{cx-rx:g} {cy:g}a{rx:g} {ry:g} 0 1 0 {2*rx:g} 0"
                f"a{rx:g} {ry:g} 0 1 0 {-2*rx:g} 0Z")
    return ""


def build_catalogue(source):
    source = pathlib.Path(source)
    if not source.is_dir():
        raise SystemExit(f"Lucide SVG directory not found: {source}")
    icons = []
    for svg_file in sorted(source.glob("*.svg")):
        try:
            raw_svg = svg_file.read_text(encoding="utf-8")
            root = ET.fromstring(raw_svg)
            view_box = root.attrib.get("viewBox", "0 0 24 24")
            vb = [float(part) for part in view_box.replace(",", " ").split()]
            if len(vb) != 4 or not all(math.isfinite(v) for v in vb) or vb != [0.0, 0.0, 24.0, 24.0]:
                raise ValueError(f"unexpected Lucide viewBox: {view_box}")
            paths = [p for element in root.iter() if (p := path_for(element))]
        except (ET.ParseError, OSError, ValueError) as error:
            print(f"Skipping {svg_file.name}: {error}", file=sys.stderr)
            continue
        if paths:
            icons.append({"name": svg_file.stem, "viewBox": view_box, "svg": raw_svg, "paths": paths})
    if not icons:
        raise SystemExit(f"No usable Lucide icons found in {source}.")
    return icons


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: generate_catalogue.py ICONS_DIRECTORY OUTPUT_JSON")
    icons = build_catalogue(sys.argv[1])
    output = pathlib.Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"icons": icons}, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"Bundled {len(icons)} original Lucide SVG sources into {output}")


if __name__ == "__main__":
    main()
