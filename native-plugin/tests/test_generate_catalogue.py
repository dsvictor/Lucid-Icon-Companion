#!/usr/bin/env python3
"""Unit tests for converting Lucide SVG primitives to FigCraft path data."""

import pathlib
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "scripts"))
from generate_catalogue import build_catalogue, path_for


class SvgPathTests(unittest.TestCase):
    def element(self, svg):
        return ET.fromstring(svg)

    def test_line_and_polyline(self):
        self.assertEqual(path_for(self.element('<line x1="1" y1="2" x2="3" y2="4"/>')), "M1 2L3 4")
        self.assertEqual(path_for(self.element('<polyline points="1,2 3,4 5,6"/>')), "M1 2 3 4 5 6")

    def test_polygon_is_closed(self):
        self.assertTrue(path_for(self.element('<polygon points="0,0 10,0 5,8"/>')).endswith("Z"))

    def test_rect_supports_rounded_corners(self):
        path = path_for(self.element('<rect x="1" y="2" width="20" height="12" rx="2"/>'))
        self.assertTrue(path.startswith("M3 2"))
        self.assertIn("a", path)
        self.assertTrue(path.endswith("Z"))

    def test_circle_and_ellipse_use_exact_arc_geometry(self):
        self.assertIn("a", path_for(self.element('<circle cx="12" cy="12" r="5"/>')))
        self.assertIn("a", path_for(self.element('<ellipse cx="12" cy="12" rx="6" ry="3"/>')))

    def test_catalogue_preserves_individual_elements_and_viewbox(self):
        with tempfile.TemporaryDirectory() as temp:
            icons_dir = pathlib.Path(temp) / "icons"
            icons_dir.mkdir()
            (icons_dir / "multi.svg").write_text(
                '<svg viewBox="0 0 24 24"><path d="M2 12h20"/><circle cx="12" cy="12" r="3"/></svg>',
                encoding="utf-8",
            )
            icons = build_catalogue(icons_dir)
            self.assertEqual(icons[0]["name"], "multi")
            self.assertEqual(icons[0]["viewBox"], "0 0 24 24")
            self.assertEqual(len(icons[0]["paths"]), 2)
            self.assertEqual(icons[0]["paths"][0], "M2 12h20")
            self.assertIn("a", icons[0]["paths"][1])
            self.assertIn("<circle", icons[0]["svg"])


if __name__ == "__main__":
    unittest.main()
