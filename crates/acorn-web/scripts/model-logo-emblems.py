#!/usr/bin/env python3
"""Trace retained logo masks into editable, hole-preserving extrusion contours. MIT."""
import argparse
import json
import math
from pathlib import Path
from PIL import Image


def area(points):
    return sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(points,points[1:]+points[:1]))/2


def boundaries(image, tolerance=.65):
    mask = image.getchannel("A")
    width,height = mask.size
    pixels = mask.load()
    edges = {}
    for y in range(height):
        for x in range(width):
            if pixels[x,y] >= 128:
                sides = [((x,y),(x+1,y),y==0 or pixels[x,y-1]<128),
                         ((x+1,y),(x+1,y+1),x==width-1 or pixels[x+1,y]<128),
                         ((x+1,y+1),(x,y+1),y==height-1 or pixels[x,y+1]<128),
                         ((x,y+1),(x,y),x==0 or pixels[x-1,y]<128)]
                for start,end,exposed in sides:
                    if exposed:
                        edges.setdefault(start,set()).add(end)
    loops = []
    directions = {(1,0):0,(0,1):1,(-1,0):2,(0,-1):3}
    while edges:
        start = min(edges)
        points,current,heading = [],start,0
        while not points or current!=start:
            points.append(current)
            choices = edges[current]
            successor = min(choices,key=lambda point:[1,0,3,2].index((directions[(point[0]-current[0],point[1]-current[1])]-heading)%4))
            heading = directions[(successor[0]-current[0],successor[1]-current[1])]
            choices.remove(successor)
            if not choices:
                del edges[current]
            current = successor
        if abs(area(points)) >= 4:
            loops.append(simplify_closed(points,tolerance))
    return loops


def contains(polygon, point):
    x,y = point
    crossings = [a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1]) for a,b in zip(polygon,polygon[1:]+polygon[:1]) if (a[1]>y)!=(b[1]>y)]
    return sum(value>x for value in crossings)%2==1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]/"assets/models")
    args = parser.parse_args()
    folder = args.root/"designs/laboratory-logos/emblems"
    manifest = json.loads((folder/"catalog.json").read_text())
    for entry in manifest["emblems"]:
        image = Image.open(args.root/entry["mask"]).convert("RGBA")
        assert image.getchannel("A").getextrema()[0]==0, f"Emblem needs a transparent background: {entry['slug']}"
        image.thumbnail((1536,1536),Image.Resampling.LANCZOS)
        tolerance = max(.65,min(1.25,max(image.size)/1000))
        loops = boundaries(image,tolerance)
        outers = [points for points in loops if area(points)>0]
        holes = [points for points in loops if area(points)<0]
        shapes = [[outer] for outer in outers]
        for hole in holes:
            parents = [(abs(area(outer)),i) for i,outer in enumerate(outers) if contains(outer,hole[0])]
            assert parents, f"Uncontained logo cutout: {entry['slug']}"
            shapes[min(parents)[1]].append(hole)
        assert shapes, f"Empty emblem: {entry['slug']}"
        value = {"source":entry["mask"],"size":list(image.size),"shapes":shapes,"simplificationPixels":tolerance}
        (folder/f"{entry['slug']}.contours.json").write_text(json.dumps(value,sort_keys=True,separators=(",",":"))+"\n")
        print(f"Traced {entry['slug']}: {len(shapes)} components, {len(holes)} cutouts, {sum(len(ring) for shape in shapes for ring in shape)} vertices")


def simplify(points, tolerance):
    a,b = points[0],points[-1]
    length = math.dist(a,b)
    distances = [abs((b[0]-a[0])*(a[1]-point[1])-(a[0]-point[0])*(b[1]-a[1]))/length for point in points] if length else [math.dist(a,point) for point in points]
    furthest = max(range(len(points)),key=distances.__getitem__)
    return simplify(points[:furthest+1],tolerance)[:-1]+simplify(points[furthest:],tolerance) if distances[furthest]>tolerance else [a,b]


def simplify_closed(points, tolerance):
    split = max(range(len(points)),key=lambda i:math.dist(points[0],points[i]))
    return simplify(points[:split+1],tolerance)[:-1]+simplify(points[split:]+points[:1],tolerance)[:-1]


if __name__ == "__main__":
    main()
