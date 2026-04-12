#!/usr/bin/env python3
"""Convert all SVGs in ui/assets to 32x32 transparent PNGs."""

import os
from pathlib import Path

try:
    import cairosvg
except ImportError:
    raise SystemExit("cairosvg is required: pip install cairosvg")

ASSETS_DIR = Path(__file__).parent / "ui" / "assets"
SIZE = 32

svg_files = list(ASSETS_DIR.glob("*.svg"))
if not svg_files:
    print("No SVG files found in", ASSETS_DIR)
else:
    for svg_path in sorted(svg_files):
        out_path = svg_path.with_suffix(".png")
        cairosvg.svg2png(
            url=str(svg_path),
            write_to=str(out_path),
            output_width=SIZE,
            output_height=SIZE,
        )
        print(f"Converted: {svg_path.name} -> {out_path.name}")
    print(f"\nDone. {len(svg_files)} file(s) converted.")
