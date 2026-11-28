#!/usr/bin/env python3
"""Author text-free DOE laboratory emblem pins and six plain color variants. MIT."""
import argparse
from collections import Counter
import importlib.util
import json
import math
from pathlib import Path

PLAIN = {"blue":"blue","brown":"orange","green":"teal","magenta":"magenta","purple":"purple","teal":"water"}


def clean_ring(points):
    previous = []
    while previous!=points:
        previous = points
        points = [b for a,b,c in zip(points[-1:]+points[:-1],points,points[1:]+points[:1])
                  if abs((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])) > 1e-7*math.dist(a,c)]
        assert len(points)>=3, "Logo contour collapsed"
    return points


def emblem(m, logo, source):
    shapes = source["shapes"]
    points = [point for shape in shapes for ring in shape for point in ring]
    center = [(min(p[axis] for p in points)+max(p[axis] for p in points))/2 for axis in range(2)]
    extent = [max(p[axis] for p in points)-min(p[axis] for p in points) for axis in range(2)]
    scale = min(1.56/extent[0],1.56/extent[1])
    radius = max(math.hypot((x-center[0])*scale,(y-center[1])*scale) for x,y in points)
    scale *= min(1,.88/radius)
    # Triangulate integer pixel coordinates so collinear points remain exact.
    shapes = [[clean_ring([(x,-y) for x,y in ring]) for ring in shape] for shape in shapes]
    shapes = [[ring if (logo.area(ring)>0)==(i==0) else list(reversed(ring)) for i,ring in enumerate(shape)] for shape in shapes]
    # A diagonal integer basis avoids horizontal hole-bridge alignment in Sandia's emblem.
    basis = [[[(x-y,x+y) for x,y in ring] for ring in shape] for shape in shapes]
    triangles = logo.triangulate(basis)
    for shape,faces in zip(shapes,triangles):
        flat = [point for ring in shape for point in ring]
        assert all(logo.area([flat[i] for i in face])>1e-12 for face in faces), "Invalid emblem cap triangle"
    shapes = [[[((x-center[0])*scale,(y+center[1])*scale) for x,y in ring] for ring in shape] for shape in shapes]
    return [extrusion(m,shape,faces) for shape,faces in zip(shapes,triangles)]


