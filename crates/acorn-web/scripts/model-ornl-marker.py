#!/usr/bin/env python3
"""Author a compact, double-sided ORNL location marker. Standard library; MIT."""
import argparse
import importlib.util
import json
import math
from pathlib import Path
import re
from xml.etree import ElementTree


def create(m):
    outline = [(0, 0)] + [(1.12 * math.cos(a), 2.25 + 1.12 * math.sin(a))
                         for a in [math.radians(210 - 240 * i / 96) for i in range(97)]]
    inner = [(x * .946, 1.9 + (y - 1.9) * .946) for x, y in outline]
    rim = m.OTHER.beveled_outline(m, outline, .40, "steel", .025)
    shell = m.OTHER.beveled_outline(m, inner, .44, "teal", .032)
    front = [oak_leaf(m)]
    back = [dict(part, vertices=m.G.transform((part["vertices"], part["triangles"]), rotation=(0, math.pi, 0))[0]) for part in front]
    return [dict(rim, group="rim"), dict(shell, group="marker-body")] + [dict(part, group="identity-front") for part in front] + [dict(part, group="identity-back") for part in back]


def cubic_points(a, b, c, d, depth=0):
    """Flatten the source Bezier curve deterministically for triangulation."""
    excess = math.dist(a, b) + math.dist(b, c) + math.dist(c, d) - math.dist(a, d)
    if excess < .25 or depth == 10:
        result = [d]
    else:
        ab, bc, cd = [tuple((x + y) / 2 for x, y in zip(left, right)) for left, right in [(a, b), (b, c), (c, d)]]
        abc, bcd = [tuple((x + y) / 2 for x, y in zip(left, right)) for left, right in [(ab, bc), (bc, cd)]]
        middle = tuple((x + y) / 2 for x, y in zip(abc, bcd))
        result = cubic_points(a, ab, abc, middle, depth + 1) + cubic_points(middle, bcd, cd, d, depth + 1)
    return result


def logo_outline():
    """Read the single closed oak-leaf contour from the bundled white ORNL logo."""
    source = Path(__file__).resolve().parents[2] / "acorn-cli/assets/images/logo_ornl_white.svg"
    paths = list(ElementTree.parse(source).getroot().iter("{http://www.w3.org/2000/svg}path"))
    data = paths[-1].attrib["d"]
    assert len(re.findall("[Mm]", data)) == 1 and len(re.findall("[Zz]", data)) == 1
    tokens = re.findall(r"[A-Za-z]|[-+]?(?:\d*\.?\d+)(?:[eE][-+]?\d+)?", data)
    index, command = 0, ""
    current, control = (0, 0), (0, 0)
    points = []
    while index < len(tokens):
        if tokens[index].isalpha():
            command = tokens[index]
            index += 1
        assert command in {"M", "C", "c", "l", "s", "z"}, f"Unexpected ORNL SVG command: {command}"
        count = {"M": 2, "C": 6, "c": 6, "l": 2, "s": 4, "z": 0}[command]
        values = [float(value) for value in tokens[index:index + count]]
        index += count
        pairs = [tuple(values[i:i + 2]) for i in range(0, count, 2)]
        absolute = [tuple(current[axis] + point[axis] for axis in range(2)) for point in pairs] if command.islower() else pairs
        if command == "M":
            current = absolute[0]
            points.append(current)
        elif command == "l":
            current = absolute[0]
            points.append(current)
        elif command in {"C", "c", "s"}:
            controls = [tuple(2 * current[axis] - control[axis] for axis in range(2)), *absolute] if command == "s" else absolute
            points += cubic_points(current, *controls)
            control, current = controls[-2:]
    return points[:-1] if math.dist(points[0], points[-1]) < 1e-6 else points


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/models")
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("collection", Path(__file__).with_name("model-collection.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    value = m.recipe("research", "ornl-marker", "ORNL Marker", create(m), pedestal=False)
    value["revision"] = 3
    value["license"] = "MIT marker geometry; ORNL logo rights retained by its owner"
    value["copyright"] = "ACORN marker geometry: MIT. ORNL oak-leaf artwork: respective owner; derived from the bundled ORNL logo."
    value["references"] = ["../../../acorn-cli/assets/images/logo_ornl_white.svg"]
    value["provenance"] = "ORNL location marker: text-free beveled ORNL Green pin with the raised oak-leaf contour derived from the bundled acorn-cli/assets/images/logo_ornl_white.svg wordmark and extruded on both faces. Graphite rim and white emblem; no lettering, name or plaque. Point at bottom-center origin; +Y up and +Z front; illustrative 3.37-meter authoring size, intended to scale at placement."
    value["materials"]["teal"] = m.BRAND.material("teal", metallic=.08, roughness=.42)
    value["materials"]["steel"] = m.BRAND.material("steel", metallic=.72, roughness=.30)
    value["materials"]["white"] = m.BRAND.material("white", metallic=.05, roughness=.42)
    value["materials"]["white"]["emissiveFactor"] = [.04, .04, .04]
    target = args.root / "research/ornl-marker.model.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")
    print(f"Authored {value['id']}: {sum(len(part['triangles']) for part in value['parts'])} triangles")


def oak_leaf(m):
    points = logo_outline()
    lower = [min(point[axis] for point in points) for axis in range(2)]
    upper = [max(point[axis] for point in points) for axis in range(2)]
    scale = 1.50 / (upper[1] - lower[1])
    outline = [((x - (lower[0] + upper[0]) / 2) * scale, -(y - (lower[1] + upper[1]) / 2) * scale) for x, y in points]
    return m.E(outline, .032, "white", (0, 2.35, .236))


if __name__ == "__main__":
    main()
