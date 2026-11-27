"""Deterministic four-meter surface tiles for original campus models."""
# SPDX-License-Identifier: MIT
import math
import struct
import zlib


def _chunk(kind, payload):
    return (struct.pack(">I", len(payload)) + kind + payload
            + struct.pack(">I", zlib.crc32(kind + payload)))


def _field(size, cells, seed):
    """Interpolate a periodic noise lattice without hard cell boundaries."""
    lattice = tuple(tuple(_hash(x, y, seed) & 255 for x in range(cells)) for y in range(cells))
    coordinates = tuple((pixel + 0.5) * cells / size for pixel in range(size))
    samples = tuple((int(value), (int(value) + 1) % cells,
                     (value % 1) ** 2 * (3 - 2 * (value % 1))) for value in coordinates)
    pixels = bytearray(size * size)
    for y, (row, following, blend) in enumerate(samples):
        values = tuple(a + (b - a) * blend for a, b in zip(lattice[row], lattice[following]))
        offset = y * size
        for x, (column, adjacent, weight) in enumerate(samples):
            pixels[offset + x] = int(values[column] + (values[adjacent] - values[column]) * weight + 0.5)
    return pixels


def _hash(x, y, seed):
    value = (x * 374761393 + y * 668265263 + seed * 1442695041) & 0xFFFFFFFF
    value = ((value ^ (value >> 13)) * 1274126177) & 0xFFFFFFFF
    return value ^ (value >> 16)


def _png(pixels, size):
    channels = tuple(bytes((value, value, value)) for value in range(256))
    scanlines = b"".join(b"\0" + b"".join(channels[value] for value in pixels[row:row + size])
                         for row in range(0, len(pixels), size))
    header = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + _chunk(b"IHDR", header)
            + _chunk(b"IDAT", zlib.compress(scanlines, 6)) + _chunk(b"IEND", b""))


def surface_png(kind, size=1024):
    """Return a repeatable grayscale RGB PNG for a four-meter square surface.

    Kinds are asphalt, brick, concrete, lawn and roof. Values stay between
    212 and 255: approximately 0.66-1.0 after glTF's sRGB-to-linear conversion,
    preserving the material's base color. These are color maps, not roughness
    or normal maps. Brick uses 16 running-bond units by 40 courses with 20 mm
    mortar joints; roof seams divide one-meter-wide, two-meter-long panels.
    """
    seeds = {"asphalt": 71, "brick": 13, "concrete": 43, "lawn": 97, "roof": 29}
    if kind not in seeds:
        raise ValueError(f"Unknown surface kind: {kind}")
    if not isinstance(size, int) or isinstance(size, bool) or size < 1:
        raise ValueError("Surface size must be a positive integer")
    seed = seeds[kind]
    broad = _field(size, 7 if kind == "lawn" else 4, seed)
    detail = _field(size, 31 if kind == "lawn" else 32, seed + 1)
    columns, courses = (16, 40) if kind == "brick" else (4, 2)
    x_units = tuple(tuple((x + 0.5) * columns / size + shift * 0.5 for x in range(size))
                    for shift in range(2))
    x_cells = tuple(tuple(int(value) % columns for value in row) for row in x_units)
    x_edges = tuple(tuple(min(value % 1, 1 - value % 1) * size / columns for value in row)
                    for row in x_units)
    tones = tuple(tuple(224 + (_hash(x, y, seed + 2) % 17) - 8 for x in range(columns))
                  for y in range(courses))
    joint_half_width = size * 0.02 / 4 / 2
    mowing_sine = tuple(math.sin(8 * math.pi * (x + 0.5) / size) for x in range(size))
    mowing_cosine = tuple(math.cos(8 * math.pi * (x + 0.5) / size) for x in range(size))
    pixels = bytearray(size * size)
    for y in range(size):
        unit = (y + 0.5) * courses / size
        course = int(unit)
        shift = course % 2
        edge_y = min(unit % 1, 1 - unit % 1) * size / courses
        phase = 0.35 * math.sin(2 * math.pi * (y + 0.5) / size)
        phase_sine, phase_cosine = math.sin(phase), math.cos(phase)
        offset = y * size
        for x in range(size):
            index = offset + x
            grain = (_hash(x, y, seed + 3) & 255) - 128
            cloud, fleck = broad[index] - 128, detail[index] - 128
            if kind == "brick":
                value = tones[course][x_cells[shift][x]] + cloud // 36 + fleck // 25 + grain // 32
                coverage = max(0, min(1, joint_half_width + 0.5 - min(x_edges[shift][x], edge_y)))
                value += (249 + grain // 64 - value) * coverage
            elif kind == "roof":
                value = 239 + cloud // 16 + fleck // 32 + grain // 48
                coverage = max(0, min(1, joint_half_width + 0.5 - min(x_edges[shift][x], edge_y)))
                value += (216 + grain // 64 - value) * coverage
            elif kind == "concrete":
                value = 241 + cloud // 28 + fleck // 24 + grain // 32 - (6 if grain < -122 else 0)
            elif kind == "asphalt":
                value = 232 + cloud // 20 + fleck // 35 + grain // 18
            else:
                stripe = 3 * (mowing_sine[x] * phase_cosine + mowing_cosine[x] * phase_sine)
                value = 232 + cloud // 13 + fleck // 19 + grain // 24 + stripe
            pixels[index] = max(212, min(255, int(value + 0.5)))
    return _png(pixels, size)
