# /// script
# requires-python = ">=3.9"
# dependencies = ["fonttools>=4.40"]
# ///
"""Shift a KH Dot font's outlines onto a baseline-aligned dot grid.

The KH Dot TTFs place their dot grid a quarter dot below the baseline
(outline y = -16 + 64k units at 1024 upem for the 16-dot face). egui
rasterizes each glyph relative to an integer-pixel baseline, so every
horizontal stroke is split across two pixel rows. Moving all outlines up by
the offset, and the vertical metrics with them, puts every dot edge on a whole
pixel.

Usage: uv run scripts/align_font.py <input.ttf> <output.ttf>
"""

import sys

from fontTools.ttLib import TTFont


def dot_grid_offset(font: TTFont, dots: int) -> int:
    """Upward shift (font units) that puts outline y coordinates on the dot grid."""
    unit = font["head"].unitsPerEm // dots
    glyf = font["glyf"]
    offsets = set()
    for name in font.getGlyphOrder():
        glyph = glyf[name]
        # `.notdef` is a hand-drawn box, not dot art.
        if name == ".notdef" or glyph.isComposite() or glyph.numberOfContours <= 0:
            continue
        offsets.update(-y % unit for _, y in glyph.coordinates)
    if len(offsets) != 1:
        sys.exit(f"outlines are not on a single {unit}-unit grid: offsets {sorted(offsets)}")
    return offsets.pop()


def main() -> None:
    src, dst = sys.argv[1:3]
    font = TTFont(src, recalcTimestamp=False)
    shift = dot_grid_offset(font, dots=16)

    glyf = font["glyf"]
    for name in font.getGlyphOrder():
        glyph = glyf[name]
        # Composites only reference simple glyphs, which move on their own.
        if not glyph.isComposite() and glyph.numberOfContours > 0:
            glyph.coordinates.translate((0, shift))
            glyph.recalcBounds(glyf)

    hhea, os2 = font["hhea"], font["OS/2"]
    hhea.ascent += shift
    hhea.descent += shift
    os2.sTypoAscender += shift
    os2.sTypoDescender += shift
    os2.usWinAscent += shift
    os2.usWinDescent -= shift

    font.save(dst)
    print(f"shifted outlines up by {shift} units -> {dst}")


if __name__ == "__main__":
    main()