def extrusion(m, contours, triangles):
    points = [point for ring in contours for point in ring]
    count,offset = len(points),0
    vertices = [(x,y+2.35,z) for z in [.219,.253] for x,y in points]
    faces = [(a+count,b+count,c+count) for a,b,c in triangles]+[(c,b,a) for a,b,c in triangles]
    for ring in contours:
        for i in range(len(ring)):
            a,b = offset+i,offset+(i+1)%len(ring)
            faces += [(a,b,b+count),(a,b+count,a+count)]
        offset += len(ring)
    edges = Counter(tuple(sorted(pair)) for face in faces for pair in zip(face,face[1:]+face[:1]))
    assert all(count==2 for count in edges.values()), "Emblem extrusion must be closed"
    return dict(m.part((vertices,faces),"white"),group="identity-front")


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-","_"),Path(__file__).with_name(f"{name}.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root",type=Path,default=Path(__file__).resolve().parents[1]/"assets/models")
    args = parser.parse_args()
    m,logo = load("model-collection"),load("model-acorn-logo")
    manifest = json.loads((args.root/"designs/laboratory-logos/catalog.json").read_text())
    emblems = json.loads((args.root/"designs/laboratory-logos/emblems/catalog.json").read_text())
    artwork = {entry["slug"]:entry for entry in emblems["emblems"]}
    colors = json.loads((args.root/"designs/laboratory-logos/colors.json").read_text())
    brands = {entry["slug"]:entry for entry in colors["laboratories"]}
    assert len(manifest["laboratories"])==16
    for entry in manifest["laboratories"]:
        contours = json.loads((args.root/"designs/laboratory-logos/emblems"/f"{entry['slug']}.contours.json").read_text())
        color = brands[entry["slug"]]
        front = emblem(m,logo,contours)
        parts = shell(m,"lab-brand")+front+mirror(m,front)
        materials = {key:m.BRAND.material(key,metallic=.72 if key=="steel" else .05,roughness=.30 if key=="steel" else .42) for key in ["steel","white"]}
        materials["lab-brand"] = {"extras":{"brand":"laboratory brand","colorName":color["label"],"srgbHex":color["hex"],"source":color["sourceUrl"]},
                                  "pbrMetallicRoughness":{"baseColorFactor":[*m.BRAND.linear(color["hex"]),1],"metallicFactor":.08,"roughnessFactor":.42}}
        materials["white"]["emissiveFactor"] = [.04,.04,.04]
        value = recipe("research",f"{entry['slug']}-marker",f"{entry['abbreviation']} — {entry['name']} Marker",parts,materials)
        value["revision"] = 3
        value["license"] = "MIT marker geometry; laboratory logo rights retained by its owner"
        value["copyright"] = f"ACORN marker geometry: MIT. {entry['name']} emblem derived from official artwork: respective owner. See laboratory-logos/emblems/catalog.json."
        value["references"] = list(dict.fromkeys([entry["sourcePage"],entry["logoUrl"],artwork[entry["slug"]]["sourceUrl"],color["sourceUrl"]]))
        value["provenance"] = f"{entry['name']} location marker with a raised white graphic emblem on both faces, {color['label']} ({color['hex']}) body and Graphite rim. Text-free: no name, abbreviation, billboard or logo plaque. {artwork[entry['slug']]['notes']} Source-derived contours with open cutouts; logos retain their owners' rights. Bottom-center point, +Y up, +Z front; illustrative 3.37-meter authoring height."
        save(args.root,value)
    palette_path = args.root/"palette.json"
    palette = json.loads(palette_path.read_text())
    palette["laboratoryBrands"] = {entry["label"]:entry["hex"] for entry in colors["laboratories"] if entry["slug"]!="ornl"}
    palette["laboratoryBrandSource"] = "designs/laboratory-logos/colors.json"
    palette_path.write_text(json.dumps(palette,indent=2,sort_keys=True)+"\n")
    for color,role in sorted(PLAIN.items()):
        parts = shell(m,role)
        materials = {key:m.BRAND.material(key,metallic=.72 if key=="steel" else .08,roughness=.30 if key=="steel" else .42) for key in sorted({part["material"] for part in parts})}
        value = recipe("marker",f"plain-{color}",f"Plain Marker — {color.title()}",parts,materials)
        value["provenance"] = f"Plain {color} map pin with a beveled body and Graphite metallic rim. No text, logo or pedestal. Uses the recorded ORNL palette or requested natural brown finish. Bottom-center point, +Y up, +Z front; illustrative 3.37-meter authoring height."
        save(args.root,value)


def mirror(m, parts):
    return [dict(part,group="identity-back",vertices=m.G.transform((part["vertices"],part["triangles"]),rotation=(0,math.pi,0))[0]) for part in parts]


def recipe(category, name, label, parts, materials):
    return {"id":f"{category}/{name}","category":category,"label":label,"revision":1,"materials":materials,
            "parts":[dict(part,name=f"{part['group']}-{i}") for i,part in enumerate(parts)]}


def save(root, value):
    folder = root/("markers" if value["category"]=="marker" else value["category"])
    folder.mkdir(parents=True,exist_ok=True)
    (folder/f"{value['id'].split('/')[1]}.model.json").write_text(json.dumps(value,sort_keys=True,separators=(",",":"))+"\n")
    print(f"Authored {value['id']}")


def shell(m, material):
    outline = [(0,0)]+[(1.12*math.cos(a),2.25+1.12*math.sin(a)) for a in [math.radians(210-240*i/96) for i in range(97)]]
    inner = [(x*.946,1.9+(y-1.9)*.946) for x,y in outline]
    return [dict(m.OTHER.beveled_outline(m,outline,.40,"steel",.025),group="rim"),
            dict(m.OTHER.beveled_outline(m,inner,.44,material,.032),group="marker-body")]


if __name__ == "__main__":
    main()
